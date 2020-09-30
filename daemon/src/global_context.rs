/// Structure holding the "global context" needed to
/// run a transport and hook up a session.
use crate::config::Config;
use crate::shared_state::create_shared_state;
use crate::shared_state::SharedStateMap;
use crate::tokens::{SharedTokensManager, TokensManager};
use common::remote_service::{RemoteServiceManager, SharedRemoteServiceManager};
use common::remote_services_registrar::RemoteServicesRegistrar;
use common::traits::{SessionContext, Shared, SharedSessionContext};

#[derive(Clone)]
pub struct GlobalContext {
    pub config: Config,
    pub tokens_manager: SharedTokensManager,
    pub remote_service_manager: SharedRemoteServiceManager,
    service_state: SharedStateMap,
    pub session_context: SharedSessionContext,
}

impl GlobalContext {
    pub fn new(config: &Config) -> Self {
        let registrar = RemoteServicesRegistrar::new(
            &config.general.remote_services_config,
            &config.general.remote_services_path,
        );
        let remote_service_manager = Shared::adopt(RemoteServiceManager::new(
            &config.general.remote_services_path,
            registrar,
        ));

        Self {
            config: config.clone(),
            tokens_manager: TokensManager::new_shareable(),
            remote_service_manager,
            service_state: create_shared_state(),
            session_context: Shared::adopt(SessionContext::default()),
        }
    }

    pub fn service_state(&self) -> SharedStateMap {
        self.service_state.clone()
    }
}
