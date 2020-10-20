// Shared state of the Apps service.

use crate::apps_registry::AppsRegistry;
use crate::config::Config;
use crate::generated::common::*;
use crate::update_scheduler::SchedulerMessage;
use common::traits::Shared;
use std::sync::mpsc::Sender;
use vhost_server::config::VhostApi;

lazy_static! {
    pub(crate) static ref APPS_SHARED_SHARED_DATA: Shared<AppsSharedData> =
        Shared::adopt(AppsSharedData::default());
}

pub struct AppsSharedData {
    pub config: Config,
    pub vhost_api: VhostApi,
    pub state: AppsServiceState,
    pub registry: AppsRegistry,
    pub scheduler: Option<Sender<SchedulerMessage>>,
}

impl AppsSharedData {
    pub fn default() -> Self {
        AppsSharedData {
            config: Config::default(),
            vhost_api: VhostApi::default(),
            state: AppsServiceState::Initializing,
            registry: AppsRegistry::default(),
            scheduler: None,
        }
    }

    pub fn get_all_apps(&self) -> Result<Vec<AppsObject>, AppsServiceError> {
        if self.state != AppsServiceState::Running {
            return Err(AppsServiceError::InvalidState);
        }
        Ok(self.registry.get_all())
    }

    pub fn get_by_manifest_url(&self, manifest_url: &str) -> Result<AppsObject, AppsServiceError> {
        match self.registry.get_by_manifest_url(manifest_url) {
            Some(app) => Ok(AppsObject::from(&app)),
            None => Err(AppsServiceError::AppNotFound),
        }
    }
}
