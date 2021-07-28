use crate::generated::common::*;
use crate::generated::service::*;
use common::core::BaseMessage;
use common::traits::{
    DispatcherId, EmptyConfig, OriginAttributes, Service, SessionSupport, Shared,
    SharedServiceState, SharedSessionContext, StateLogger, TrackerId,
};

use log::{debug, info};

pub struct SharedObj {
    volume_state: AudioVolumeState,
    event_broadcaster: AudioVolumeEventBroadcaster,
}

impl From<&EmptyConfig> for SharedObj {
    fn from(_config: &EmptyConfig) -> Self {
        Self {
            volume_state: AudioVolumeState::None,
            event_broadcaster: AudioVolumeEventBroadcaster::default(),
        }
    }
}

impl StateLogger for SharedObj {
    fn log(&self) {
        self.event_broadcaster.log();
    }
}

pub struct AudioVolume {
    id: TrackerId,
    shared_obj: Shared<SharedObj>,
    dispatcher_id: DispatcherId,
}

impl AudioVolume {
    fn set_state(&self, state: AudioVolumeState) {
        let mut shared_lock = self.shared_obj.lock();
        shared_lock.volume_state = state;
        debug!("broadcast AudioVolumeState {:?}", shared_lock.volume_state);
        shared_lock
            .event_broadcaster
            .broadcast_audio_volume_changed(shared_lock.volume_state);
    }
}

impl AudioVolumeManager for AudioVolume {}

impl AudioVolumeMethods for AudioVolume {
    fn request_volume_up(&mut self, responder: &AudioVolumeRequestVolumeUpResponder) {
        self.set_state(AudioVolumeState::VolumeUp);
        responder.resolve();
    }

    fn request_volume_down(&mut self, responder: &AudioVolumeRequestVolumeDownResponder) {
        self.set_state(AudioVolumeState::VolumeDown);
        responder.resolve();
    }

    fn request_volume_show(&mut self, responder: &AudioVolumeRequestVolumeShowResponder) {
        self.set_state(AudioVolumeState::VolumeShow);
        responder.resolve();
    }
}

common::impl_shared_state!(AudioVolume, SharedObj, EmptyConfig);

impl Service<AudioVolume> for AudioVolume {
    fn create(
        _attrs: &OriginAttributes,
        _context: SharedSessionContext,
        helper: SessionSupport,
    ) -> Result<AudioVolume, String> {
        info!("AudioVolumeService::create");
        let service_id = helper.session_tracker_id().service();
        let event_dispatcher = AudioVolumeEventDispatcher::from(helper, 0);
        let shared_obj = Self::shared_state();
        let dispatcher_id = shared_obj.lock().event_broadcaster.add(&event_dispatcher);
        info!("AudioVolume::create with dispatcher_id {}", dispatcher_id);

        let service = AudioVolume {
            id: service_id,
            shared_obj,
            dispatcher_id,
        };

        Ok(service)
    }

    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        info!("AudioVolumeService::format_request");
        let req: Result<AudioVolumeManagerFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("AudioVolumeService request: {:?}", req),
            Err(err) => format!("Unable to AudioVolumeService request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        info!("incoming request {:?} ", message);
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        info!("releasing object {}", object_id);
        true
    }
}

impl Drop for AudioVolume {
    fn drop(&mut self) {
        debug!(
            "Dropping AudioVolume Service#{}, dispatcher_id {}",
            self.id, self.dispatcher_id
        );
        let shared_lock = &mut self.shared_obj.lock();
        shared_lock.event_broadcaster.remove(self.dispatcher_id);
    }
}
