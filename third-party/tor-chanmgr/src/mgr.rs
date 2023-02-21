//! Abstract implementation of a channel manager

use crate::mgr::state::{OpenEntry, PendingEntry};
use crate::{ChanProvenance, ChannelConfig, ChannelUsage, Dormancy, Error, Result};

use crate::factory::BootstrapReporter;
use async_trait::async_trait;
use futures::channel::oneshot;
use futures::future::{FutureExt, Shared};
use rand::Rng;
use std::result::Result as StdResult;
use std::sync::Arc;
use std::time::Duration;
use tor_error::internal;
use tor_linkspec::{HasRelayIds, RelayIds};
use tor_netdir::params::NetParameters;
use tor_proto::channel::params::ChannelPaddingInstructionsUpdates;

mod state;

/// Trait to describe as much of a
/// [`Channel`](tor_proto::channel::Channel) as `AbstractChanMgr`
/// needs to use.
pub(crate) trait AbstractChannel: Clone + HasRelayIds {
    /// Return true if this channel is usable.
    ///
    /// A channel might be unusable because it is closed, because it has
    /// hit a bug, or for some other reason.  We don't return unusable
    /// channels back to the user.
    fn is_usable(&self) -> bool;
    /// Return the amount of time a channel has not been in use.
    /// Return None if the channel is currently in use.
    fn duration_unused(&self) -> Option<Duration>;

    /// Reparameterize this channel according to the provided `ChannelPaddingInstructionsUpdates`
    ///
    /// The changed parameters may not be implemented "immediately",
    /// but this will be done "reasonably soon".
    fn reparameterize(
        &self,
        updates: Arc<ChannelPaddingInstructionsUpdates>,
    ) -> tor_proto::Result<()>;

    /// Specify that this channel should do activities related to channel padding
    ///
    /// See [`Channel::engage_padding_activities`]
    ///
    /// [`Channel::engage_padding_activities`]: tor_proto::channel::Channel::engage_padding_activities
    fn engage_padding_activities(&self);
}

/// Trait to describe how channels-like objects are created.
///
/// This differs from [`ChannelFactory`](crate::factory::ChannelFactory) in that
/// it's a purely crate-internal type that we use to decouple the
/// AbstractChanMgr code from actual "what is a channel" concerns.
#[async_trait]
pub(crate) trait AbstractChannelFactory {
    /// The type of channel that this factory can build.
    type Channel: AbstractChannel;
    /// Type that explains how to build a channel.
    type BuildSpec: HasRelayIds;

    /// Construct a new channel to the destination described at `target`.
    ///
    /// This function must take care of all timeouts, error detection,
    /// and so on.
    ///
    /// It should not retry; that is handled at a higher level.
    async fn build_channel(
        &self,
        target: &Self::BuildSpec,
        reporter: BootstrapReporter,
    ) -> Result<Self::Channel>;
}

/// A type- and network-agnostic implementation for [`ChanMgr`](crate::ChanMgr).
///
/// This type does the work of keeping track of open channels and pending
/// channel requests, launching requests as needed, waiting for pending
/// requests, and so forth.
///
/// The actual job of launching connections is deferred to an
/// `AbstractChannelFactory` type.
pub(crate) struct AbstractChanMgr<CF: AbstractChannelFactory> {
    /// All internal state held by this channel manager.
    ///
    /// The most important part is the map from relay identity to channel, or
    /// to pending channel status.
    pub(crate) channels: state::MgrState<CF>,

    /// A bootstrap reporter to give out when building channels.
    pub(crate) reporter: BootstrapReporter,
}

/// Type alias for a future that we wait on to see when a pending
/// channel is done or failed.
type Pending = Shared<oneshot::Receiver<Result<()>>>;

/// Type alias for the sender we notify when we complete a channel (or fail to
/// complete it).
type Sending = oneshot::Sender<Result<()>>;

impl<CF: AbstractChannelFactory + Clone> AbstractChanMgr<CF> {
    /// Make a new empty channel manager.
    pub(crate) fn new(
        connector: CF,
        config: &ChannelConfig,
        dormancy: Dormancy,
        netparams: &NetParameters,
        reporter: BootstrapReporter,
    ) -> Self {
        AbstractChanMgr {
            channels: state::MgrState::new(connector, config.clone(), dormancy, netparams),
            reporter,
        }
    }

    /// Run a function to modify the channel builder in this object.
    #[allow(dead_code)]
    pub(crate) fn with_mut_builder<F>(&self, func: F)
    where
        F: FnOnce(&mut CF),
    {
        self.channels.with_mut_builder(func);
    }

    /// Remove every unusable entry from this channel manager.
    #[cfg(test)]
    pub(crate) fn remove_unusable_entries(&self) -> Result<()> {
        self.channels.remove_unusable()
    }

    /// Helper: return the objects used to inform pending tasks
    /// about a newly open or failed channel.
    fn setup_launch<C: Clone>(&self, ids: RelayIds) -> (state::ChannelState<C>, Sending) {
        let (snd, rcv) = oneshot::channel();
        let pending = rcv.shared();
        (
            state::ChannelState::Building(state::PendingEntry { ids, pending }),
            snd,
        )
    }

    /// Get a channel corresponding to the identities of `target`.
    ///
    /// If a usable channel exists with that identity, return it.
    ///
    /// If no such channel exists already, and none is in progress,
    /// launch a new request using `target`.
    ///
    /// If no such channel exists already, but we have one that's in
    /// progress, wait for it to succeed or fail.
    pub(crate) async fn get_or_launch(
        &self,
        target: CF::BuildSpec,
        usage: ChannelUsage,
    ) -> Result<(CF::Channel, ChanProvenance)> {
        use ChannelUsage as CU;

        let chan = self.get_or_launch_internal(target).await?;

        match usage {
            CU::Dir | CU::UselessCircuit => {}
            CU::UserTraffic => chan.0.engage_padding_activities(),
        }

        Ok(chan)
    }

    /// Get a channel whose identity is `ident` - internal implementation
    async fn get_or_launch_internal(
        &self,
        target: CF::BuildSpec,
    ) -> Result<(CF::Channel, ChanProvenance)> {
        /// How many times do we try?
        const N_ATTEMPTS: usize = 2;
        let mut attempts_so_far = 0;
        let mut final_attempt = false;
        let mut provenance = ChanProvenance::Preexisting;

        // TODO(nickm): It would be neat to use tor_retry instead.
        let mut last_err = None;

        while attempts_so_far < N_ATTEMPTS || final_attempt {
            attempts_so_far += 1;

            // For each attempt, we _first_ look at the state of the channel map
            // to decide on an `Action`, and _then_ we execute that action.

            // First, see what state we're in, and what we should do about it.
            let action = self.choose_action(&target, final_attempt)?;

            // We are done deciding on our Action! It's time act based on the
            // Action that we chose.
            match action {
                // If this happens, we were trying to make one final check of our state, but
                // we would have had to make additional attempts.
                None => {
                    if !final_attempt {
                        return Err(Error::Internal(internal!(
                            "No action returned while not on final attempt"
                        )));
                    }
                    break;
                }
                // Easy case: we have an error or a channel to return.
                Some(Action::Return(v)) => {
                    return v.map(|chan| (chan, provenance));
                }
                // There's an in-progress channel.  Wait for it.
                Some(Action::Wait(pend)) => {
                    match pend.await {
                        Ok(Ok(())) => {
                            // We were waiting for a channel, and it succeeded, or it
                            // got cancelled.  But it might have gotten more
                            // identities while negotiating than it had when it was
                            // launched, or it might have failed to get all the
                            // identities we want. Check for this.
                            final_attempt = true;
                            provenance = ChanProvenance::NewlyCreated;
                            last_err.get_or_insert(Error::RequestCancelled);
                        }
                        Ok(Err(e)) => {
                            last_err = Some(e);
                        }
                        Err(_) => {
                            last_err =
                                Some(Error::Internal(internal!("channel build task disappeared")));
                        }
                    }
                }
                // We need to launch a channel.
                Some(Action::Launch(send)) => {
                    let connector = self.channels.builder();
                    let outcome = connector
                        .build_channel(&target, self.reporter.clone())
                        .await;
                    let status = self.handle_build_outcome(&target, outcome);

                    // It's okay if all the receivers went away:
                    // that means that nobody was waiting for this channel.
                    let _ignore_err = send.send(status.clone().map(|_| ()));

                    match status {
                        Ok(Some(chan)) => {
                            return Ok((chan, ChanProvenance::NewlyCreated));
                        }
                        Ok(None) => {
                            final_attempt = true;
                            provenance = ChanProvenance::NewlyCreated;
                            last_err.get_or_insert(Error::RequestCancelled);
                        }
                        Err(e) => last_err = Some(e),
                    }
                }
            }

            // End of this attempt. We will try again...
        }

        Err(last_err.unwrap_or_else(|| Error::Internal(internal!("no error was set!?"))))
    }

    /// Helper: based on our internal state, decide which action to take when
    /// asked for a channel, and update our internal state accordingly.
    ///
    /// If `final_attempt` is true, then we will not pick any action that does
    /// not result in an immediate result. If we would pick such an action, we
    /// instead return `Ok(None)`.  (We could instead have the caller detect
    /// such actions, but it's less efficient to construct them, insert them,
    /// and immediately revert them.)
    fn choose_action(
        &self,
        target: &CF::BuildSpec,
        final_attempt: bool,
    ) -> Result<Option<Action<CF::Channel>>> {
        use state::ChannelState::*;
        self.channels.with_channels(|channel_map| {
            match channel_map.by_all_ids(target) {
                Some(Open(OpenEntry { channel, .. })) => {
                    if channel.is_usable() {
                        // This entry is a perfect match for the target keys:
                        // we'll return the open entry.
                        return Ok(Some(Action::Return(Ok(channel.clone()))));
                    } else if final_attempt {
                        // We don't launch an attempt in this case.
                        return Ok(None);
                    } else {
                        // This entry was a perfect match for the target, but it
                        // is no longer usable! We launch a new connection to
                        // this target, and wait on that.
                        let (new_state, send) = self.setup_launch(RelayIds::from_relay_ids(target));
                        channel_map.try_insert(new_state)?;

                        return Ok(Some(Action::Launch(send)));
                    }
                }
                Some(Building(PendingEntry { pending, .. })) => {
                    // This entry is a perfect match for the target keys: we'll
                    // return the pending entry. (We don't know for sure if it
                    // will match once it completes, since we might discover
                    // additional keys beyond those listed for this pending
                    // entry.)
                    if final_attempt {
                        // We don't launch an attempt in this case.
                        return Ok(None);
                    }
                    return Ok(Some(Action::Wait(pending.clone())));
                }
                _ => {}
            }
            // Okay, we don't have an exact match.  But we might have one or more _partial_ matches?
            let overlapping = channel_map.all_overlapping(target);
            if overlapping
                .iter()
                .any(|entry| matches!(entry, Open(OpenEntry{ channel, ..}) if channel.is_usable()))
            {
                // At least one *open, usable* channel has been negotiated that
                // overlaps only partially with our target: it has proven itself
                // to have _one_ of our target identities, but not all.
                //
                // Because this channel exists, we know that our target cannot
                // succeed, since relays are not allowed to share _any_
                // identities.
                return Ok(Some(Action::Return(Err(Error::IdentityConflict))));
            } else if let Some(first_building) = overlapping
                .iter()
                .find(|entry| matches!(entry, Building(_)))
            {
                // There is at least one *in-progress* channel that has at least
                // one identity in common with our target, but it does not have
                // _all_ the identities we want.
                //
                // If it succeeds, we might find that we can use it; or we might
                // find out that it is not suitable.  So we'll wait for it, and
                // see what happens.
                //
                // TODO: This approach will _not_ be sufficient once  we are
                // implementing relays, and some of our channels are created in
                // response to client requests.
                match first_building {
                    Open(_) => unreachable!(),
                    Building(PendingEntry { pending, .. }) => {
                        if final_attempt {
                            // We don't wait in this case.
                            return Ok(None);
                        }
                        return Ok(Some(Action::Wait(pending.clone())));
                    }
                }
            }

            if final_attempt {
                // We don't launch an attempt in this case.
                return Ok(None);
            }

            // Great, nothing interfered at all.
            let (new_state, send) = self.setup_launch(RelayIds::from_relay_ids(target));
            channel_map.try_insert(new_state)?;
            Ok(Some(Action::Launch(send)))
        })?
    }

    /// We just tried to build a channel: Handle the outcome and decide what to
    /// do.
    ///
    /// Return `Ok(None)` if we have a transient error that we expect will be
    /// cleaned up by one final call to `choose_action`.
    fn handle_build_outcome(
        &self,
        target: &CF::BuildSpec,
        outcome: Result<CF::Channel>,
    ) -> Result<Option<CF::Channel>> {
        use state::ChannelState::*;
        match outcome {
            Ok(chan) => {
                // The channel got built: remember it, tell the
                // others, and return it.
                self.channels
                    .with_channels_and_params(|channel_map, channels_params| {
                        match channel_map.remove_exact(target) {
                            Some(Building(_)) => {
                                // We successfully removed our pending
                                // action. great!  Fall through and add
                                // the channel we just built.
                            }
                            None => {
                                // Something removed our entry from the list.
                                // Time to retry.
                                return Ok(None);
                            }
                            Some(ent @ Open(_)) => {
                                // Oh no. Something else built an entry
                                // here, and replaced us.  Put that
                                // something back, and retry.

                                channel_map.insert(ent);
                                return Ok(None);
                            }
                        }

                        // This isn't great.  We context switch to the newly-created
                        // channel just to tell it how and whether to do padding.  Ideally
                        // we would pass the params at some suitable point during
                        // building.  However, that would involve the channel taking a
                        // copy of the params, and that must happen in the same channel
                        // manager lock acquisition span as the one where we insert the
                        // channel into the table so it will receive updates.  I.e.,
                        // here.
                        let update = channels_params.initial_update();
                        if let Some(update) = update {
                            chan.reparameterize(update.into())
                                .map_err(|_| internal!("failure on new channel"))?;
                        }
                        let new_entry = Open(OpenEntry {
                            channel: chan.clone(),
                            max_unused_duration: Duration::from_secs(
                                rand::thread_rng().gen_range(180..270),
                            ),
                        });
                        channel_map.insert(new_entry);
                        Ok(Some(chan))
                    })?
            }
            Err(e) => {
                // The channel failed. Make it non-pending, tell the
                // others, and set the error.
                self.channels.with_channels(|channel_map| {
                    match channel_map.remove_exact(target) {
                        Some(Building(_)) | None => {
                            // We successfully removed our pending
                            // action, or somebody else did.
                        }
                        Some(ent @ Open(_)) => {
                            // Oh no. Something else built an entry
                            // here, and replaced us.  Put that
                            // something back.
                            channel_map.insert(ent);
                        }
                    }
                })?;
                Err(e)
            }
        }
    }

    /// Update the netdir
    pub(crate) fn update_netparams(
        &self,
        netparams: Arc<dyn AsRef<NetParameters>>,
    ) -> StdResult<(), tor_error::Bug> {
        self.channels.reconfigure_general(None, None, netparams)
    }

    /// Notifies the chanmgr to be dormant like dormancy
    pub(crate) fn set_dormancy(
        &self,
        dormancy: Dormancy,
        netparams: Arc<dyn AsRef<NetParameters>>,
    ) -> StdResult<(), tor_error::Bug> {
        self.channels
            .reconfigure_general(None, Some(dormancy), netparams)
    }

    /// Reconfigure all channels
    pub(crate) fn reconfigure(
        &self,
        config: &ChannelConfig,
        netparams: Arc<dyn AsRef<NetParameters>>,
    ) -> StdResult<(), tor_error::Bug> {
        self.channels
            .reconfigure_general(Some(config), None, netparams)
    }

    /// Expire any channels that have been unused longer than
    /// their maximum unused duration assigned during creation.
    ///
    /// Return a duration from now until next channel expires.
    ///
    /// If all channels are in use or there are no open channels,
    /// return 180 seconds which is the minimum value of
    /// max_unused_duration.
    pub(crate) fn expire_channels(&self) -> Duration {
        self.channels.expire_channels()
    }

    /// Test only: return the current open usable channel with a given
    /// `ident`, if any.
    #[cfg(test)]
    pub(crate) fn get_nowait<'a, T>(&self, ident: T) -> Option<CF::Channel>
    where
        T: Into<tor_linkspec::RelayIdRef<'a>>,
    {
        use state::ChannelState::*;
        self.channels
            .with_channels(|channel_map| match channel_map.by_id(ident) {
                Some(Open(ref ent)) if ent.channel.is_usable() => Some(ent.channel.clone()),
                _ => None,
            })
            .expect("Poisoned lock")
    }
}

/// Possible actions that we'll decide to take when asked for a channel.
#[allow(clippy::large_enum_variant)]
enum Action<C> {
    /// We found no channel.  We're going to launch a new one,
    /// then tell everybody about it.
    Launch(Sending),
    /// We found an in-progress attempt at making a channel.
    /// We're going to wait for it to finish.
    Wait(Pending),
    /// We found a usable channel.  We're going to return it.
    Return(Result<C>),
}

#[cfg(test)]
mod test {
    // @@ begin test lint list maintained by maint/add_warning @@
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::clone_on_copy)]
    #![allow(clippy::dbg_macro)]
    #![allow(clippy::print_stderr)]
    #![allow(clippy::print_stdout)]
    #![allow(clippy::single_char_pattern)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::unchecked_duration_subtraction)]
    //! <!-- @@ end test lint list maintained by maint/add_warning @@ -->
    use super::*;
    use crate::Error;

    use futures::join;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tor_error::bad_api_usage;
    use tor_llcrypto::pk::ed25519::Ed25519Identity;

    use crate::ChannelUsage as CU;
    use tor_rtcompat::{task::yield_now, test_with_one_runtime, Runtime};

    #[derive(Clone)]
    struct FakeChannelFactory<RT> {
        runtime: RT,
    }

    #[derive(Clone, Debug)]
    struct FakeChannel {
        ed_ident: Ed25519Identity,
        mood: char,
        closing: Arc<AtomicBool>,
        detect_reuse: Arc<char>,
        // last_params: Option<ChannelPaddingInstructionsUpdates>,
    }

    impl PartialEq for FakeChannel {
        fn eq(&self, other: &Self) -> bool {
            Arc::ptr_eq(&self.detect_reuse, &other.detect_reuse)
        }
    }

    impl AbstractChannel for FakeChannel {
        fn is_usable(&self) -> bool {
            !self.closing.load(Ordering::SeqCst)
        }
        fn duration_unused(&self) -> Option<Duration> {
            None
        }
        fn reparameterize(
            &self,
            _updates: Arc<ChannelPaddingInstructionsUpdates>,
        ) -> tor_proto::Result<()> {
            // *self.last_params.lock().unwrap() = Some((*updates).clone());
            Ok(())
        }
        fn engage_padding_activities(&self) {}
    }

    impl HasRelayIds for FakeChannel {
        fn identity(
            &self,
            key_type: tor_linkspec::RelayIdType,
        ) -> Option<tor_linkspec::RelayIdRef<'_>> {
            match key_type {
                tor_linkspec::RelayIdType::Ed25519 => Some((&self.ed_ident).into()),
                _ => None,
            }
        }
    }

    impl FakeChannel {
        fn start_closing(&self) {
            self.closing.store(true, Ordering::SeqCst);
        }
    }

    impl<RT: Runtime> FakeChannelFactory<RT> {
        fn new(runtime: RT) -> Self {
            FakeChannelFactory { runtime }
        }
    }

    fn new_test_abstract_chanmgr<R: Runtime>(runtime: R) -> AbstractChanMgr<FakeChannelFactory<R>> {
        let cf = FakeChannelFactory::new(runtime);
        AbstractChanMgr::new(
            cf,
            &ChannelConfig::default(),
            Default::default(),
            &Default::default(),
            BootstrapReporter::fake(),
        )
    }

    #[derive(Clone, Debug)]
    struct FakeBuildSpec(u32, char, Ed25519Identity);

    impl HasRelayIds for FakeBuildSpec {
        fn identity(
            &self,
            key_type: tor_linkspec::RelayIdType,
        ) -> Option<tor_linkspec::RelayIdRef<'_>> {
            match key_type {
                tor_linkspec::RelayIdType::Ed25519 => Some((&self.2).into()),
                _ => None,
            }
        }
    }

    /// Helper to make a fake Ed identity from a u32.
    fn u32_to_ed(n: u32) -> Ed25519Identity {
        let mut bytes = [0; 32];
        bytes[0..4].copy_from_slice(&n.to_be_bytes());
        bytes.into()
    }

    #[async_trait]
    impl<RT: Runtime> AbstractChannelFactory for FakeChannelFactory<RT> {
        type Channel = FakeChannel;
        type BuildSpec = FakeBuildSpec;

        async fn build_channel(
            &self,
            target: &Self::BuildSpec,
            _reporter: BootstrapReporter,
        ) -> Result<FakeChannel> {
            yield_now().await;
            let FakeBuildSpec(ident, mood, id) = *target;
            let ed_ident = u32_to_ed(ident);
            assert_eq!(ed_ident, id);
            match mood {
                // "X" means never connect.
                '❌' | '🔥' => return Err(Error::UnusableTarget(bad_api_usage!("emoji"))),
                // "zzz" means wait for 15 seconds then succeed.
                '💤' => {
                    self.runtime.sleep(Duration::new(15, 0)).await;
                }
                _ => {}
            }
            Ok(FakeChannel {
                ed_ident,
                mood,
                closing: Arc::new(AtomicBool::new(false)),
                detect_reuse: Default::default(),
                // last_params: None,
            })
        }
    }

    #[test]
    fn connect_one_ok() {
        test_with_one_runtime!(|runtime| async {
            let mgr = new_test_abstract_chanmgr(runtime);
            let target = FakeBuildSpec(413, '!', u32_to_ed(413));
            let chan1 = mgr
                .get_or_launch(target.clone(), CU::UserTraffic)
                .await
                .unwrap()
                .0;
            let chan2 = mgr.get_or_launch(target, CU::UserTraffic).await.unwrap().0;

            assert_eq!(chan1, chan2);

            let chan3 = mgr.get_nowait(&u32_to_ed(413)).unwrap();
            assert_eq!(chan1, chan3);
        });
    }

    #[test]
    fn connect_one_fail() {
        test_with_one_runtime!(|runtime| async {
            let mgr = new_test_abstract_chanmgr(runtime);

            // This is set up to always fail.
            let target = FakeBuildSpec(999, '❌', u32_to_ed(999));
            let res1 = mgr.get_or_launch(target, CU::UserTraffic).await;
            assert!(matches!(res1, Err(Error::UnusableTarget(_))));

            let chan3 = mgr.get_nowait(&u32_to_ed(999));
            assert!(chan3.is_none());
        });
    }

    #[test]
    fn test_concurrent() {
        test_with_one_runtime!(|runtime| async {
            let mgr = new_test_abstract_chanmgr(runtime);

            // TODO(nickm): figure out how to make these actually run
            // concurrently. Right now it seems that they don't actually
            // interact.
            let (ch3a, ch3b, ch44a, ch44b, ch86a, ch86b) = join!(
                mgr.get_or_launch(FakeBuildSpec(3, 'a', u32_to_ed(3)), CU::UserTraffic),
                mgr.get_or_launch(FakeBuildSpec(3, 'b', u32_to_ed(3)), CU::UserTraffic),
                mgr.get_or_launch(FakeBuildSpec(44, 'a', u32_to_ed(44)), CU::UserTraffic),
                mgr.get_or_launch(FakeBuildSpec(44, 'b', u32_to_ed(44)), CU::UserTraffic),
                mgr.get_or_launch(FakeBuildSpec(86, '❌', u32_to_ed(86)), CU::UserTraffic),
                mgr.get_or_launch(FakeBuildSpec(86, '🔥', u32_to_ed(86)), CU::UserTraffic),
            );
            let ch3a = ch3a.unwrap();
            let ch3b = ch3b.unwrap();
            let ch44a = ch44a.unwrap();
            let ch44b = ch44b.unwrap();
            let err_a = ch86a.unwrap_err();
            let err_b = ch86b.unwrap_err();

            assert_eq!(ch3a, ch3b);
            assert_eq!(ch44a, ch44b);
            assert_ne!(ch44a, ch3a);

            assert!(matches!(err_a, Error::UnusableTarget(_)));
            assert!(matches!(err_b, Error::UnusableTarget(_)));
        });
    }

    #[test]
    fn unusable_entries() {
        test_with_one_runtime!(|runtime| async {
            let mgr = new_test_abstract_chanmgr(runtime);

            let (ch3, ch4, ch5) = join!(
                mgr.get_or_launch(FakeBuildSpec(3, 'a', u32_to_ed(3)), CU::UserTraffic),
                mgr.get_or_launch(FakeBuildSpec(4, 'a', u32_to_ed(4)), CU::UserTraffic),
                mgr.get_or_launch(FakeBuildSpec(5, 'a', u32_to_ed(5)), CU::UserTraffic),
            );

            let ch3 = ch3.unwrap().0;
            let _ch4 = ch4.unwrap();
            let ch5 = ch5.unwrap().0;

            ch3.start_closing();
            ch5.start_closing();

            let ch3_new = mgr
                .get_or_launch(FakeBuildSpec(3, 'b', u32_to_ed(3)), CU::UserTraffic)
                .await
                .unwrap()
                .0;
            assert_ne!(ch3, ch3_new);
            assert_eq!(ch3_new.mood, 'b');

            mgr.remove_unusable_entries().unwrap();

            assert!(mgr.get_nowait(&u32_to_ed(3)).is_some());
            assert!(mgr.get_nowait(&u32_to_ed(4)).is_some());
            assert!(mgr.get_nowait(&u32_to_ed(5)).is_none());
        });
    }
}
