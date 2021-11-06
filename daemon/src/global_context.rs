/// Structure holding the "global context" needed to
/// run a transport and hook up a session.
use crate::config::Config;
use crate::shared_state::create_shared_state;
use common::remote_service::{RemoteServiceManager, SharedRemoteServiceManager};
use common::remote_services_registrar::RemoteServicesRegistrar;
use common::tokens::SharedTokensManager;
use common::traits::{SessionContext, Shared, SharedServiceState, SharedSessionContext};

#[derive(Clone)]
pub struct GlobalContext {
    pub config: Config,
    pub tokens_manager: SharedTokensManager,
    pub remote_service_manager: SharedRemoteServiceManager,
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
        create_shared_state(config);

        let tokens_manager = geckobridge::service::GeckoBridgeService::shared_state()
            .lock()
            .get_tokens_manager();
        Self {
            config: config.clone(),
            tokens_manager,
            remote_service_manager,
            session_context: Shared::adopt(SessionContext::default()),
        }
    }
}
