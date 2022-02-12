// This is an ugly macro that turns a service into a remoted one.
// Doing something cleaner needs more refactoring that will happen
// in the future.

// Exposes a service as a remote one.
#[macro_export]
macro_rules! expose_remote_service {
    ($use_name:path, $crate_name:ident, $service_name:ident) => {

// Session manager for a child daemon.

mod session {

use $use_name;
use std::env;
use common::core::{BaseMessage, GetServiceResponse};
use common::object_tracker::ObjectTracker;
use common::remote_service::{ChildToParentMessage, LockedIpcWriter, ParentToChildMessage, IPC_BUFFER_SIZE};
use common::socket_pair::PairedStream;
use common::traits::{SessionTrackerId, MessageSender, StdSender,
    SharedEventMap, EventMapKey, IdFactory, MessageKind, ObjectTrackerMethods, OriginAttributes, Service,
    SessionContext, SessionSupport, Shared, SharedIdFactory, SharedSessionContext, EmptyConfig,
    TrackerId, SharedServiceState
};
use common::try_continue;
use log::{debug, error, info};
use nix::sys::socket::{getsockopt, sockopt};
use parking_lot::Mutex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufReader, Write};
use std::os::unix::io::{FromRawFd, RawFd};
use std::sync::{mpsc, Arc};
use std::thread;

fn handle_ipc(fd: RawFd) {
    // Initialize the service shared data.
    $crate_name::service::$service_name::init_shared_state(&EmptyConfig);

    let (sender, receiver) = mpsc::channel();
    let base_stream = unsafe { PairedStream::from_raw_fd(fd) };
    let reader_stream = base_stream.clone();
    let parent_writer = LockedIpcWriter::new(base_stream);

    // Thread listening on incoming messages.
    let inner_parent_writer = parent_writer.clone();
    let _handle = thread::Builder::new()
        .name("child daemon ipc thread".into())
        .spawn(move || {
            debug!("In child daemon ipc thread");
            let event_map: SharedEventMap = Shared::default();
            let mut session = Session::new(sender, event_map.clone());

            loop {
                debug!("Waiting for ParentToChildMessage...");
                let reader = BufReader::with_capacity(IPC_BUFFER_SIZE, reader_stream.clone());
                let message: ParentToChildMessage = match bincode::deserialize_from(reader) {
                    Ok(res) => res,
                    Err(err) => {
                        error!("Error decoding parent message: {}", err);
                        // If we fail to decode, there is no good way to recover so instead we just
                        // exit the whole process.
                        std::process::exit(2);
                    }
                };
                match message {
                    ParentToChildMessage::CreateService(service_name, service_fingerprint, tracker_id, origin_attributes) => {
                        info!(
                            "About to create service `{}` {:?} {:?}",
                            service_name, tracker_id, origin_attributes
                        );
                        let res =
                            session.get_service(&service_name, &service_fingerprint, tracker_id, &origin_attributes);
                        debug!("session.get_service result is {:?}", res);
                        let response = ChildToParentMessage::Created(tracker_id, res);
                        if let Err(err) = inner_parent_writer.serialize(&response) {
                            error!("Failed to serialize Created: {}", err);
                        }
                        debug!("result sent back to parent");
                    }
                    ParentToChildMessage::ReleaseService(service_name, tracker_id) => {
                        debug!("About to release service {} {:?}", service_name, tracker_id);
                        session.release_service(tracker_id);
                    }
                    ParentToChildMessage::Request(session_id, data) => {
                        debug!("Received request len={}", data.len());
                        use bincode::Options;

                        match common::deserialize_bincode(&data) {
                            Ok(msg) => {
                                session.process_base_message(session_id, &msg);
                            }
                            Err(err) => error!("Failed to unpack BaseMessage: {}", err),
                        }
                    }
                    ParentToChildMessage::EnableEvent(session_id, object_id, event_id) => {
                        // Update the session event map.
                        debug!("Enabling event session #{} object #{} event #{}", session_id, object_id, event_id);
                        event_map.lock().insert(EventMapKey::new(session_id, object_id, event_id), true);
                    }
                    ParentToChildMessage::DisableEvent(session_id, object_id, event_id) => {
                        // Update the session event map.
                        debug!("Disabling event session #{} object #{}  event #{}", session_id, object_id, event_id);
                        event_map.lock().remove(&EventMapKey::new(session_id, object_id, event_id));
                    }
                    ParentToChildMessage::ReleaseObject(tracker_id, object_id) => {
                        let res = session.on_release_object(tracker_id, object_id);
                        let response = ChildToParentMessage::ObjectReleased(tracker_id, res);
                        if let Err(err) = inner_parent_writer.serialize(&response) {
                            error!("Failed to serialize ObjectReleased: {}", err);
                        }
                    }
                }
            }
        })
        .expect("Failed to create child daemon ipc thread");

    // Block our main thread on relaying messages back to the parent. That's ok
    // because we don't have any other event loop.
    loop {
        match receiver.recv() {
            Ok(message) => {
                if let MessageKind::Data(tracker_id, data) = message {
                    if let Err(err) =
                    parent_writer.serialize(&ChildToParentMessage::Packet(tracker_id, data)) {
                        error!("Failed to serialize Packet: {}", err);
                    }
                } else {
                    error!("Child daemons should only relay Data!");
                }
            }
            Err(err) => {
                error!("Failed to receive message on child event loop.");
            }
        }
    }
}

// The session only tracks services, not individual objects.
pub enum TrackableServices {
    $service_name(RefCell<$service_name>),
}

pub struct Session {
    tracker: ObjectTracker<TrackableServices, SessionTrackerId>,
    context: SharedSessionContext,
    sender: mpsc::Sender<MessageKind>,
    id_factory: SharedIdFactory,
    event_map: SharedEventMap,
    shared_state: Shared<<$service_name as SharedServiceState>::State>,
}

#[derive(Debug)]
pub enum SessionError {
    MissingEnvVar,
    BadFdValue(String),
}

impl Session {
    pub fn start() -> Result<(), SessionError> {
        if let Ok(fd_s) = env::var("IPC_FD") {
            if let Ok(fd) = fd_s.parse::<RawFd>() {
                debug!("Starting with fd {} for parent IPC", fd);
                // handle_ipc() doesn't return until the ipc thread stops.
                handle_ipc(fd);
            } else {
                return Err(SessionError::BadFdValue(fd_s))
            }
        } else {
            return Err(SessionError::MissingEnvVar);
        }

        Ok(())
    }

    fn new(sender: mpsc::Sender<MessageKind>, event_map: SharedEventMap) -> Self {
        Session {
            tracker: ObjectTracker::default(),
            context: Shared::adopt(SessionContext::default()),
            sender,
            // Generate ids starting at u32::MAX for now until we have a better way to
            // proxy the IdFactory
            id_factory: Shared::adopt(IdFactory::new(std::u32::MAX)),
            event_map,
            shared_state: $service_name::shared_state(),
        }
    }

    pub fn get_service(
        &mut self,
        service_name: &str,
        service_fingerprint: &str,
        tracker_id: SessionTrackerId,
        origin_attributes: &OriginAttributes,
    ) -> GetServiceResponse {
        info!("Creating service `{}`", service_name);

        // Early return if we can check that we don't have this service registered.
        if service_name != $crate_name::generated::service::SERVICE_NAME {
            return GetServiceResponse::UnknownService;
        }

        if service_fingerprint != $crate_name::generated::service::SERVICE_FINGERPRINT {
            error!("Fingerprint mismatch for service {}. Expected {} but got {}",
            service_name, $crate_name::generated::service::SERVICE_FINGERPRINT, service_fingerprint);
            return GetServiceResponse::FingerprintMismatch;
        }

        let helpers = SessionSupport::new(
            tracker_id,
            MessageSender::new(Box::new(StdSender::new(&self.sender))),
            self.id_factory.clone(),
            self.event_map.clone());

        // Tries to instanciate a service.
        if !$crate_name::generated::service::check_service_permission(origin_attributes) {
            error!("Could not create service {}: required permission not present.", stringify!(service));
            GetServiceResponse::MissingPermission
        } else {
            match $service_name::create(
            &origin_attributes.clone(),
            self.context.clone(),
            helpers,
            ) {
                Ok(s) => {
                    let s_item = TrackableServices::$service_name(RefCell::new(s));
                    self.tracker.track_with(s_item, tracker_id);

                    GetServiceResponse::Success(0)
                },
                Err(err) => {
                    error!("Could not create service {} !", stringify!(service));
                    GetServiceResponse::InternalError(err)
                }
            }
        }
    }

    pub fn release_service(&mut self, tracker_id: SessionTrackerId) {
        self.tracker.untrack(tracker_id);
    }

    pub fn process_base_message(&mut self, session_id: u32, message: &BaseMessage) {
        if message.service == 0 {
            // Special case, this is the Core service.
            error!("The child daemon should not receive messages targeted at the Core service!");
            return;
        }

        // Retrieve the service from the object tracker, and
        // forward it the payload to decode.
        let tracker_id = SessionTrackerId::from(session_id, message.service);
        match self.tracker.get(tracker_id) {
            Some(TrackableServices::$service_name(ref service)) => {
                // Creates a temporary session helper bound to this session id.
                let helper = SessionSupport::new(
                                tracker_id,
                                MessageSender::new(Box::new(StdSender::new(&self.sender))),
                                self.id_factory.clone(),
                                self.event_map.clone());
                service.borrow_mut().on_request(&helper, message)
            }
            None => {
                error!("Unable to find service with session {} and id {}", session_id, message.service);
            }
        }
    }

    pub fn on_release_object(&self, tracker_id: SessionTrackerId, object_id: u32) -> bool {
        if tracker_id.service() == 0 {
            // Special case, this is the Core service.
            error!("The child daemon should not receive messages targeted at the Core service!");
            return false;
        }

        // Look for the service with the matching id.
        match self.tracker.get(tracker_id) {
            Some(TrackableServices::$service_name(ref service)) => {
                service.borrow_mut().release_object(object_id)
            }
            None => {
                error!("Unable to find service with id: {:?}", tracker_id);
                false
            }
        }
    }
}

}
    };
} // End of macro
