use crate::config::Config;
use crate::session_counter::SessionKind;
use crate::shared_state::{
    enabled_services, format_request_helper, on_create_service_helper, on_release_object_helper,
    process_base_message_helper, SharedStateMap, TrackableServices,
};
use bincode::{self, Options};
use common::core::{
    BaseMessage, BaseMessageKind, CoreRequest, CoreResponse, DisableEventListenerRequest,
    DisableEventListenerResponse, EnableEventListenerRequest, EnableEventListenerResponse,
    GetServiceRequest, GetServiceResponse, HasServiceRequest, HasServiceResponse,
    ReleaseObjectRequest, ReleaseObjectResponse, SessionAck, SessionHandshake,
};
use common::object_tracker::ObjectTracker;
use common::remote_service::{RemoteService, SharedRemoteServiceManager};
use common::tokens::SharedTokensManager;
use common::traits::{
    EventMapKey, IdFactory, MessageSender, ObjectTrackerMethods, OriginAttributes, Service,
    SessionSupport, SessionTrackerId, Shared, SharedEventMap, SharedSessionContext, TrackerId,
};
use log::{debug, error, info, warn};
use std::cell::RefCell;
use std::collections::HashSet;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
enum SessionState {
    Handshake,
    Request,
}

pub struct Session {
    pub(crate) session_id: u32,
    state: SessionState,
    pub(crate) tracker: ObjectTracker<TrackableServices, TrackerId>,
    token_manager: SharedTokensManager,
    pub(crate) origin_attributes: Option<OriginAttributes>,
    message_max_time: u32,
    services_names: HashSet<String>,
    pub(crate) context: SharedSessionContext,
    remote_services_manager: SharedRemoteServiceManager,
    pub(crate) shared_state: SharedStateMap,
    pub(crate) session_helper: SessionSupport,
    bytes_received: usize,
    kind: SessionKind,
}

impl Drop for Session {
    fn drop(&mut self) {
        debug!(
            "Dropping Session: received {} bytes, sent {} bytes",
            self.bytes_received,
            self.session_helper.bytes_sent()
        );
        self.close();
        self.kind.end();
    }
}

impl Session {
    // Creates a session for a given message sender.
    #[allow(clippy::too_many_arguments)]
    fn create(
        session_id: u32,
        config: &Config,
        sender: MessageSender,
        token_manager: SharedTokensManager,
        session_context: SharedSessionContext,
        remote_services_manager: SharedRemoteServiceManager,
        shared_state: SharedStateMap,
        state: SessionState,
        origin_attributes: Option<OriginAttributes>,
        kind: SessionKind,
    ) -> Self {
        kind.start();
        remote_services_manager
            .lock()
            .add_upstream_session(session_id, sender.clone());
        let registrar = remote_services_manager.lock().registrar.clone();
        let id_factory = Shared::adopt(IdFactory::new(0));
        let event_map: SharedEventMap = Shared::default();
        Session {
            session_id,
            state,
            tracker: ObjectTracker::default(),
            token_manager,
            origin_attributes,
            message_max_time: config.general.message_max_time,
            services_names: enabled_services(&config, &registrar),
            context: session_context,
            remote_services_manager,
            shared_state,
            session_helper: SessionSupport::new(
                SessionTrackerId::from(session_id, 0),
                sender,
                id_factory,
                event_map,
            ),
            bytes_received: 0,
            kind,
        }
    }

    // Creates a session for a websocket connection: the origin attributes
    // will be populated during the handshake.
    pub fn websocket(
        session_id: u32,
        config: &Config,
        sender: MessageSender,
        token_manager: SharedTokensManager,
        session_context: SharedSessionContext,
        remote_services_manager: SharedRemoteServiceManager,
        shared_state: SharedStateMap,
    ) -> Self {
        Session::create(
            session_id,
            config,
            sender,
            token_manager,
            session_context,
            remote_services_manager,
            shared_state,
            SessionState::Handshake,
            None,
            SessionKind::Ws,
        )
    }

    // Creates a session for a UDS connection: there is no handshake needed,
    // and the origin attributes are hardcoded to a "uds" identity.
    pub fn uds(
        session_id: u32,
        config: &Config,
        sender: MessageSender,
        token_manager: SharedTokensManager,
        session_context: SharedSessionContext,
        remote_services_manager: SharedRemoteServiceManager,
        shared_state: SharedStateMap,
    ) -> Self {
        Session::create(
            session_id,
            config,
            sender,
            token_manager,
            session_context,
            remote_services_manager,
            shared_state,
            SessionState::Request,
            Some(OriginAttributes::new("uds", HashSet::new())),
            SessionKind::Uds,
        )
    }

    pub fn replace_sender(&mut self, sender: MessageSender) {
        // Update this session sender registration with the remote service manager.
        {
            let mut rsm = self.remote_services_manager.lock();
            rsm.remove_upstream_session(self.session_id);
            rsm.add_upstream_session(self.session_id, sender.clone());
        }

        self.session_helper = SessionSupport::new(
            SessionTrackerId::from(self.session_id, 0),
            sender,
            Shared::adopt(IdFactory::new(0)),
            Shared::default(),
        );
    }

    fn abort_connection(&mut self, reason: &str) {
        error!("Aborting connection: {}", reason);
        self.close();
        self.session_helper
            .close_session()
            .expect("Failed to send message");
    }

    fn on_release_object(&mut self, req: &ReleaseObjectRequest, message: &mut BaseMessage) {
        match on_release_object_helper(&self.tracker.get(req.service), req, message) {
            Ok(success) => {
                let response = CoreResponse::ReleaseObject(ReleaseObjectResponse { success });
                message.kind = BaseMessageKind::Response(message.request());
                self.session_helper.serialize_message(&message, &response);
            }
            Err(err) => {
                self.abort_connection(&err);
            }
        }
    }

    fn on_get_service(&mut self, req: &GetServiceRequest, message: &mut BaseMessage) {
        debug!("About to create service `{}`", req.name);

        // Early return if we can check that we don't have this service registered.
        if !self.services_names.contains(&req.name)
            && !self
                .services_names
                .contains(&format!("{}:remote", req.name))
        {
            error!("Could not instanciate service named `{}`", req.name);
            self.session_helper
                .serialize_message(&message, &GetServiceResponse::UnknownService);
            return;
        }

        let s_id = self.tracker.next_id();
        let mut response = on_create_service_helper(self, s_id, req);

        // Check if this is a request to create a remote service.
        if self
            .services_names
            .contains(&format!("{}:remote", req.name))
        {
            info!("About to create remote service `{}`", req.name);
            match RemoteService::create_remote(
                SessionTrackerId::from(self.session_id, s_id),
                &self.origin_attributes.clone().unwrap(),
                self.context.clone(),
                self.remote_services_manager.clone(),
                &req.name,
                &req.fingerprint,
            ) {
                Ok(s) => {
                    let s_item = TrackableServices::Remote(RefCell::new(s));
                    let id = self.tracker.track(s_item);
                    response = CoreResponse::GetService(GetServiceResponse::Success(id));
                }
                Err(err) => {
                    error!("Could not create service {}: {}", req.name, err);
                }
            }
        }

        message.kind = BaseMessageKind::Response(message.request());
        self.session_helper.serialize_message(&message, &response);
    }

    fn on_has_service(&mut self, req: &HasServiceRequest, message: &mut BaseMessage) {
        // Early return if we can check that we don't have this service registered.
        let has_service = self.services_names.contains(&req.name)
            || self
                .services_names
                .contains(&format!("{}:remote", req.name));
        debug!("HasService `{}` is {}", req.name, has_service);

        message.kind = BaseMessageKind::Response(message.request());
        self.session_helper.serialize_message(
            &message,
            &CoreResponse::HasService(HasServiceResponse {
                success: has_service,
            }),
        );
    }

    fn enable_event(&mut self, req: &EnableEventListenerRequest, message: &mut BaseMessage) {
        debug!("{}-{} enable event:{}", req.service, req.object, req.event);
        self.session_helper
            .event_map()
            .lock()
            .insert(EventMapKey::from_ids(req), true);
        let response = CoreResponse::EnableEvent(EnableEventListenerResponse { success: true });
        message.kind = BaseMessageKind::Response(message.request());
        self.session_helper.serialize_message(&message, &response);

        // Relay the event enabling to remote services.
        self.remote_services_manager
            .lock()
            .enable_event(req.service, req.object, req.event);
    }

    fn disable_event(&mut self, req: &DisableEventListenerRequest, message: &mut BaseMessage) {
        debug!("{}-{} disable event:{}", req.service, req.object, req.event);
        let success = self
            .session_helper
            .event_map()
            .lock()
            .remove(&EventMapKey::from_ids(req))
            .unwrap_or(false);

        let response = CoreResponse::DisableEvent(DisableEventListenerResponse { success });
        message.kind = BaseMessageKind::Response(message.request());
        self.session_helper.serialize_message(&message, &response);

        // Relay the event disabling to remote services.
        self.remote_services_manager
            .lock()
            .disable_event(req.service, req.object, req.event);
    }

    fn process_core_message(&mut self, message: &mut BaseMessage) {
        let req: Result<CoreRequest, bincode::Error> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => match req {
                CoreRequest::GetService(ref req) => {
                    self.on_get_service(req, message);
                }
                CoreRequest::HasService(ref req) => {
                    self.on_has_service(req, message);
                }
                CoreRequest::ReleaseObject(ref req) => {
                    self.on_release_object(req, message);
                }
                CoreRequest::EnableEvent(ref req) => {
                    self.enable_event(req, message);
                }
                CoreRequest::DisableEvent(ref req) => {
                    self.disable_event(req, message);
                }
            },
            Err(err) => {
                error!("Unable to process core request: {:?}", err);
            }
        }
    }

    fn process_base_message(&mut self, message: &mut BaseMessage) {
        if message.service == 0 {
            // Special case, this is the Core service.
            self.process_core_message(message);
            return;
        }

        // Retrieve the service from the object tracker, and
        // forward it the payload to decode.
        if let Err(err) = process_base_message_helper(
            &self.tracker.get(message.service),
            &self.session_helper,
            message,
        ) {
            self.abort_connection(&err);
        }
    }

    /// Receives a request from the client.
    fn on_request(&mut self, message: &[u8]) {
        let req: Result<BaseMessage, bincode::Error> = common::deserialize_bincode(message);
        match req {
            Ok(mut msg) => {
                self.process_base_message(&mut msg);
            }
            Err(err) => {
                self.abort_connection(&format!("Unexpected bincode message: {:?}", err));
            }
        }
    }

    /// Receives and process a handshake message.
    fn on_handshake(&mut self, message: &[u8]) {
        let req: Result<SessionHandshake, bincode::Error> = common::deserialize_bincode(message);
        match req {
            Ok(handshake) => {
                info!("Got client handshake");

                // Check that we are presented with a valid token, and store the identity if
                // this is the case.
                if let Some(attr) = self
                    .token_manager
                    .lock()
                    .get_origin_attributes(&handshake.token)
                {
                    self.origin_attributes = Some(attr);
                }

                if self.origin_attributes.is_none() {
                    self.abort_connection(&format!("Invalid token: {}", handshake.token));
                }

                // Everything is fine, send a success SessionAck.
                let resp = SessionAck { success: true };
                self.state = SessionState::Request;
                self.session_helper.send_message(&resp);
            }
            Err(err) => {
                // Unable to get handshake, close the connection.
                self.abort_connection(&format!("Error decoding handshake: {:?}", err));
            }
        }
    }

    // Returns a printable string representing a request.
    // This decodes the protobuf message without running the request.
    fn format_request(&self, message: &[u8]) -> String {
        let req: Result<BaseMessage, bincode::Error> = common::get_bincode().deserialize(message);
        match req {
            Ok(msg) => {
                if msg.service == 0 {
                    // Decode a Core message.
                    let req: Result<CoreRequest, bincode::Error> =
                        common::deserialize_bincode(message);
                    match req {
                        Ok(request) => match request {
                            CoreRequest::GetService(ref req) => format!("GetService {}", req.name),
                            CoreRequest::HasService(ref req) => format!("HasService {}", req.name),
                            CoreRequest::ReleaseObject(ref req) => {
                                format!("ReleaseObject {} on service {}", req.object, req.service)
                            }
                            CoreRequest::EnableEvent(ref req) => format!(
                                "{} EnableEventListener {} on Service {}",
                                req.object, req.event, req.service,
                            ),
                            CoreRequest::DisableEvent(ref req) => format!(
                                "{} DisableEventListener {} on Service {}",
                                req.object, req.event, req.service,
                            ),
                        },
                        Err(err) => format!("Unable to process request: {:?}", err),
                    }
                } else {
                    // Decode a service request, by delegating to the service itself
                    // since the session doesn't know the service-specific message
                    // structure.
                    format_request_helper(
                        &self.tracker.get(msg.service),
                        &self.session_helper,
                        &msg,
                    )
                }
            }
            Err(err) => format!("Unexpected message: {:?}", err),
        }
    }

    pub fn on_message(&mut self, message: &[u8]) {
        debug!("on_message len={}", message.len());
        self.bytes_received += message.len();
        // Measure how long we block the transport thread when processing
        // a message. Warn if we take longer than the predefined threshold
        // in the config.
        let timer = SystemTime::now();
        let is_handshake = self.state == SessionState::Handshake;
        match self.state {
            SessionState::Handshake => {
                self.on_handshake(message);
            }
            SessionState::Request => {
                self.on_request(message);
            }
        }

        match timer.elapsed() {
            Ok(elapsed) => {
                let millis = (elapsed.as_secs() * 1000 + u64::from(elapsed.subsec_millis())) as u32;
                if millis > self.message_max_time {
                    let what = if is_handshake {
                        "Handshake".into()
                    } else {
                        self.format_request(message)
                    };
                    warn!("Processing '{}' took too long: {}ms", what, millis);
                }
            }
            Err(err) => error!("Faled to get message processing time: {:?}", err.duration()),
        }
    }

    pub fn close(&mut self) {
        debug!("Session close");
        self.remote_services_manager
            .lock()
            .remove_upstream_session(self.session_id);
        self.tracker.clear();
    }
}

#[cfg(test)]
mod test {
    use super::Session;
    use super::SessionState;
    use crate::config::Config;
    use bincode::Options;
    use common::core::{
        BaseMessage, CoreRequest, DisableEventListenerRequest, EnableEventListenerRequest,
        SessionHandshake,
    };
    use common::is_event_in_map;
    use common::remote_service::RemoteServiceManager;
    use common::remote_services_registrar::RemoteServicesRegistrar;
    use common::tokens::TokensManager;
    use common::traits::{
        MessageKind, MessageSender, OriginAttributes, SessionContext, SessionTrackerId, Shared,
        StdSender,
    };
    use log::error;
    use serde::Serialize;
    use std::collections::HashSet;
    use std::sync::mpsc;

    pub fn encode_message<T: Serialize>(message: &T) -> Option<Vec<u8>> {
        match common::get_bincode().serialize(message) {
            Ok(val) => Some(val),
            Err(err) => {
                error!("Failed to serialize message: {:?}", err);
                None
            }
        }
    }

    #[test]
    fn test_session() {
        let (sender, receiver) = mpsc::channel();
        let shared_sender = MessageSender::new(Box::new(StdSender::new(&sender)));
        let token_manager = TokensManager::new_shareable();
        let config = Config::test_on_port(9000);
        let registrar = RemoteServicesRegistrar::new("foo.toml", "");
        let shared_rsm = Shared::adopt(RemoteServiceManager::new("./remote", registrar));
        let context = Shared::adopt(SessionContext::default());

        let mut session = Session::websocket(
            0,
            &config,
            shared_sender,
            token_manager.clone(),
            context,
            shared_rsm,
            Shared::<_>::default(),
        );

        let handshake = SessionHandshake {
            token: "test-token".into(),
        };
        // Start with an unknown token.

        let mut buffer = encode_message(&handshake).expect("Failed to encode");
        session.on_message(&buffer);
        let answer = receiver.recv().unwrap();
        assert_eq!(answer, MessageKind::Close);

        // Now register this token.
        token_manager.lock().register(
            "test-token",
            OriginAttributes::new("test-identity", HashSet::new()),
        );
        session.on_message(&buffer);
        let answer = receiver.recv().unwrap();
        assert_eq!(
            answer,
            MessageKind::Data(SessionTrackerId::from(0, 0), vec![1])
        );

        // Re-using the same token fails.
        session.on_message(&buffer);
        let answer = receiver.recv().unwrap();
        assert_eq!(answer, MessageKind::Close);

        // Register the token again, and test a bad version number.
        token_manager.lock().register(
            "test-token",
            OriginAttributes::new("test-identity", HashSet::new()),
        );
        buffer = encode_message(&handshake).expect("Failed to encode");
        session.on_message(&buffer);
        let answer = receiver.recv().unwrap();
        assert_eq!(answer, MessageKind::Close);
    }

    #[test]
    fn test_event_listener() {
        use common::core::BaseMessageKind;

        let (sender, receiver) = mpsc::channel();
        let shared_sender = MessageSender::new(Box::new(StdSender::new(&sender)));
        let token_manager = TokensManager::new_shareable();
        let context = Shared::adopt(SessionContext::default());
        let config = Config::test_on_port(9001);
        let registrar = RemoteServicesRegistrar::new("foo.toml", "");
        let shared_rsm = Shared::adopt(RemoteServiceManager::new("./remote", registrar));

        let mut session = Session::websocket(
            0,
            &config,
            shared_sender,
            token_manager,
            context,
            shared_rsm,
            Shared::<_>::default(),
        );
        session.state = SessionState::Request;
        // Enable event listener
        let mut content = CoreRequest::EnableEvent(EnableEventListenerRequest {
            service: 1,
            object: 1,
            event: 1,
        });
        let mut buffer1 = encode_message(&content).expect("Failed to encode protobuf");

        let mut message = BaseMessage {
            service: 0,
            object: 1,
            kind: BaseMessageKind::Request(1),
            content: buffer1.clone(),
        };
        let mut buffer2 = encode_message(&message).expect("Failed to encode");
        session.on_message(&buffer2);
        receiver.recv().unwrap();
        assert_eq!(
            is_event_in_map(&session.session_helper.event_map(), 1, 1, 1),
            true
        );

        // Disable a unexist event listener
        content = CoreRequest::DisableEvent(DisableEventListenerRequest {
            service: 1,
            object: 1,
            event: 2,
        });
        buffer1 = encode_message(&content).expect("Failed to encode protobuf");

        message = BaseMessage {
            service: 0,
            object: 1,
            kind: BaseMessageKind::Request(3),
            content: buffer1.clone(),
        };
        buffer2 = encode_message(&message).expect("Failed to encode");
        session.on_message(&buffer2);
        receiver.recv().unwrap();
        assert_eq!(
            is_event_in_map(&session.session_helper.event_map(), 1, 1, 1),
            true
        );
        assert_eq!(session.session_helper.event_map().lock().len(), 1);

        // Disable an existed event listener
        content = CoreRequest::DisableEvent(DisableEventListenerRequest {
            service: 1,
            object: 1,
            event: 1,
        });
        buffer1 = encode_message(&content).expect("Failed to encode protobuf");

        message = BaseMessage {
            service: 0,
            object: 1,
            kind: BaseMessageKind::Request(5),
            content: buffer1,
        };
        buffer2 = encode_message(&message).expect("Failed to encode");
        session.on_message(&buffer2);
        receiver.recv().unwrap();
        assert_ne!(
            is_event_in_map(&session.session_helper.event_map(), 1, 1, 1),
            true
        );
        assert_eq!(session.session_helper.event_map().lock().len(), 0);
    }
}
