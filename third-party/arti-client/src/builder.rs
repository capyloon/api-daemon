//! Types for conveniently constructing TorClients.

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use crate::{err::ErrorDetail, BootstrapBehavior, Result, TorClient, TorClientConfig};
use std::sync::Arc;
use tor_dirmgr::DirMgrConfig;
use tor_rtcompat::Runtime;

/// An object that knows how to construct some kind of DirProvider.
///
/// Note that this type is only actually exposed when the `experimental-api`
/// feature is enabled.
#[allow(unreachable_pub)]
pub trait DirProviderBuilder<R: Runtime> {
    fn build(
        &self,
        runtime: R,
        circmgr: Arc<tor_circmgr::CircMgr<R>>,
        config: DirMgrConfig,
    ) -> Result<Arc<dyn tor_dirmgr::DirProvider + Send + Sync + 'static>>;
}

/// A DirProviderBuilder that constructs a regular DirMgr.
#[derive(Clone, Debug)]
struct DirMgrBuilder {}

impl<R: Runtime> DirProviderBuilder<R> for DirMgrBuilder {
    fn build(
        &self,
        runtime: R,
        circmgr: Arc<tor_circmgr::CircMgr<R>>,
        config: DirMgrConfig,
    ) -> Result<Arc<dyn tor_dirmgr::DirProvider + Send + Sync + 'static>> {
        let dirmgr = tor_dirmgr::DirMgr::create_unbootstrapped(config, runtime, circmgr)
            .map_err(ErrorDetail::from)?;
        Ok(Arc::new(dirmgr))
    }
}

/// An object for constructing a [`TorClient`].
///
/// Returned by [`TorClient::builder()`].
#[derive(Clone)]
#[must_use]
pub struct TorClientBuilder<R: Runtime> {
    /// The runtime for the client to use
    runtime: R,
    /// The client's configuration.
    config: TorClientConfig,
    /// How the client should behave when it is asked to do something on the Tor
    /// network before `bootstrap()` is called.
    bootstrap_behavior: BootstrapBehavior,
    /// Optional object to construct a DirProvider.
    ///
    /// Wrapped in an Arc so that we don't need to force DirProviderBuilder to
    /// implement Clone.
    dirmgr_builder: Arc<dyn DirProviderBuilder<R>>,
}

impl<R: Runtime> TorClientBuilder<R> {
    /// Construct a new TorClientBuilder with the given runtime.
    pub(crate) fn new(runtime: R) -> Self {
        Self {
            runtime,
            config: TorClientConfig::default(),
            bootstrap_behavior: BootstrapBehavior::default(),
            dirmgr_builder: Arc::new(DirMgrBuilder {}),
        }
    }

    /// Set the configuration for the `TorClient` under construction.
    ///
    /// If not called, then a compiled-in default configuration will be used.
    pub fn config(mut self, config: TorClientConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the bootstrap behavior for the `TorClient` under construction.
    ///
    /// If not called, then the default ([`BootstrapBehavior::OnDemand`]) will
    /// be used.
    pub fn bootstrap_behavior(mut self, bootstrap_behavior: BootstrapBehavior) -> Self {
        self.bootstrap_behavior = bootstrap_behavior;
        self
    }

    /// Override the default function used to construct the directory provider.
    ///
    /// Only available when compiled with the `experimental-api` feature: this
    /// code is unstable.
    #[cfg(all(feature = "experimental-api", feature = "error_detail"))]
    pub fn dirmgr_builder<B>(mut self, builder: Arc<dyn DirProviderBuilder<R>>) -> Self
    where
        B: DirProviderBuilder<R> + 'static,
    {
        self.dirmgr_builder = builder;
        self
    }

    /// Create a `TorClient` from this builder, without automatically launching
    /// the bootstrap process.
    ///
    /// If you have left the default [`BootstrapBehavior`] in place, the client
    /// will bootstrap itself as soon any attempt is made to use it.  You can
    /// also bootstrap the client yourself by running its
    /// [`bootstrap()`](TorClient::bootstrap) method.
    ///
    /// If you have replaced the default behavior with [`BootstrapBehavior::Manual`],
    /// any attempts to use the client will fail with an error of kind
    /// [`ErrorKind::BootstrapRequired`](crate::ErrorKind::BootstrapRequired),
    /// until you have called [`TorClient::bootstrap`] yourself.  
    /// This option is useful if you wish to have control over the bootstrap
    /// process (for example, you might wish to avoid initiating network
    /// connections until explicit user confirmation is given).
    pub fn create_unbootstrapped(self) -> Result<TorClient<R>> {
        TorClient::create_inner(
            self.runtime,
            self.config,
            self.bootstrap_behavior,
            self.dirmgr_builder.as_ref(),
        )
        .map_err(ErrorDetail::into)
    }

    /// Create a TorClient from this builder, and try to bootstrap it.
    pub async fn create_bootstrapped(self) -> Result<TorClient<R>> {
        let r = self.create_unbootstrapped()?;
        r.bootstrap().await?;
        Ok(r)
    }
}
