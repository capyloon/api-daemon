//! A general interface for Tor client usage.
//!
//! To construct a client, run the [`TorClient::create_bootstrapped`] method.
//! Once the client is bootstrapped, you can make anonymous
//! connections ("streams") over the Tor network using
//! [`TorClient::connect`].
use crate::address::IntoTorAddr;

use crate::config::{ClientAddrConfig, StreamTimeoutConfig, TorClientConfig};
use tor_circmgr::{DirInfo, IsolationToken, StreamIsolationBuilder, TargetPort};
use tor_config::MutCfg;
use tor_dirmgr::DirEvent;
use tor_persist::{FsStateMgr, StateMgr};
use tor_proto::circuit::ClientCirc;
use tor_proto::stream::{DataStream, IpVersionPreference, StreamParameters};
use tor_rtcompat::{PreferredRuntime, Runtime, SleepProviderExt};

use futures::lock::Mutex as AsyncMutex;
use futures::stream::StreamExt;
use futures::task::SpawnExt;
use std::convert::TryInto;
use std::net::IpAddr;
use std::result::Result as StdResult;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use crate::err::ErrorDetail;
use crate::{status, util, TorClientBuilder};
use tracing::{debug, error, info, warn};

/// An active client session on the Tor network.
///
/// While it's running, it will fetch directory information, build
/// circuits, and make connections for you.
///
/// Cloning this object makes a new reference to the same underlying
/// handles: it's usually better to clone the `TorClient` than it is to
/// create a new one.
// TODO(nickm): This type now has 5 Arcs inside it, and 2 types that have
// implicit Arcs inside them! maybe it's time to replace much of the insides of
// this with an Arc<TorClientInner>?
#[derive(Clone)]
pub struct TorClient<R: Runtime> {
    /// Asynchronous runtime object.
    runtime: R,
    /// Default isolation token for streams through this client.
    ///
    /// This is eventually used for `owner_token` in `tor-circmgr/src/usage.rs`, and is orthogonal
    /// to the `stream_token` which comes from `connect_prefs` (or a passed-in `StreamPrefs`).
    /// (ie, both must be the same to share a circuit).
    client_isolation: IsolationToken,
    /// Connection preferences.  Starts out as `Default`,  Inherited by our clones.
    connect_prefs: StreamPrefs,
    /// Circuit manager for keeping our circuits up to date and building
    /// them on-demand.
    circmgr: Arc<tor_circmgr::CircMgr<R>>,
    /// Directory manager for keeping our directory material up to date.
    dirmgr: Arc<dyn tor_dirmgr::DirProvider + Send + Sync>,
    /// Location on disk where we store persistent data.
    statemgr: FsStateMgr,
    /// Client address configuration
    addrcfg: Arc<MutCfg<ClientAddrConfig>>,
    /// Client DNS configuration
    timeoutcfg: Arc<MutCfg<StreamTimeoutConfig>>,
    /// Mutex used to serialize concurrent attempts to reconfigure a TorClient.
    ///
    /// See [`TorClient::reconfigure`] for more information on its use.
    reconfigure_lock: Arc<Mutex<()>>,

    /// A stream of bootstrap messages that we can clone when a client asks for
    /// it.
    ///
    /// (We don't need to observe this stream ourselves, since it drops each
    /// unobserved status change when the next status change occurs.)
    status_receiver: status::BootstrapEvents,

    /// mutex used to prevent two tasks from trying to bootstrap at once.
    bootstrap_in_progress: Arc<AsyncMutex<()>>,

    /// Whether or not we should call `bootstrap` before doing things that require
    /// bootstrapping. If this is `false`, we will just call `wait_for_bootstrap`
    /// instead.
    should_bootstrap: BootstrapBehavior,
}

/// Preferences for whether a [`TorClient`] should bootstrap on its own or not.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BootstrapBehavior {
    /// Bootstrap the client automatically when requests are made that require the client to be
    /// bootstrapped.
    OnDemand,
    /// Make no attempts to automatically bootstrap. [`TorClient::bootstrap`] must be manually
    /// invoked in order for the [`TorClient`] to become useful.
    ///
    /// Attempts to use the client (e.g. by creating connections or resolving hosts over the Tor
    /// network) before calling [`bootstrap`](TorClient::bootstrap) will fail, and
    /// return an error that has kind [`ErrorKind::BootstrapRequired`](crate::ErrorKind::BootstrapRequired).
    Manual,
}

impl Default for BootstrapBehavior {
    fn default() -> Self {
        BootstrapBehavior::OnDemand
    }
}

/// Preferences for how to route a stream over the Tor network.
#[derive(Debug, Clone, Default)]
pub struct StreamPrefs {
    /// What kind of IPv6/IPv4 we'd prefer, and how strongly.
    ip_ver_pref: IpVersionPreference,
    /// How should we isolate connection(s) ?
    isolation: StreamIsolationPreference,
    /// Whether to return the stream optimistically.
    optimistic_stream: bool,
}

/// Record of how we are isolating connections
#[derive(Debug, Clone)]
enum StreamIsolationPreference {
    /// No additional isolation
    None,
    /// Id of the isolation group the connection should be part of
    Explicit(IsolationToken),
    /// Isolate every connection!
    EveryStream,
}

impl Default for StreamIsolationPreference {
    fn default() -> Self {
        StreamIsolationPreference::None
    }
}

impl StreamPrefs {
    /// Construct a new StreamPrefs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Indicate that a stream may be made over IPv4 or IPv6, but that
    /// we'd prefer IPv6.
    pub fn ipv6_preferred(&mut self) -> &mut Self {
        self.ip_ver_pref = IpVersionPreference::Ipv6Preferred;
        self
    }

    /// Indicate that a stream may only be made over IPv6.
    ///
    /// When this option is set, we will only pick exit relays that
    /// support IPv6, and we will tell them to only give us IPv6
    /// connections.
    pub fn ipv6_only(&mut self) -> &mut Self {
        self.ip_ver_pref = IpVersionPreference::Ipv6Only;
        self
    }

    /// Indicate that a stream may be made over IPv4 or IPv6, but that
    /// we'd prefer IPv4.
    ///
    /// This is the default.
    pub fn ipv4_preferred(&mut self) -> &mut Self {
        self.ip_ver_pref = IpVersionPreference::Ipv4Preferred;
        self
    }

    /// Indicate that a stream may only be made over IPv4.
    ///
    /// When this option is set, we will only pick exit relays that
    /// support IPv4, and we will tell them to only give us IPv4
    /// connections.
    pub fn ipv4_only(&mut self) -> &mut Self {
        self.ip_ver_pref = IpVersionPreference::Ipv4Only;
        self
    }

    /// Indicate that the stream should be opened "optimistically".
    ///
    /// By default, streams are not "optimistic". When you call
    /// [`TorClient::connect()`], it won't give you a stream until the
    /// exit node has confirmed that it has successfully opened a
    /// connection to your target address.  It's safer to wait in this
    /// way, but it is slower: it takes an entire round trip to get
    /// your confirmation.
    ///
    /// If a stream _is_ configured to be "optimistic", on the other
    /// hand, then `TorClient::connect()` will return the stream
    /// immediately, without waiting for an answer from the exit.  You
    /// can start sending data on the stream right away, though of
    /// course this data will be lost if the connection is not
    /// actually successful.
    pub fn optimistic(&mut self) -> &mut Self {
        self.optimistic_stream = true;
        self
    }

    /// Return a TargetPort to describe what kind of exit policy our
    /// target circuit needs to support.
    fn wrap_target_port(&self, port: u16) -> TargetPort {
        match self.ip_ver_pref {
            IpVersionPreference::Ipv6Only => TargetPort::ipv6(port),
            _ => TargetPort::ipv4(port),
        }
    }

    /// Return a new StreamParameters based on this configuration.
    fn stream_parameters(&self) -> StreamParameters {
        let mut params = StreamParameters::default();
        params
            .ip_version(self.ip_ver_pref)
            .optimistic(self.optimistic_stream);
        params
    }

    /// Indicate which other connections might use the same circuit
    /// as this one.
    ///
    /// By default all connections made on all clones of a `TorClient` may share connections.
    /// Connections made with a particular `isolation_group` may share circuits with each other.
    ///
    /// This connection preference is orthogonal to isolation established by
    /// [`TorClient::isolated_client`].  Connections made with an `isolated_client` (and its
    /// clones) will not share circuits with the original client, even if the same
    /// `isolation_group` is specified via the `ConnectionPrefs` in force.
    pub fn set_isolation_group(&mut self, isolation_group: IsolationToken) -> &mut Self {
        self.isolation = StreamIsolationPreference::Explicit(isolation_group);
        self
    }

    /// Indicate that connections with these preferences should have their own isolation group
    ///
    /// This is a convenience method which creates a fresh [`IsolationToken`]
    /// and sets it for these preferences.
    ///
    /// This connection preference is orthogonal to isolation established by
    /// [`TorClient::isolated_client`].  Connections made with an `isolated_client` (and its
    /// clones) will not share circuits with the original client, even if the same
    /// `isolation_group` is specified via the `ConnectionPrefs` in force.
    pub fn new_isolation_group(&mut self) -> &mut Self {
        self.isolation = StreamIsolationPreference::Explicit(IsolationToken::new());
        self
    }

    /// Indicate that no connection should share a circuit with any other.
    ///
    /// **Use with care:** This is likely to have poor performance, and imposes a much greater load
    /// on the Tor network.  Use this option only to make small numbers of connections each of
    /// which needs to be isolated from all other connections.
    ///
    /// (Don't just use this as a "get more privacy!!" method: the circuits
    /// that it put connections on will have no more privacy than any other
    /// circuits.  The only benefit is that these circuits will not be shared
    /// by multiple streams.)
    ///
    /// This can be undone by calling `set_isolation_group` or `new_isolation_group` on these
    /// preferences.
    pub fn isolate_every_stream(&mut self) -> &mut Self {
        self.isolation = StreamIsolationPreference::EveryStream;
        self
    }

    /// Return a token to describe which connections might use
    /// the same circuit as this one.
    fn isolation_group(&self) -> Option<IsolationToken> {
        use StreamIsolationPreference as SIP;
        match self.isolation {
            SIP::None => None,
            SIP::Explicit(ig) => Some(ig),
            SIP::EveryStream => Some(IsolationToken::new()),
        }
    }

    // TODO: Add some way to be IPFlexible, and require exit to support both.
}

impl TorClient<PreferredRuntime> {
    /// Bootstrap a connection to the Tor network, using the provided `config`.
    ///
    /// Returns a client once there is enough directory material to
    /// connect safely over the Tor network.
    ///
    /// Consider using [`TorClient::builder`] for more fine-grained control.
    ///
    /// # Panics
    ///
    /// If Tokio is being used (the default), panics if created outside the context of a currently
    /// running Tokio runtime. See the documentation for [`PreferredRuntime::current`] for
    /// more information.
    ///
    /// If using `async-std`, either take care to ensure Arti is not compiled with Tokio support,
    /// or manually create an `async-std` runtime using [`tor_rtcompat`] and use it with
    /// [`TorClient::with_runtime`].
    pub async fn create_bootstrapped(config: TorClientConfig) -> crate::Result<Self> {
        let runtime = PreferredRuntime::current()
            .expect("TorClient could not get an asynchronous runtime; are you running in the right context?");

        Self::with_runtime(runtime)
            .config(config)
            .create_bootstrapped()
            .await
    }

    /// Return a new builder for creating TorClient objects.
    ///
    /// If you want to make a [`TorClient`] synchronously, this is what you want; call
    /// `TorClientBuilder::create_unbootstrapped` on the returned builder.
    ///
    /// # Panics
    ///
    /// If Tokio is being used (the default), panics if created outside the context of a currently
    /// running Tokio runtime. See the documentation for `tokio::runtime::Handle::current` for
    /// more information.
    ///
    /// If using `async-std`, either take care to ensure Arti is not compiled with Tokio support,
    /// or manually create an `async-std` runtime using [`tor_rtcompat`] and use it with
    /// [`TorClient::with_runtime`].
    pub fn builder() -> TorClientBuilder<PreferredRuntime> {
        let runtime = PreferredRuntime::current()
            .expect("TorClient could not get an asynchronous runtime; are you running in the right context?");

        TorClientBuilder::new(runtime)
    }
}

impl<R: Runtime> TorClient<R> {
    /// Return a new builder for creating TorClient objects, with a custom provided [`Runtime`].
    ///
    /// See the [`tor_rtcompat`] crate for more information on custom runtimes.
    pub fn with_runtime(runtime: R) -> TorClientBuilder<R> {
        TorClientBuilder::new(runtime)
    }

    /// Implementation of `create_unbootstrapped`, split out in order to avoid manually specifying
    /// double error conversions.
    pub(crate) fn create_inner(
        runtime: R,
        config: TorClientConfig,
        autobootstrap: BootstrapBehavior,
        dirmgr_builder: &dyn crate::builder::DirProviderBuilder<R>,
    ) -> StdResult<Self, ErrorDetail> {
        let circ_cfg = config.get_circmgr_config()?;
        let dir_cfg = config.get_dirmgr_config()?;
        let statemgr = FsStateMgr::from_path(config.storage.expand_state_dir()?)?;
        let addr_cfg = config.address_filter.clone();
        let timeout_cfg = config.stream_timeouts;

        let (status_sender, status_receiver) = postage::watch::channel();
        let status_receiver = status::BootstrapEvents {
            inner: status_receiver,
        };
        let chanmgr = Arc::new(tor_chanmgr::ChanMgr::new(runtime.clone()));
        let circmgr =
            tor_circmgr::CircMgr::new(circ_cfg, statemgr.clone(), &runtime, Arc::clone(&chanmgr))
                .map_err(ErrorDetail::CircMgrSetup)?;

        let dirmgr = dirmgr_builder
            .build(runtime.clone(), Arc::clone(&circmgr), dir_cfg)
            .map_err(crate::Error::into_detail)?;

        let conn_status = chanmgr.bootstrap_events();
        let dir_status = dirmgr.bootstrap_events();
        runtime
            .spawn(status::report_status(
                status_sender,
                conn_status,
                dir_status,
            ))
            .map_err(|e| ErrorDetail::from_spawn("top-level status reporter", e))?;

        runtime
            .spawn(continually_expire_channels(
                runtime.clone(),
                Arc::downgrade(&chanmgr),
            ))
            .map_err(|e| ErrorDetail::from_spawn("channel expiration task", e))?;

        // Launch a daemon task to inform the circmgr about new
        // network parameters.
        runtime
            .spawn(keep_circmgr_params_updated(
                dirmgr.events(),
                Arc::downgrade(&circmgr),
                Arc::downgrade(&dirmgr),
            ))
            .map_err(|e| ErrorDetail::from_spawn("circmgr parameter updater", e))?;

        runtime
            .spawn(update_persistent_state(
                runtime.clone(),
                Arc::downgrade(&circmgr),
                statemgr.clone(),
            ))
            .map_err(|e| ErrorDetail::from_spawn("persistent state updater", e))?;

        runtime
            .spawn(continually_launch_timeout_testing_circuits(
                runtime.clone(),
                Arc::downgrade(&circmgr),
                Arc::downgrade(&dirmgr),
            ))
            .map_err(|e| ErrorDetail::from_spawn("timeout-probe circuit launcher", e))?;

        runtime
            .spawn(continually_preemptively_build_circuits(
                runtime.clone(),
                Arc::downgrade(&circmgr),
                Arc::downgrade(&dirmgr),
            ))
            .map_err(|e| ErrorDetail::from_spawn("preemptive circuit launcher", e))?;

        let client_isolation = IsolationToken::new();

        Ok(TorClient {
            runtime,
            client_isolation,
            connect_prefs: Default::default(),
            circmgr,
            dirmgr,
            statemgr,
            addrcfg: Arc::new(addr_cfg.into()),
            timeoutcfg: Arc::new(timeout_cfg.into()),
            reconfigure_lock: Arc::new(Mutex::new(())),
            status_receiver,
            bootstrap_in_progress: Arc::new(AsyncMutex::new(())),
            should_bootstrap: autobootstrap,
        })
    }

    /// Bootstrap a connection to the Tor network, with a client created by `create_unbootstrapped`.
    ///
    /// Since cloned copies of a `TorClient` share internal state, you can bootstrap a client by
    /// cloning it and running this function in a background task (or similar). This function
    /// only needs to be called on one client in order to bootstrap all of its clones.
    ///
    /// Returns once there is enough directory material to connect safely over the Tor network.
    /// If the client or one of its clones has already been bootstrapped, returns immediately with
    /// success. If a bootstrap is in progress, waits for it to finish, then retries it if it
    /// failed (returning success if it succeeded).
    ///
    /// Bootstrap progress can be tracked by listening to the event receiver returned by
    /// [`bootstrap_events`](TorClient::bootstrap_events).
    ///
    /// # Failures
    ///
    /// If the bootstrapping process fails, returns an error. This function can safely be called
    /// again later to attempt to bootstrap another time.
    pub async fn bootstrap(&self) -> crate::Result<()> {
        self.bootstrap_inner().await.map_err(ErrorDetail::into)
    }

    /// Implementation of `bootstrap`, split out in order to avoid manually specifying
    /// double error conversions.
    async fn bootstrap_inner(&self) -> StdResult<(), ErrorDetail> {
        // Wait for an existing bootstrap attempt to finish first.
        //
        // This is a futures::lock::Mutex, so it's okay to await while we hold it.
        let _bootstrap_lock = self.bootstrap_in_progress.lock().await;

        if self.statemgr.try_lock()?.held() {
            debug!("It appears we have the lock on our state files.");
        } else {
            info!(
                "Another process has the lock on our state files. We'll proceed in read-only mode."
            );
        }

        // If we fail to bootstrap (i.e. we return before the disarm() point below), attempt to
        // unlock the state files.
        let unlock_guard = util::StateMgrUnlockGuard::new(&self.statemgr);

        self.dirmgr.bootstrap().await?;

        self.circmgr.update_network_parameters(
            self.dirmgr
                .latest_netdir()
                .ok_or(ErrorDetail::DirMgr(tor_dirmgr::Error::DirectoryNotPresent))?
                .params(),
        );

        // Since we succeeded, disarm the unlock guard.
        unlock_guard.disarm();

        Ok(())
    }

    /// ## For `BootstrapBehavior::Ondemand` clients
    ///
    /// Initiate a bootstrap by calling `bootstrap` (which is idempotent, so attempts to
    /// bootstrap twice will just do nothing).
    ///
    /// ## For `BootstrapBehavior::Manual` clients
    ///
    /// Check whether a bootstrap is in progress; if one is, wait until it finishes
    /// and then return. (Otherwise, return immediately.)
    async fn wait_for_bootstrap(&self) -> StdResult<(), ErrorDetail> {
        match self.should_bootstrap {
            BootstrapBehavior::OnDemand => {
                self.bootstrap_inner().await?;
            }
            BootstrapBehavior::Manual => {
                // Grab the lock, and immediately release it.  That will ensure that nobody else is trying to bootstrap.
                self.bootstrap_in_progress.lock().await;
            }
        }
        Ok(())
    }

    /// Change the configuration of this TorClient to `new_config`.
    ///
    /// The `how` describes whether to perform an all-or-nothing
    /// reconfiguration: either all of the configuration changes will be
    /// applied, or none will. If you have disabled all-or-nothing changes, then
    /// only fatal errors will be reported in this function's return value.
    ///
    /// This function applies its changes to **all** TorClient instances derived
    /// from the same call to `TorClient::create_*`: even ones whose circuits
    /// are isolated from this handle.
    ///
    /// # Limitations
    ///
    /// Although most options are reconfigurable, there are some whose values
    /// can't be changed on an a running TorClient.  Those options (or their
    /// sections) are explicitly documented not to be changeable.
    ///
    /// Changing some options do not take effect immediately on all open streams
    /// and circuits, but rather affect only future streams and circuits.  Those
    /// are also explicitly documented.
    pub fn reconfigure(
        &self,
        new_config: &TorClientConfig,
        how: tor_config::Reconfigure,
    ) -> crate::Result<()> {
        // We need to hold this lock while we're reconfiguring the client: even
        // though the individual fields have their own synchronization, we can't
        // safely let two threads change them at once.  If we did, then we'd
        // introduce time-of-check/time-of-use bugs in checking our configuration,
        // deciding how to change it, then applying the changes.
        let _guard = self.reconfigure_lock.lock().expect("Poisoned lock");

        match how {
            tor_config::Reconfigure::AllOrNothing => {
                // We have to check before we make any changes.
                self.reconfigure(new_config, tor_config::Reconfigure::CheckAllOrNothing)?;
            }
            tor_config::Reconfigure::CheckAllOrNothing => {}
            tor_config::Reconfigure::WarnOnFailures => {}
            _ => {}
        }

        let circ_cfg = new_config.get_circmgr_config().map_err(wrap_err)?;
        let dir_cfg = new_config.get_dirmgr_config().map_err(wrap_err)?;
        let state_cfg = new_config.storage.expand_state_dir().map_err(wrap_err)?;
        let addr_cfg = &new_config.address_filter;
        let timeout_cfg = &new_config.stream_timeouts;

        if state_cfg != self.statemgr.path() {
            how.cannot_change("storage.state_dir").map_err(wrap_err)?;
        }

        self.circmgr.reconfigure(&circ_cfg, how).map_err(wrap_err)?;
        self.dirmgr.reconfigure(&dir_cfg, how).map_err(wrap_err)?;

        if how == tor_config::Reconfigure::CheckAllOrNothing {
            return Ok(());
        }

        self.addrcfg.replace(addr_cfg.clone());
        self.timeoutcfg.replace(timeout_cfg.clone());

        Ok(())
    }

    /// Return a new isolated `TorClient` handle.
    ///
    /// The two `TorClient`s will share internal state and configuration, but
    /// their streams will never share circuits with one another.
    ///
    /// Use this function when you want separate parts of your program to
    /// each have a TorClient handle, but where you don't want their
    /// activities to be linkable to one another over the Tor network.
    ///
    /// Calling this function is usually preferable to creating a
    /// completely separate TorClient instance, since it can share its
    /// internals with the existing `TorClient`.
    ///
    /// (Connections made with clones of the returned `TorClient` may
    /// share circuits with each other.)
    #[must_use]
    pub fn isolated_client(&self) -> TorClient<R> {
        let mut result = self.clone();
        result.client_isolation = IsolationToken::new();
        result
    }

    /// Launch an anonymized connection to the provided address and port over
    /// the Tor network.
    ///
    /// Note that because Tor prefers to do DNS resolution on the remote side of
    /// the network, this function takes its address as a string:
    ///
    /// ```no_run
    /// # use arti_client::*;use tor_rtcompat::Runtime;
    /// # async fn ex<R:Runtime>(tor_client: TorClient<R>) -> Result<()> {
    /// // The most usual way to connect is via an address-port tuple.
    /// let socket = tor_client.connect(("www.example.com", 443)).await?;
    ///
    /// // You can also specify an address and port as a colon-separated string.
    /// let socket = tor_client.connect("www.example.com:443").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Hostnames are _strongly_ preferred here: if this function allowed the
    /// caller here to provide an IPAddr or [`IpAddr`] or
    /// [`SocketAddr`](std::net::SocketAddr) address, then  
    ///
    /// ```no_run
    /// # use arti_client::*; use tor_rtcompat::Runtime;
    /// # async fn ex<R:Runtime>(tor_client: TorClient<R>) -> Result<()> {
    /// # use std::net::ToSocketAddrs;
    /// // BAD: We're about to leak our target address to the local resolver!
    /// let address = "www.example.com:443".to_socket_addrs().unwrap().next().unwrap();
    /// // 🤯 Oh no! Now any eavesdropper can tell where we're about to connect! 🤯
    ///
    /// // Fortunately, this won't compile, since SocketAddr doesn't implement IntoTorAddr.
    /// // let socket = tor_client.connect(address).await?;
    /// //                                 ^^^^^^^ the trait `IntoTorAddr` is not implemented for `std::net::SocketAddr`
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// If you really do need to connect to an IP address rather than a
    /// hostname, and if you're **sure** that the IP address came from a safe
    /// location, there are a few ways to do so.
    ///
    /// ```no_run
    /// # use arti_client::{TorClient,Result};use tor_rtcompat::Runtime;
    /// # use std::net::{SocketAddr,IpAddr};
    /// # async fn ex<R:Runtime>(tor_client: TorClient<R>) -> Result<()> {
    /// # use std::net::ToSocketAddrs;
    /// // ⚠️This is risky code!⚠️
    /// // (Make sure your addresses came from somewhere safe...)
    ///
    /// // If we have a fixed address, we can just provide it as a string.
    /// let socket = tor_client.connect("192.0.2.22:443").await?;
    /// let socket = tor_client.connect(("192.0.2.22", 443)).await?;
    ///
    /// // If we have a SocketAddr or an IpAddr, we can use the
    /// // DangerouslyIntoTorAddr trait.
    /// use arti_client::DangerouslyIntoTorAddr;
    /// let sockaddr = SocketAddr::from(([192, 0, 2, 22], 443));
    /// let ipaddr = IpAddr::from([192, 0, 2, 22]);
    /// let socket = tor_client.connect(sockaddr.into_tor_addr_dangerously().unwrap()).await?;
    /// let socket = tor_client.connect((ipaddr, 443).into_tor_addr_dangerously().unwrap()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect<A: IntoTorAddr>(&self, target: A) -> crate::Result<DataStream> {
        self.connect_with_prefs(target, &self.connect_prefs).await
    }

    /// Launch an anonymized connection to the provided address and
    /// port over the Tor network, with explicit connection preferences.
    ///
    /// Note that because Tor prefers to do DNS resolution on the remote
    /// side of the network, this function takes its address as a string.
    /// (See [`TorClient::connect()`] for more information.)
    pub async fn connect_with_prefs<A: IntoTorAddr>(
        &self,
        target: A,
        prefs: &StreamPrefs,
    ) -> crate::Result<DataStream> {
        let addr = target.into_tor_addr().map_err(wrap_err)?;
        addr.enforce_config(&self.addrcfg.get())?;
        let (addr, port) = addr.into_string_and_port();

        let exit_ports = [prefs.wrap_target_port(port)];
        let circ = self
            .get_or_launch_exit_circ(&exit_ports, prefs)
            .await
            .map_err(wrap_err)?;
        info!("Got a circuit for {}:{}", addr, port);

        let stream_future = circ.begin_stream(&addr, port, Some(prefs.stream_parameters()));
        // This timeout is needless but harmless for optimistic streams.
        let stream = self
            .runtime
            .timeout(self.timeoutcfg.get().connect_timeout, stream_future)
            .await
            .map_err(|_| ErrorDetail::ExitTimeout)?
            .map_err(wrap_err)?;

        Ok(stream)
    }

    /// Sets the default preferences for future connections made with this client.
    ///
    /// The preferences set with this function will be inherited by clones of this client, but
    /// updates to the preferences in those clones will not propagate back to the original.  I.e.,
    /// the preferences are copied by `clone`.
    ///
    /// Connection preferences always override configuration, even configuration set later
    /// (eg, by a config reload).
    //
    // This function is private just because we're not sure we want to provide this API.
    // https://gitlab.torproject.org/tpo/core/arti/-/merge_requests/250#note_2771238
    fn set_stream_prefs(&mut self, connect_prefs: StreamPrefs) {
        self.connect_prefs = connect_prefs;
    }

    /// Provides a new handle on this client, but with adjusted default preferences.
    ///
    /// Connections made with e.g. [`connect`](TorClient::connect) on the returned handle will use
    /// `connect_prefs`.  This is a convenience wrapper for `clone` and `set_connect_prefs`.
    #[must_use]
    pub fn clone_with_prefs(&self, connect_prefs: StreamPrefs) -> Self {
        let mut result = self.clone();
        result.set_stream_prefs(connect_prefs);
        result
    }

    /// On success, return a list of IP addresses.
    pub async fn resolve(&self, hostname: &str) -> crate::Result<Vec<IpAddr>> {
        self.resolve_with_prefs(hostname, &self.connect_prefs).await
    }

    /// On success, return a list of IP addresses, but use prefs.
    pub async fn resolve_with_prefs(
        &self,
        hostname: &str,
        prefs: &StreamPrefs,
    ) -> crate::Result<Vec<IpAddr>> {
        let addr = (hostname, 0).into_tor_addr().map_err(wrap_err)?;
        addr.enforce_config(&self.addrcfg.get()).map_err(wrap_err)?;

        let circ = self.get_or_launch_exit_circ(&[], prefs).await?;

        let resolve_future = circ.resolve(hostname);
        let addrs = self
            .runtime
            .timeout(self.timeoutcfg.get().resolve_timeout, resolve_future)
            .await
            .map_err(|_| ErrorDetail::ExitTimeout)?
            .map_err(wrap_err)?;

        Ok(addrs)
    }

    /// Perform a remote DNS reverse lookup with the provided IP address.
    ///
    /// On success, return a list of hostnames.
    pub async fn resolve_ptr(&self, addr: IpAddr) -> crate::Result<Vec<String>> {
        self.resolve_ptr_with_prefs(addr, &self.connect_prefs).await
    }

    /// Perform a remote DNS reverse lookup with the provided IP address.
    ///
    /// On success, return a list of hostnames.
    pub async fn resolve_ptr_with_prefs(
        &self,
        addr: IpAddr,
        prefs: &StreamPrefs,
    ) -> crate::Result<Vec<String>> {
        let circ = self.get_or_launch_exit_circ(&[], prefs).await?;

        let resolve_ptr_future = circ.resolve_ptr(addr);
        let hostnames = self
            .runtime
            .timeout(
                self.timeoutcfg.get().resolve_ptr_timeout,
                resolve_ptr_future,
            )
            .await
            .map_err(|_| ErrorDetail::ExitTimeout)?
            .map_err(wrap_err)?;

        Ok(hostnames)
    }

    /// Return a reference to this this client's directory manager.
    ///
    /// This function is unstable. It is only enabled if the crate was
    /// built with the `experimental-api` feature.
    #[cfg(feature = "experimental-api")]
    pub fn dirmgr(&self) -> Arc<dyn tor_dirmgr::DirProvider + Send + Sync> {
        Arc::clone(&self.dirmgr)
    }

    /// Return a reference to this this client's circuit manager.
    ///
    /// This function is unstable. It is only enabled if the crate was
    /// built with the `experimental-api` feature.
    #[cfg(feature = "experimental-api")]
    pub fn circmgr(&self) -> Arc<tor_circmgr::CircMgr<R>> {
        Arc::clone(&self.circmgr)
    }

    /// Return a reference to the runtime being used by this client.
    //
    // This API is not a hostage to fortune since we already require that R: Clone,
    // and necessarily a TorClient must have a clone of it.
    //
    // We provide it simply to save callers who have a TorClient from
    // having to separately keep their own handle,
    pub fn runtime(&self) -> &R {
        &self.runtime
    }

    /// Get or launch an exit-suitable circuit with a given set of
    /// exit ports.
    async fn get_or_launch_exit_circ(
        &self,
        exit_ports: &[TargetPort],
        prefs: &StreamPrefs,
    ) -> StdResult<ClientCirc, ErrorDetail> {
        self.wait_for_bootstrap().await?;
        let dir = self
            .dirmgr
            .latest_netdir()
            .ok_or(ErrorDetail::BootstrapRequired {
                action: "launch a circuit",
            })?;

        let isolation = {
            let mut b = StreamIsolationBuilder::new();
            // Always consider our client_isolation.
            b.owner_token(self.client_isolation);
            // Consider stream isolation too, if it's set.
            if let Some(tok) = prefs.isolation_group() {
                b.stream_token(tok);
            }
            // Failure should be impossible with this builder.
            b.build().expect("Failed to construct StreamIsolation")
        };

        let circ = self
            .circmgr
            .get_or_launch_exit(dir.as_ref().into(), exit_ports, isolation)
            .await
            .map_err(|cause| ErrorDetail::ObtainExitCircuit {
                cause,
                exit_ports: exit_ports.into(),
            })?;
        drop(dir); // This decreases the refcount on the netdir.

        Ok(circ)
    }

    /// Return a current [`status::BootstrapStatus`] describing how close this client
    /// is to being ready for user traffic.
    pub fn bootstrap_status(&self) -> status::BootstrapStatus {
        self.status_receiver.inner.borrow().clone()
    }

    /// Return a stream of [`status::BootstrapStatus`] events that will be updated
    /// whenever the client's status changes.
    ///
    /// The receiver might not receive every update sent to this stream, though
    /// when it does poll the stream it should get the most recent one.
    //
    // TODO(nickm): will this also need to implement Send and 'static?
    pub fn bootstrap_events(&self) -> status::BootstrapEvents {
        self.status_receiver.clone()
    }
}

/// Alias for TorError::from(Error)
pub(crate) fn wrap_err<T>(err: T) -> crate::Error
where
    ErrorDetail: From<T>,
{
    ErrorDetail::from(err).into()
}

/// Whenever a [`DirEvent::NewConsensus`] arrives on `events`, update
/// `circmgr` with the consensus parameters from `dirmgr`.
///
/// Exit when `events` is closed, or one of `circmgr` or `dirmgr` becomes
/// dangling.
///
/// This is a daemon task: it runs indefinitely in the background.
async fn keep_circmgr_params_updated<R: Runtime>(
    mut events: impl futures::Stream<Item = DirEvent> + Unpin,
    circmgr: Weak<tor_circmgr::CircMgr<R>>,
    dirmgr: Weak<dyn tor_dirmgr::DirProvider + Send + Sync>,
) {
    use DirEvent::*;
    while let Some(event) = events.next().await {
        match event {
            NewConsensus => {
                if let (Some(cm), Some(dm)) = (Weak::upgrade(&circmgr), Weak::upgrade(&dirmgr)) {
                    let netdir = dm
                        .latest_netdir()
                        .expect("got new consensus event, without a netdir?");
                    cm.update_network_parameters(netdir.params());
                    cm.update_network(&netdir);
                } else {
                    debug!("Circmgr or dirmgr has disappeared; task exiting.");
                    break;
                }
            }
            NewDescriptors => {
                if let (Some(cm), Some(dm)) = (Weak::upgrade(&circmgr), Weak::upgrade(&dirmgr)) {
                    let netdir = dm
                        .latest_netdir()
                        .expect("got new descriptors event, without a netdir?");
                    cm.update_network(&netdir);
                } else {
                    debug!("Circmgr or dirmgr has disappeared; task exiting.");
                    break;
                }
            }
            _ => {
                // Nothing we recognize.
            }
        }
    }
}

/// Run forever, periodically telling `circmgr` to update its persistent
/// state.
///
/// Exit when we notice that `circmgr` has been dropped.
///
/// This is a daemon task: it runs indefinitely in the background.
async fn update_persistent_state<R: Runtime>(
    runtime: R,
    circmgr: Weak<tor_circmgr::CircMgr<R>>,
    statemgr: FsStateMgr,
) {
    // TODO: Consider moving this function into tor-circmgr after we have more
    // experience with the state system.

    loop {
        if let Some(circmgr) = Weak::upgrade(&circmgr) {
            use tor_persist::LockStatus::*;

            match statemgr.try_lock() {
                Err(e) => {
                    error!("Problem with state lock file: {}", e);
                    break;
                }
                Ok(NewlyAcquired) => {
                    info!("We now own the lock on our state files.");
                    if let Err(e) = circmgr.upgrade_to_owned_persistent_state() {
                        error!("Unable to upgrade to owned state files: {}", e);
                        break;
                    }
                }
                Ok(AlreadyHeld) => {
                    if let Err(e) = circmgr.store_persistent_state() {
                        error!("Unable to flush circmgr state: {}", e);
                        break;
                    }
                }
                Ok(NoLock) => {
                    if let Err(e) = circmgr.reload_persistent_state() {
                        error!("Unable to reload circmgr state: {}", e);
                        break;
                    }
                }
            }
        } else {
            debug!("Circmgr has disappeared; task exiting.");
            return;
        }
        // TODO(nickm): This delay is probably too small.
        //
        // Also, we probably don't even want a fixed delay here.  Instead,
        // we should be updating more frequently when the data is volatile
        // or has important info to save, and not at all when there are no
        // changes.
        runtime.sleep(Duration::from_secs(60)).await;
    }

    error!("State update task is exiting prematurely.");
}

/// Run indefinitely, launching circuits as needed to get a good
/// estimate for our circuit build timeouts.
///
/// Exit when we notice that `circmgr` or `dirmgr` has been dropped.
///
/// This is a daemon task: it runs indefinitely in the background.
///
/// # Note
///
/// I'd prefer this to be handled entirely within the tor-circmgr crate;
/// see [`tor_circmgr::CircMgr::launch_timeout_testing_circuit_if_appropriate`]
/// for more information.
async fn continually_launch_timeout_testing_circuits<R: Runtime>(
    rt: R,
    circmgr: Weak<tor_circmgr::CircMgr<R>>,
    dirmgr: Weak<dyn tor_dirmgr::DirProvider + Send + Sync>,
) {
    while let (Some(cm), Some(dm)) = (Weak::upgrade(&circmgr), Weak::upgrade(&dirmgr)) {
        if let Some(netdir) = dm.latest_netdir() {
            if let Err(e) = cm.launch_timeout_testing_circuit_if_appropriate(&netdir) {
                warn!("Problem launching a timeout testing circuit: {}", e);
            }
            let delay = netdir
                .params()
                .cbt_testing_delay
                .try_into()
                .expect("Out-of-bounds value from BoundedInt32");

            drop((cm, dm));
            rt.sleep(delay).await;
        } else {
            // TODO(eta): ideally, this should wait until we successfully bootstrap using
            //            the bootstrap status API
            rt.sleep(Duration::from_secs(10)).await;
        }
    }
}

/// Run indefinitely, launching circuits where the preemptive circuit
/// predictor thinks it'd be a good idea to have them.
///
/// Exit when we notice that `circmgr` or `dirmgr` has been dropped.
///
/// This is a daemon task: it runs indefinitely in the background.
///
/// # Note
///
/// This would be better handled entirely within `tor-circmgr`, like
/// other daemon tasks.
async fn continually_preemptively_build_circuits<R: Runtime>(
    rt: R,
    circmgr: Weak<tor_circmgr::CircMgr<R>>,
    dirmgr: Weak<dyn tor_dirmgr::DirProvider + Send + Sync>,
) {
    while let (Some(cm), Some(dm)) = (Weak::upgrade(&circmgr), Weak::upgrade(&dirmgr)) {
        if let Some(netdir) = dm.latest_netdir() {
            cm.launch_circuits_preemptively(DirInfo::Directory(&netdir))
                .await;
            rt.sleep(Duration::from_secs(10)).await;
        } else {
            // TODO(eta): ideally, this should wait until we successfully bootstrap using
            //            the bootstrap status API
            rt.sleep(Duration::from_secs(10)).await;
        }
    }
}
/// Periodically expire any channels that have been unused beyond
/// the maximum duration allowed.
///
/// Exist when we find that `chanmgr` is dropped
///
/// This is a daemon task that runs indefinitely in the background
async fn continually_expire_channels<R: Runtime>(rt: R, chanmgr: Weak<tor_chanmgr::ChanMgr<R>>) {
    loop {
        let delay = if let Some(cm) = Weak::upgrade(&chanmgr) {
            cm.expire_channels()
        } else {
            // channel manager is closed.
            return;
        };
        // This will sometimes be an underestimate, but it's no big deal; we just sleep some more.
        rt.sleep(Duration::from_secs(delay.as_secs())).await;
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::config::TorClientConfigBuilder;
    use crate::{ErrorKind, HasKind};

    #[test]
    fn create_unbootstrapped() {
        tor_rtcompat::test_with_one_runtime!(|rt| async {
            let state_dir = tempfile::tempdir().unwrap();
            let cache_dir = tempfile::tempdir().unwrap();
            let cfg = TorClientConfigBuilder::from_directories(state_dir, cache_dir)
                .build()
                .unwrap();
            let _ = TorClient::with_runtime(rt)
                .config(cfg)
                .bootstrap_behavior(BootstrapBehavior::Manual)
                .create_unbootstrapped()
                .unwrap();
        });
    }

    #[test]
    fn unbootstrapped_client_unusable() {
        tor_rtcompat::test_with_one_runtime!(|rt| async {
            let state_dir = tempfile::tempdir().unwrap();
            let cache_dir = tempfile::tempdir().unwrap();
            let cfg = TorClientConfigBuilder::from_directories(state_dir, cache_dir)
                .build()
                .unwrap();
            let client = TorClient::with_runtime(rt)
                .config(cfg)
                .bootstrap_behavior(BootstrapBehavior::Manual)
                .create_unbootstrapped()
                .unwrap();
            let result = client.connect("example.com:80").await;
            assert!(result.is_err());
            assert_eq!(result.err().unwrap().kind(), ErrorKind::BootstrapRequired);
        });
    }
}
