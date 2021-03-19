// Shared state of the Apps service.

use crate::apps_registry::AppsRegistry;
use crate::config::Config;
use crate::generated::common::*;
use crate::update_scheduler::SchedulerMessage;
use common::traits::{Shared, StateLogger};
use log::info;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use vhost_server::config::VhostApi;

pub struct DeviceInfo;

lazy_static! {
    pub(crate) static ref APPS_SHARED_SHARED_DATA: Shared<AppsSharedData> =
        Shared::adopt(AppsSharedData::default());
}

pub struct AppsSharedData {
    pub config: Config,
    pub vhost_api: VhostApi,
    pub state: AppsServiceState,
    pub registry: AppsRegistry,
    pub token_provider: TokenProvider,
    pub device_info: Option<DeviceInfo>,
    pub scheduler: Option<Sender<SchedulerMessage>>,
    pub downloadings: HashMap<String, Sender<()>>, // Update url -> cancel sender
}

impl StateLogger for AppsSharedData {
    fn log(&self) {
        info!("  State: {:?}", self.state);
        info!(
            "  Token Provider registerer: {}",
            self.token_provider.get_provider().is_some()
        );
        self.registry.event_broadcaster.log();
    }
}

impl Default for AppsSharedData {
    fn default() -> Self {
        AppsSharedData {
            config: Config::default(),
            vhost_api: VhostApi::default(),
            state: AppsServiceState::Initializing,
            registry: AppsRegistry::default(),
            token_provider: TokenProvider::default(),
            device_info: None,
            scheduler: None,
            downloadings: HashMap::new(),
        }
    }
}

impl AppsSharedData {
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

#[derive(Clone)]
pub struct TokenProvider {
    proxy: Option<TokenProviderProxy>,
    token: Option<Token>,
    valid_until: Instant,
}

impl TokenProvider {
    fn default() -> Self {
        Self {
            proxy: None,
            token: None,
            valid_until: Instant::now(),
        }
    }

    pub fn set_provider(&mut self, provider: TokenProviderProxy) {
        self.proxy = Some(provider);
    }

    pub fn get_provider(&self) -> Option<TokenProviderProxy> {
        self.proxy.clone()
    }

    pub fn set_token(&mut self, token: Token) {
        let one_hour = 60 * 60;
        self.valid_until = Instant::now() + Duration::from_secs(one_hour);
        self.token = Some(token);
    }

    pub fn has_valid_token(&self) -> Option<Token> {
        match &self.token {
            None => None,
            Some(token) => {
                if self.valid_until > Instant::now() {
                    Some(token.clone())
                } else {
                    None
                }
            }
        }
    }
}
