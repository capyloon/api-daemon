//! Facilities to build circuits directly, instead of via a circuit manager.

use crate::path::{OwnedPath, TorPath};
use crate::timeouts::{self, Action};
use crate::{Error, Result};
use async_trait::async_trait;
use futures::channel::oneshot;
use futures::task::SpawnExt;
use futures::Future;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tor_chanmgr::{ChanMgr, ChanProvenance, ChannelUsage};
use tor_guardmgr::GuardStatus;
use tor_linkspec::{ChanTarget, OwnedChanTarget, OwnedCircTarget};
use tor_proto::circuit::{CircParameters, ClientCirc, PendingClientCirc};
use tor_rtcompat::{Runtime, SleepProviderExt};

mod guardstatus;

pub(crate) use guardstatus::GuardStatusHandle;

/// Represents an objects that can be constructed in a circuit-like way.
///
/// This is only a separate trait for testing purposes, so that we can swap
/// our some other type when we're testing Builder.
///
/// TODO: I'd like to have a simpler testing strategy here; this one
/// complicates things a bit.
#[async_trait]
pub(crate) trait Buildable: Sized {
    /// Launch a new one-hop circuit to a given relay, given only a
    /// channel target `ct` specifying that relay.
    ///
    /// (Since we don't have a CircTarget here, we can't extend the circuit
    /// to be multihop later on.)
    async fn create_chantarget<RT: Runtime>(
        chanmgr: &ChanMgr<RT>,
        rt: &RT,
        guard_status: &GuardStatusHandle,
        ct: &OwnedChanTarget,
        params: &CircParameters,
        usage: ChannelUsage,
    ) -> Result<Self>;

    /// Launch a new circuit through a given relay, given a circuit target
    /// `ct` specifying that relay.
    async fn create<RT: Runtime>(
        chanmgr: &ChanMgr<RT>,
        rt: &RT,
        guard_status: &GuardStatusHandle,
        ct: &OwnedCircTarget,
        params: &CircParameters,
        usage: ChannelUsage,
    ) -> Result<Self>;

    /// Extend this circuit-like object by one hop, to the location described
    /// in `ct`.
    async fn extend<RT: Runtime>(
        &self,
        rt: &RT,
        ct: &OwnedCircTarget,
        params: &CircParameters,
    ) -> Result<()>;
}

/// Try to make a [`PendingClientCirc`] to a given relay, and start its
/// reactor.
///
/// This is common code, shared by all the first-hop functions in the
/// implementation of `Buildable` for `Arc<ClientCirc>`.
async fn create_common<RT: Runtime, CT: ChanTarget>(
    chanmgr: &ChanMgr<RT>,
    rt: &RT,
    target: &CT,
    guard_status: &GuardStatusHandle,
    usage: ChannelUsage,
) -> Result<PendingClientCirc> {
    // Get or construct the channel.
    let result = chanmgr.get_or_launch(target, usage).await;

    // Report the clock skew if appropriate, and exit if there has been an error.
    let chan = match result {
        Ok((chan, ChanProvenance::NewlyCreated)) => {
            guard_status.skew(chan.clock_skew());
            chan
        }
        Ok((chan, _)) => chan,
        Err(cause) => {
            if let Some(skew) = cause.clock_skew() {
                guard_status.skew(skew);
            }
            return Err(Error::Channel {
                peer: OwnedChanTarget::from_chan_target(target),
                cause,
            });
        }
    };
    // Construct the (zero-hop) circuit.
    let (pending_circ, reactor) = chan.new_circ().await.map_err(|error| Error::Protocol {
        error,
        peer: None, // we don't blame the peer, because new_circ() does no networking.
        action: "initializing circuit",
    })?;

    rt.spawn(async {
        let _ = reactor.run().await;
    })
    .map_err(|e| Error::from_spawn("circuit reactor task", e))?;

    Ok(pending_circ)
}

#[async_trait]
impl Buildable for ClientCirc {
    async fn create_chantarget<RT: Runtime>(
        chanmgr: &ChanMgr<RT>,
        rt: &RT,
        guard_status: &GuardStatusHandle,
        ct: &OwnedChanTarget,
        params: &CircParameters,
        usage: ChannelUsage,
    ) -> Result<Self> {
        let circ = create_common(chanmgr, rt, ct, guard_status, usage).await?;
        circ.create_firsthop_fast(params)
            .await
            .map_err(|error| Error::Protocol {
                peer: Some(ct.clone()),
                error,
                action: "running CREATE_FAST handshake",
            })
    }
    async fn create<RT: Runtime>(
        chanmgr: &ChanMgr<RT>,
        rt: &RT,
        guard_status: &GuardStatusHandle,
        ct: &OwnedCircTarget,
        params: &CircParameters,
        usage: ChannelUsage,
    ) -> Result<Self> {
        let circ = create_common(chanmgr, rt, ct, guard_status, usage).await?;
        circ.create_firsthop_ntor(ct, params.clone())
            .await
            .map_err(|error| Error::Protocol {
                peer: Some(OwnedChanTarget::from_chan_target(ct)),
                error,
                action: "creating first hop",
            })
    }
    async fn extend<RT: Runtime>(
        &self,
        _rt: &RT,
        ct: &OwnedCircTarget,
        params: &CircParameters,
    ) -> Result<()> {
        self.extend_ntor(ct, params)
            .await
            .map_err(|error| Error::Protocol {
                error,
                // We can't know who caused the error, since it may have been
                // the hop we were extending from, or the hop we were extending
                // to.
                peer: None,
                action: "extending circuit",
            })
    }
}

/// An implementation type for [`CircuitBuilder`].
///
/// A `CircuitBuilder` holds references to all the objects that are needed
/// to build circuits correctly.
///
/// In general, you should not need to construct or use this object yourself,
/// unless you are choosing your own paths.
struct Builder<R: Runtime, C: Buildable + Sync + Send + 'static> {
    /// The runtime used by this circuit builder.
    runtime: R,
    /// A channel manager that this circuit builder uses to make channels.
    chanmgr: Arc<ChanMgr<R>>,
    /// An estimator to determine the correct timeouts for circuit building.
    timeouts: timeouts::Estimator,
    /// We don't actually hold any clientcircs, so we need to put this
    /// type here so the compiler won't freak out.
    _phantom: std::marker::PhantomData<C>,
}

impl<R: Runtime, C: Buildable + Sync + Send + 'static> Builder<R, C> {
    /// Construct a new [`Builder`].
    fn new(runtime: R, chanmgr: Arc<ChanMgr<R>>, timeouts: timeouts::Estimator) -> Self {
        Builder {
            runtime,
            chanmgr,
            timeouts,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Build a circuit, without performing any timeout operations.
    ///
    /// After each hop is built, increments n_hops_built.  Make sure that
    /// `guard_status` has its pending status set correctly to correspond
    /// to a circuit failure at any given stage.
    ///
    /// (TODO: Find
    /// a better design there.)
    async fn build_notimeout(
        self: Arc<Self>,
        path: OwnedPath,
        params: CircParameters,
        start_time: Instant,
        n_hops_built: Arc<AtomicU32>,
        guard_status: Arc<GuardStatusHandle>,
        usage: ChannelUsage,
    ) -> Result<C> {
        match path {
            OwnedPath::ChannelOnly(target) => {
                // If we fail now, it's the guard's fault.
                guard_status.pending(GuardStatus::Failure);
                let circ = C::create_chantarget(
                    &self.chanmgr,
                    &self.runtime,
                    &guard_status,
                    &target,
                    &params,
                    usage,
                )
                .await?;
                self.timeouts
                    .note_hop_completed(0, self.runtime.now() - start_time, true);
                n_hops_built.fetch_add(1, Ordering::SeqCst);
                Ok(circ)
            }
            OwnedPath::Normal(p) => {
                assert!(!p.is_empty());
                let n_hops = p.len() as u8;
                // If we fail now, it's the guard's fault.
                guard_status.pending(GuardStatus::Failure);
                let circ = C::create(
                    &self.chanmgr,
                    &self.runtime,
                    &guard_status,
                    &p[0],
                    &params,
                    usage,
                )
                .await?;
                self.timeouts
                    .note_hop_completed(0, self.runtime.now() - start_time, n_hops == 0);
                // If we fail after this point, we can't tell whether it's
                // the fault of the guard or some later relay.
                guard_status.pending(GuardStatus::Indeterminate);
                n_hops_built.fetch_add(1, Ordering::SeqCst);
                let mut hop_num = 1;
                for relay in p[1..].iter() {
                    circ.extend(&self.runtime, relay, &params).await?;
                    n_hops_built.fetch_add(1, Ordering::SeqCst);
                    self.timeouts.note_hop_completed(
                        hop_num,
                        self.runtime.now() - start_time,
                        hop_num == (n_hops - 1),
                    );
                    hop_num += 1;
                }
                Ok(circ)
            }
        }
    }

    /// Build a circuit from an [`OwnedPath`].
    async fn build_owned(
        self: &Arc<Self>,
        path: OwnedPath,
        params: &CircParameters,
        guard_status: Arc<GuardStatusHandle>,
        usage: ChannelUsage,
    ) -> Result<C> {
        let action = Action::BuildCircuit { length: path.len() };
        let (timeout, abandon_timeout) = self.timeouts.timeouts(&action);
        let start_time = self.runtime.now();

        // TODO: This is probably not the best way for build_notimeout to
        // tell us how many hops it managed to build, but at least it is
        // isolated here.
        let hops_built = Arc::new(AtomicU32::new(0));

        let self_clone = Arc::clone(self);
        let params = params.clone();

        let circuit_future = self_clone.build_notimeout(
            path,
            params,
            start_time,
            Arc::clone(&hops_built),
            guard_status,
            usage,
        );

        match double_timeout(&self.runtime, circuit_future, timeout, abandon_timeout).await {
            Ok(circuit) => Ok(circuit),
            Err(Error::CircTimeout) => {
                let n_built = hops_built.load(Ordering::SeqCst);
                self.timeouts
                    .note_circ_timeout(n_built as u8, self.runtime.now() - start_time);
                Err(Error::CircTimeout)
            }
            Err(e) => Err(e),
        }
    }

    /// Return a reference to this Builder runtime.
    pub(crate) fn runtime(&self) -> &R {
        &self.runtime
    }
}

/// A factory object to build circuits.
///
/// A `CircuitBuilder` holds references to all the objects that are needed
/// to build circuits correctly.
///
/// In general, you should not need to construct or use this object yourself,
/// unless you are choosing your own paths.
pub struct CircuitBuilder<R: Runtime> {
    /// The underlying [`Builder`] object
    builder: Arc<Builder<R, ClientCirc>>,
    /// Configuration for how to choose paths for circuits.
    path_config: tor_config::MutCfg<crate::PathConfig>,
    /// State-manager object to use in storing current state.
    storage: crate::TimeoutStateHandle,
    /// Guard manager to tell us which guards nodes to use for the circuits
    /// we build.
    guardmgr: tor_guardmgr::GuardMgr<R>,
}

impl<R: Runtime> CircuitBuilder<R> {
    /// Construct a new [`CircuitBuilder`].
    // TODO: eventually I'd like to make this a public function, but
    // TimeoutStateHandle is private.
    pub(crate) fn new(
        runtime: R,
        chanmgr: Arc<ChanMgr<R>>,
        path_config: crate::PathConfig,
        storage: crate::TimeoutStateHandle,
        guardmgr: tor_guardmgr::GuardMgr<R>,
    ) -> Self {
        let timeouts = timeouts::Estimator::from_storage(&storage);

        CircuitBuilder {
            builder: Arc::new(Builder::new(runtime, chanmgr, timeouts)),
            path_config: path_config.into(),
            storage,
            guardmgr,
        }
    }

    /// Return this builder's [`PathConfig`](crate::PathConfig).
    pub(crate) fn path_config(&self) -> Arc<crate::PathConfig> {
        self.path_config.get()
    }

    /// Replace this builder's [`PathConfig`](crate::PathConfig).
    pub(crate) fn set_path_config(&self, new_config: crate::PathConfig) {
        self.path_config.replace(new_config);
    }

    /// Flush state to the state manager if we own the lock.
    ///
    /// Return `Ok(true)` if we saved, and `Ok(false)` if we didn't hold the lock.
    pub(crate) fn save_state(&self) -> Result<bool> {
        if !self.storage.can_store() {
            return Ok(false);
        }
        // TODO: someday we'll want to only do this if there is something
        // changed.
        self.builder.timeouts.save_state(&self.storage)?;
        self.guardmgr.store_persistent_state()?;
        Ok(true)
    }

    /// Replace our state with a new owning state, assuming we have
    /// storage permission.
    pub(crate) fn upgrade_to_owned_state(&self) -> Result<()> {
        self.builder
            .timeouts
            .upgrade_to_owning_storage(&self.storage);
        self.guardmgr.upgrade_to_owned_persistent_state()?;
        Ok(())
    }
    /// Reload persistent state from disk, if we don't have storage permission.
    pub(crate) fn reload_state(&self) -> Result<()> {
        if !self.storage.can_store() {
            self.builder
                .timeouts
                .reload_readonly_from_storage(&self.storage);
        }
        self.guardmgr.reload_persistent_state()?;
        Ok(())
    }

    /// Reconfigure this builder using the latest set of network parameters.
    ///
    /// (NOTE: for now, this only affects circuit timeout estimation.)
    pub fn update_network_parameters(&self, p: &tor_netdir::params::NetParameters) {
        self.builder.timeouts.update_params(p);
    }

    /// Like `build`, but construct a new circuit from an [`OwnedPath`].
    pub(crate) async fn build_owned(
        &self,
        path: OwnedPath,
        params: &CircParameters,
        guard_status: Arc<GuardStatusHandle>,
        usage: ChannelUsage,
    ) -> Result<ClientCirc> {
        self.builder
            .build_owned(path, params, guard_status, usage)
            .await
    }

    /// Try to construct a new circuit from a given path, using appropriate
    /// timeouts.
    ///
    /// This circuit is _not_ automatically registered with any
    /// circuit manager; if you don't hang on it it, it will
    /// automatically go away when the last reference is dropped.
    pub async fn build(
        &self,
        path: &TorPath<'_>,
        params: &CircParameters,
        usage: ChannelUsage,
    ) -> Result<ClientCirc> {
        let owned = path.try_into()?;
        self.build_owned(owned, params, Arc::new(None.into()), usage)
            .await
    }

    /// Return true if this builder is currently learning timeout info.
    pub(crate) fn learning_timeouts(&self) -> bool {
        self.builder.timeouts.learning_timeouts()
    }

    /// Return a reference to this builder's `GuardMgr`.
    pub(crate) fn guardmgr(&self) -> &tor_guardmgr::GuardMgr<R> {
        &self.guardmgr
    }

    /// Return a reference to this builder's runtime
    pub(crate) fn runtime(&self) -> &R {
        self.builder.runtime()
    }
}

/// Helper function: spawn a future as a background task, and run it with
/// two separate timeouts.
///
/// If the future does not complete by `timeout`, then return a
/// timeout error immediately, but keep running the future in the
/// background.
///
/// If the future does not complete by `abandon`, then abandon the
/// future completely.
async fn double_timeout<R, F, T>(
    runtime: &R,
    fut: F,
    timeout: Duration,
    abandon: Duration,
) -> Result<T>
where
    R: Runtime,
    F: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let (snd, rcv) = oneshot::channel();
    let rt = runtime.clone();
    // We create these futures now, since we want them to look at the current
    // time when they decide when to expire.
    let inner_timeout_future = rt.timeout(abandon, fut);
    let outer_timeout_future = rt.timeout(timeout, rcv);

    runtime
        .spawn(async move {
            let result = inner_timeout_future.await;
            let _ignore_cancelled_error = snd.send(result);
        })
        .map_err(|e| Error::from_spawn("circuit construction task", e))?;

    let outcome = outer_timeout_future.await;
    // 4 layers of error to collapse:
    //     One from the receiver being cancelled.
    //     One from the outer timeout.
    //     One from the inner timeout.
    //     One from the actual future's result.
    //
    // (Technically, we could refrain from unwrapping the future's result,
    // but doing it this way helps make it more certain that we really are
    // collapsing all the layers into one.)
    outcome???
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::timeouts::TimeoutEstimator;
    use futures::channel::oneshot;
    use std::sync::Mutex;
    use tor_chanmgr::ChannelConfig;
    use tor_chanmgr::ChannelUsage as CU;
    use tor_linkspec::{HasRelayIds, RelayIdType, RelayIds};
    use tor_llcrypto::pk::ed25519::Ed25519Identity;
    use tor_rtcompat::{test_with_all_runtimes, SleepProvider};
    use tracing::trace;

    /// Make a new nonfunctional `Arc<GuardStatusHandle>`
    fn gs() -> Arc<GuardStatusHandle> {
        Arc::new(None.into())
    }

    #[test]
    // TODO: re-enable this test after arti#149 is fixed. For now, it
    // is not reliable enough.
    fn test_double_timeout() {
        let t1 = Duration::from_secs(1);
        let t10 = Duration::from_secs(10);
        /// Return true if d1 is in range [d2...d2 + 0.5sec]
        fn duration_close_to(d1: Duration, d2: Duration) -> bool {
            d1 >= d2 && d1 <= d2 + Duration::from_millis(500)
        }

        test_with_all_runtimes!(|rto| async move {
            #[allow(clippy::clone_on_copy)]
            let rt = tor_rtmock::MockSleepRuntime::new(rto.clone());

            // Try a future that's ready immediately.
            let x = double_timeout(&rt, async { Ok(3_u32) }, t1, t10).await;
            assert!(x.is_ok());
            assert_eq!(x.unwrap(), 3_u32);

            trace!("acquiesce after test1");
            #[allow(clippy::clone_on_copy)]
            let rt = tor_rtmock::MockSleepRuntime::new(rto.clone());

            // Try a future that's ready after a short delay.
            let rt_clone = rt.clone();
            // (We only want the short delay to fire, not any of the other timeouts.)
            rt_clone.block_advance("manually controlling advances");
            let x = rt
                .wait_for(double_timeout(
                    &rt,
                    async move {
                        let sl = rt_clone.sleep(Duration::from_millis(100));
                        rt_clone.allow_one_advance(Duration::from_millis(100));
                        sl.await;
                        Ok(4_u32)
                    },
                    t1,
                    t10,
                ))
                .await;
            assert!(x.is_ok());
            assert_eq!(x.unwrap(), 4_u32);

            trace!("acquiesce after test2");
            #[allow(clippy::clone_on_copy)]
            let rt = tor_rtmock::MockSleepRuntime::new(rto.clone());

            // Try a future that passes the first timeout, and make sure that
            // it keeps running after it times out.
            let rt_clone = rt.clone();
            let (snd, rcv) = oneshot::channel();
            let start = rt.now();
            rt.block_advance("manually controlling advances");
            let x = rt
                .wait_for(double_timeout(
                    &rt,
                    async move {
                        let sl = rt_clone.sleep(Duration::from_secs(2));
                        rt_clone.allow_one_advance(Duration::from_secs(2));
                        sl.await;
                        snd.send(()).unwrap();
                        Ok(4_u32)
                    },
                    t1,
                    t10,
                ))
                .await;
            assert!(matches!(x, Err(Error::CircTimeout)));
            let end = rt.now();
            assert!(duration_close_to(end - start, Duration::from_secs(1)));
            let waited = rt.wait_for(rcv).await;
            assert_eq!(waited, Ok(()));

            trace!("acquiesce after test3");
            #[allow(clippy::clone_on_copy)]
            let rt = tor_rtmock::MockSleepRuntime::new(rto.clone());

            // Try a future that times out and gets abandoned.
            let rt_clone = rt.clone();
            rt.block_advance("manually controlling advances");
            let (snd, rcv) = oneshot::channel();
            let start = rt.now();
            // Let it hit the first timeout...
            rt.allow_one_advance(Duration::from_secs(1));
            let x = rt
                .wait_for(double_timeout(
                    &rt,
                    async move {
                        rt_clone.sleep(Duration::from_secs(30)).await;
                        snd.send(()).unwrap();
                        Ok(4_u32)
                    },
                    t1,
                    t10,
                ))
                .await;
            assert!(matches!(x, Err(Error::CircTimeout)));
            let end = rt.now();
            // ...and let it hit the second, too.
            rt.allow_one_advance(Duration::from_secs(9));
            let waited = rt.wait_for(rcv).await;
            assert!(waited.is_err());
            let end2 = rt.now();
            assert!(duration_close_to(end - start, Duration::from_secs(1)));
            assert!(duration_close_to(end2 - start, Duration::from_secs(10)));
        });
    }

    /// Get a pair of timeouts that we've encoded as an Ed25519 identity.
    ///
    /// In our FakeCircuit code below, the first timeout is the amount of
    /// time that we should sleep while building a hop to this key,
    /// and the second timeout is the length of time-advance we should allow
    /// after the hop is built.
    ///
    /// (This is pretty silly, but it's good enough for testing.)
    fn timeouts_from_key(id: &Ed25519Identity) -> (Duration, Duration) {
        let mut be = [0; 8];
        be[..].copy_from_slice(&id.as_bytes()[0..8]);
        let dur = u64::from_be_bytes(be);
        be[..].copy_from_slice(&id.as_bytes()[8..16]);
        let dur2 = u64::from_be_bytes(be);
        (Duration::from_millis(dur), Duration::from_millis(dur2))
    }
    /// Encode a pair of timeouts as an Ed25519 identity.
    ///
    /// In our FakeCircuit code below, the first timeout is the amount of
    /// time that we should sleep while building a hop to this key,
    /// and the second timeout is the length of time-advance we should allow
    /// after the hop is built.
    ///
    /// (This is pretty silly but it's good enough for testing.)
    fn key_from_timeouts(d1: Duration, d2: Duration) -> Ed25519Identity {
        let mut bytes = [0; 32];
        let dur = (d1.as_millis() as u64).to_be_bytes();
        bytes[0..8].copy_from_slice(&dur);
        let dur = (d2.as_millis() as u64).to_be_bytes();
        bytes[8..16].copy_from_slice(&dur);
        bytes.into()
    }

    /// As [`timeouts_from_key`], but first extract the relevant key from the
    /// OwnedChanTarget.
    fn timeouts_from_chantarget<CT: ChanTarget>(ct: &CT) -> (Duration, Duration) {
        // Extracting the Ed25519 identity should always succeed in this case:
        // we put it there ourselves!
        let ed_id = ct
            .identity(RelayIdType::Ed25519)
            .expect("No ed25519 key was present for fake ChanTarget‽")
            .try_into()
            .expect("ChanTarget provided wrong key type");
        timeouts_from_key(ed_id)
    }

    /// Replacement type for circuit, to implement buildable.
    #[derive(Clone)]
    struct FakeCirc {
        hops: Vec<RelayIds>,
        onehop: bool,
    }
    #[async_trait]
    impl Buildable for Mutex<FakeCirc> {
        async fn create_chantarget<RT: Runtime>(
            _: &ChanMgr<RT>,
            rt: &RT,
            _guard_status: &GuardStatusHandle,
            ct: &OwnedChanTarget,
            _: &CircParameters,
            _usage: ChannelUsage,
        ) -> Result<Self> {
            let (d1, d2) = timeouts_from_chantarget(ct);
            rt.sleep(d1).await;
            if !d2.is_zero() {
                rt.allow_one_advance(d2);
            }

            let c = FakeCirc {
                hops: vec![RelayIds::from_relay_ids(ct)],
                onehop: true,
            };
            Ok(Mutex::new(c))
        }
        async fn create<RT: Runtime>(
            _: &ChanMgr<RT>,
            rt: &RT,
            _guard_status: &GuardStatusHandle,
            ct: &OwnedCircTarget,
            _: &CircParameters,
            _usage: ChannelUsage,
        ) -> Result<Self> {
            let (d1, d2) = timeouts_from_chantarget(ct);
            rt.sleep(d1).await;
            if !d2.is_zero() {
                rt.allow_one_advance(d2);
            }

            let c = FakeCirc {
                hops: vec![RelayIds::from_relay_ids(ct)],
                onehop: false,
            };
            Ok(Mutex::new(c))
        }
        async fn extend<RT: Runtime>(
            &self,
            rt: &RT,
            ct: &OwnedCircTarget,
            _: &CircParameters,
        ) -> Result<()> {
            let (d1, d2) = timeouts_from_chantarget(ct);
            rt.sleep(d1).await;
            if !d2.is_zero() {
                rt.allow_one_advance(d2);
            }

            {
                let mut c = self.lock().unwrap();
                c.hops.push(RelayIds::from_relay_ids(ct));
            }
            Ok(())
        }
    }

    /// Fake implementation of TimeoutEstimator that just records its inputs.
    struct TimeoutRecorder<R> {
        runtime: R,
        hist: Vec<(bool, u8, Duration)>,
        // How much advance to permit after being told of a timeout?
        on_timeout: Duration,
        // How much advance to permit after being told of a success?
        on_success: Duration,

        snd_success: Option<oneshot::Sender<()>>,
        rcv_success: Option<oneshot::Receiver<()>>,
    }

    impl<R> TimeoutRecorder<R> {
        fn new(runtime: R) -> Self {
            Self::with_delays(runtime, Duration::from_secs(0), Duration::from_secs(0))
        }

        fn with_delays(runtime: R, on_timeout: Duration, on_success: Duration) -> Self {
            let (snd_success, rcv_success) = oneshot::channel();
            Self {
                runtime,
                hist: Vec::new(),
                on_timeout,
                on_success,
                rcv_success: Some(rcv_success),
                snd_success: Some(snd_success),
            }
        }
    }
    impl<R: Runtime> TimeoutEstimator for Arc<Mutex<TimeoutRecorder<R>>> {
        fn note_hop_completed(&mut self, hop: u8, delay: Duration, is_last: bool) {
            if !is_last {
                return;
            }
            let (rt, advance) = {
                let mut this = self.lock().unwrap();
                this.hist.push((true, hop, delay));
                let _ = this.snd_success.take().unwrap().send(());
                (this.runtime.clone(), this.on_success)
            };
            if !advance.is_zero() {
                rt.allow_one_advance(advance);
            }
        }
        fn note_circ_timeout(&mut self, hop: u8, delay: Duration) {
            let (rt, advance) = {
                let mut this = self.lock().unwrap();
                this.hist.push((false, hop, delay));
                (this.runtime.clone(), this.on_timeout)
            };
            if !advance.is_zero() {
                rt.allow_one_advance(advance);
            }
        }
        fn timeouts(&mut self, _action: &Action) -> (Duration, Duration) {
            (Duration::from_secs(3), Duration::from_secs(100))
        }
        fn learning_timeouts(&self) -> bool {
            false
        }
        fn update_params(&mut self, _params: &tor_netdir::params::NetParameters) {}

        fn build_state(&mut self) -> Option<crate::timeouts::pareto::ParetoTimeoutState> {
            None
        }
    }

    /// Testing only: create a bogus circuit target
    fn circ_t(id: Ed25519Identity) -> OwnedCircTarget {
        OwnedCircTarget::new(chan_t(id), [0x33; 32].into(), "".parse().unwrap())
    }
    /// Testing only: create a bogus channel target
    fn chan_t(id: Ed25519Identity) -> OwnedChanTarget {
        OwnedChanTarget::new(vec![], id, [0x20; 20].into())
    }

    async fn run_builder_test<R: Runtime>(
        rt: tor_rtmock::MockSleepRuntime<R>,
        advance_initial: Duration,
        path: OwnedPath,
        advance_on_timeout: Option<(Duration, Duration)>,
        usage: ChannelUsage,
    ) -> (Result<FakeCirc>, Vec<(bool, u8, Duration)>) {
        let chanmgr = Arc::new(ChanMgr::new(
            rt.clone(),
            &ChannelConfig::default(),
            Default::default(),
            &Default::default(),
        ));
        // always has 3 second timeout, 100 second abandon.
        let timeouts = match advance_on_timeout {
            Some((d1, d2)) => TimeoutRecorder::with_delays(rt.clone(), d1, d2),
            None => TimeoutRecorder::new(rt.clone()),
        };
        let timeouts = Arc::new(Mutex::new(timeouts));
        let builder: Builder<_, Mutex<FakeCirc>> = Builder::new(
            rt.clone(),
            chanmgr,
            timeouts::Estimator::new(Arc::clone(&timeouts)),
        );

        let params = CircParameters::default();

        rt.block_advance("manually controlling advances");
        rt.allow_one_advance(advance_initial);
        let outcome = rt
            .wait_for(Arc::new(builder).build_owned(path, &params, gs(), usage))
            .await;

        // Now we wait for a success to finally, finally be reported.
        if advance_on_timeout.is_some() {
            let receiver = { timeouts.lock().unwrap().rcv_success.take().unwrap() };
            let _ = rt.wait_for(receiver).await;
        }

        let circ = outcome.map(|m| m.lock().unwrap().clone());
        let timeouts = timeouts.lock().unwrap().hist.clone();

        (circ, timeouts)
    }

    #[test]
    fn build_onehop() {
        test_with_all_runtimes!(|rt| async move {
            let rt = tor_rtmock::MockSleepRuntime::new(rt);
            let id_100ms = key_from_timeouts(Duration::from_millis(100), Duration::from_millis(0));
            let path = OwnedPath::ChannelOnly(chan_t(id_100ms));

            let (outcome, timeouts) =
                run_builder_test(rt, Duration::from_millis(100), path, None, CU::UserTraffic).await;
            let circ = outcome.unwrap();
            assert!(circ.onehop);
            assert_eq!(circ.hops.len(), 1);
            assert!(circ.hops[0].same_relay_ids(&chan_t(id_100ms)));

            assert_eq!(timeouts.len(), 1);
            assert!(timeouts[0].0); // success
            assert_eq!(timeouts[0].1, 0); // one-hop
            assert_eq!(timeouts[0].2, Duration::from_millis(100));
        });
    }

    #[test]
    fn build_threehop() {
        test_with_all_runtimes!(|rt| async move {
            let rt = tor_rtmock::MockSleepRuntime::new(rt);
            let id_100ms =
                key_from_timeouts(Duration::from_millis(100), Duration::from_millis(200));
            let id_200ms =
                key_from_timeouts(Duration::from_millis(200), Duration::from_millis(300));
            let id_300ms = key_from_timeouts(Duration::from_millis(300), Duration::from_millis(0));
            let path =
                OwnedPath::Normal(vec![circ_t(id_100ms), circ_t(id_200ms), circ_t(id_300ms)]);

            let (outcome, timeouts) =
                run_builder_test(rt, Duration::from_millis(100), path, None, CU::UserTraffic).await;
            let circ = outcome.unwrap();
            assert!(!circ.onehop);
            assert_eq!(circ.hops.len(), 3);
            assert!(circ.hops[0].same_relay_ids(&chan_t(id_100ms)));
            assert!(circ.hops[1].same_relay_ids(&chan_t(id_200ms)));
            assert!(circ.hops[2].same_relay_ids(&chan_t(id_300ms)));

            assert_eq!(timeouts.len(), 1);
            assert!(timeouts[0].0); // success
            assert_eq!(timeouts[0].1, 2); // three-hop
            assert_eq!(timeouts[0].2, Duration::from_millis(600));
        });
    }

    #[test]
    fn build_huge_timeout() {
        test_with_all_runtimes!(|rt| async move {
            let rt = tor_rtmock::MockSleepRuntime::new(rt);
            let id_100ms =
                key_from_timeouts(Duration::from_millis(100), Duration::from_millis(200));
            let id_200ms =
                key_from_timeouts(Duration::from_millis(200), Duration::from_millis(2700));
            let id_hour = key_from_timeouts(Duration::from_secs(3600), Duration::from_secs(0));

            let path = OwnedPath::Normal(vec![circ_t(id_100ms), circ_t(id_200ms), circ_t(id_hour)]);

            let (outcome, timeouts) =
                run_builder_test(rt, Duration::from_millis(100), path, None, CU::UserTraffic).await;
            assert!(matches!(outcome, Err(Error::CircTimeout)));

            assert_eq!(timeouts.len(), 1);
            assert!(!timeouts[0].0); // timeout

            // BUG: Sometimes this is 1 and sometimes this is 2.
            // assert_eq!(timeouts[0].1, 2); // at third hop.
            assert_eq!(timeouts[0].2, Duration::from_millis(3000));
        });
    }

    #[test]
    fn build_modest_timeout() {
        test_with_all_runtimes!(|rt| async move {
            let rt = tor_rtmock::MockSleepRuntime::new(rt);
            let id_100ms =
                key_from_timeouts(Duration::from_millis(100), Duration::from_millis(200));
            let id_200ms =
                key_from_timeouts(Duration::from_millis(200), Duration::from_millis(2700));
            let id_3sec = key_from_timeouts(Duration::from_millis(3000), Duration::from_millis(0));

            let timeout_advance = (Duration::from_millis(4000), Duration::from_secs(0));

            let path = OwnedPath::Normal(vec![circ_t(id_100ms), circ_t(id_200ms), circ_t(id_3sec)]);

            let (outcome, timeouts) = run_builder_test(
                rt.clone(),
                Duration::from_millis(100),
                path,
                Some(timeout_advance),
                CU::UserTraffic,
            )
            .await;
            assert!(matches!(outcome, Err(Error::CircTimeout)));

            assert_eq!(timeouts.len(), 2);
            assert!(!timeouts[0].0); // timeout

            // BUG: Sometimes this is 1 and sometimes this is 2.
            //assert_eq!(timeouts[0].1, 2); // at third hop.
            assert_eq!(timeouts[0].2, Duration::from_millis(3000));

            assert!(timeouts[1].0); // success
            assert_eq!(timeouts[1].1, 2); // three-hop
                                          // BUG: This timer is not always reliable, due to races.
                                          //assert_eq!(timeouts[1].2, Duration::from_millis(3300));
        });
    }
}
