/// Structure holding the "global context" needed to
/// run a transport and hook up a session.
use crate::config::Config;
use crate::shared_state::create_shared_state;
use crate::shared_state::{SharedStateKind, SharedStateMap};
use common::remote_service::{RemoteServiceManager, SharedRemoteServiceManager};
use common::remote_services_registrar::RemoteServicesRegistrar;
use common::tokens::SharedTokensManager;
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

        // Get the shared tokens manager from the GeckoBridge shared state.
        let service_state = create_shared_state();

        let tokens_manager = {
            let lock = service_state.lock();
            let shared_data = match lock.get(&"GeckoBridge".to_string()) {
                Some(SharedStateKind::GeckoBridgeService(data)) => data,
                _ => panic!("Missing shared state for GeckoBridge!!"),
            };
            let tokens_manager = shared_data.lock().get_tokens_manager();
            tokens_manager
        };

        Self {
            config: config.clone(),
            tokens_manager,
            remote_service_manager,
            service_state,
            session_context: Shared::adopt(SessionContext::default()),
        }
    }

    pub fn service_state(&self) -> SharedStateMap {
        self.service_state.clone()
    }
}
