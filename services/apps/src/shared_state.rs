// Shared state of the Apps service.

use crate::apps_registry::AppsRegistry;
use crate::config::Config;
use crate::generated::common::*;
use crate::update_scheduler::SchedulerMessage;
use common::traits::StateLogger;
use log::{error, info};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use url::Url;
use vhost_server::config::VhostApi;

pub struct DeviceInfo;

impl From<&Config> for AppsSharedData {
    fn from(config: &Config) -> Self {
        AppsSharedData {
            config: config.clone(),
            ..Default::default()
        }
    }
}

pub struct AppsSharedData {
    pub config: Config,
    pub vhost_api: VhostApi,
    pub state: AppsServiceState,
    pub registry: AppsRegistry,
    pub token_provider: Option<TokenProviderProxy>,
    pub device_info: Option<DeviceInfo>,
    pub scheduler: Option<Sender<SchedulerMessage>>,
    pub downloadings: HashMap<Url, DownloadingCanceller>,
}

pub struct DownloadingCanceller {
    pub cancel_sender: Option<Sender<()>>,
    pub cancelled: bool,
}

impl StateLogger for AppsSharedData {
    fn log(&self) {
        info!("  State: {:?}", self.state);
        info!(
            "  Token Provider registerer: {}",
            self.token_provider.is_some()
        );
        self.registry.event_broadcaster.log();
        self.registry.print_pool_status();
    }
}

impl Default for AppsSharedData {
    fn default() -> Self {
        AppsSharedData {
            config: Config::default(),
            vhost_api: VhostApi::default(),
            state: AppsServiceState::Initializing,
            registry: AppsRegistry::default(),
            token_provider: None,
            device_info: None,
            scheduler: None,
            downloadings: HashMap::new(),
        }
    }
}

impl AppsSharedData {
    pub fn get_all_apps(&self) -> Result<Vec<AppsObject>, AppsServiceError> {
        if self.state != AppsServiceState::Running {
            error!("get_all_apps AppsService state is invalid!");
            return Err(AppsServiceError::InvalidState);
        }
        Ok(self.registry.get_all())
    }

    pub fn get_by_manifest_url(&self, manifest_url: &Url) -> Result<AppsObject, AppsServiceError> {
        match self.registry.get_by_manifest_url(manifest_url) {
            Some(app) => Ok(AppsObject::from(&app)),
            None => Err(AppsServiceError::AppNotFound),
        }
    }
}
