/// Implementation of the devicecapability service.
use crate::config::DeviceCapabilityConfig;
use crate::generated::common::*;
use crate::generated::service::*;
use common::core::BaseMessage;
use common::traits::{
    OriginAttributes, Service, SessionSupport, Shared, SharedSessionContext, StateLogger, TrackerId,
};
use log::{error, info};
use std::thread;

pub struct DeviceCapabilitySharedData {
    pub config: DeviceCapabilityConfig,
}

impl StateLogger for DeviceCapabilitySharedData {}

pub struct DeviceCapabilityService {
    id: TrackerId,
    state: Shared<DeviceCapabilitySharedData>,
}

impl DeviceCapabilityManager for DeviceCapabilityService {}

impl DeviceCapabilityFactoryMethods for DeviceCapabilityService {
    fn get(&mut self, responder: &DeviceCapabilityFactoryGetResponder, name: String) {
        let responder = responder.clone();
        let shared = self.state.clone();
        thread::spawn(move || {
            let config = &shared.lock().config;
            match config.get(&name) {
                Ok(value) => responder.resolve(value),
                Err(err) => {
                    error!("get error {}", err);
                    responder.reject();
                }
            }
        });
    }
}

impl Service<DeviceCapabilityService> for DeviceCapabilityService {
    // Shared among instances.
    type State = DeviceCapabilitySharedData;

    fn shared_state() -> Shared<Self::State> {
        Shared::adopt(DeviceCapabilitySharedData {
            config: DeviceCapabilityConfig::default(),
        })
    }

    fn create(
        _attrs: &OriginAttributes,
        _context: SharedSessionContext,
        state: Shared<Self::State>,
        helper: SessionSupport,
    ) -> Result<DeviceCapabilityService, String> {
        info!("DeviceCapabilitiyService::create");
        let service_id = helper.session_tracker_id().service();
        Ok(DeviceCapabilityService {
            id: service_id,
            state,
        })
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<DeviceCapabilityManagerFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("DeviceCapabilityService request: {:?}", req),
            Err(err) => format!(
                "Unable to format DeviceCapabilityService request: {:?}",
                err
            ),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        info!("releasing object {}", object_id);
        true
    }
}

impl Drop for DeviceCapabilityService {
    fn drop(&mut self) {
        info!("Dropping DeviceCapabilityService #{}", self.id);
    }
}
