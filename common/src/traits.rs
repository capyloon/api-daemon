use crate::core::BaseMessage;
use crate::remote_service::SharedRemoteServiceManager;
use actix::Message;
use android_utils::SystemState;
use bincode::Options;
use dyn_clone::DynClone;
use log::{debug, error};
use parking_lot::{Mutex, MutexGuard};
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use thiserror::Error;

pub type DispatcherId = u32;

#[derive(Debug, PartialEq, Message)]
#[rtype(result = "()")]
pub enum MessageKind {
    Data(SessionTrackerId, Vec<u8>),
    ChildDaemonCrash(String, i32, u32), // (service name, exit code, pid)
    Close,
}

pub trait EventIds {
    fn ids(&self) -> (u32, u32, u32);
}

// A composite key to identify an event target.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct EventMapKey {
    service: u32,
    object: u32,
    event: u32,
}

impl EventMapKey {
    pub fn new(service: u32, object: u32, event: u32) -> Self {
        Self {
            service,
            object,
            event,
        }
    }

    pub fn from_ids<T: EventIds>(source: &T) -> Self {
        let (service, object, event) = source.ids();
        Self {
            service,
            object,
            event,
        }
    }
}

pub type SharedEventMap = Shared<HashMap<EventMapKey, bool>>;

#[derive(Debug)]
pub struct IdFactory {
    current: u64,
}

impl IdFactory {
    // Creates a new factory with a base index. The base is used
    // to distinguish id sources and prevent collisions.
    pub fn new(base: u32) -> Self {
        IdFactory {
            current: u64::from(base),
        }
    }

    pub fn next_id(&mut self) -> u64 {
        // The server side generates even request ids.
        self.current += 2;
        self.current
    }
}

pub type SharedIdFactory = Shared<IdFactory>;

// The error type for send_message.
#[derive(Error, Debug)]
pub enum SendMessageError {
    #[error("Failed to send message with StdSender")]
    Std(#[from] ::std::sync::mpsc::SendError<MessageKind>),
    #[error("Failed to send message with ActorSender")]
    Actor(#[from] actix::prelude::SendError<MessageKind>),
}

// Different kind of senders can be used depending on how messages
// need to be delivered.
pub trait MessageEmitter: DynClone + Send {
    fn send_raw_message(&self, message: MessageKind);
    fn close_session(&self) -> Result<(), SendMessageError>;
}

dyn_clone::clone_trait_object!(MessageEmitter);

#[derive(Clone)]
pub struct MessageSender {
    sender: Box<dyn MessageEmitter>,
    bytes_sent: Shared<Cell<usize>>,
}

impl MessageSender {
    pub fn new(sender: Box<dyn MessageEmitter>) -> Self {
        Self {
            sender,
            bytes_sent: Shared::adopt(Cell::new(0)),
        }
    }

    pub fn bytes_sent(&self) -> usize {
        self.bytes_sent.lock().get()
    }

    fn update_bytes_sent(&self, count: usize) {
        let lock = self.bytes_sent.lock();
        lock.set(lock.get() + count);
    }

    /// Sends a raw message directly.
    pub fn send_raw_message(&self, message: MessageKind) {
        if let MessageKind::Data(_id, payload) = &message {
            self.update_bytes_sent(payload.len());
        }
        self.sender.send_raw_message(message)
    }

    pub fn close_session(&self) -> Result<(), SendMessageError> {
        self.sender.close_session()
    }

    /// Sends a buffer.
    pub fn send_buffer(&self, session_id: SessionTrackerId, buffer: Vec<u8>) {
        self.update_bytes_sent(buffer.len());
        self.sender
            .send_raw_message(MessageKind::Data(session_id, buffer))
    }

    /// Sends a bincode serialized message.
    pub fn send_message<T: ::serde::Serialize>(&self, message: &T, session_id: SessionTrackerId) {
        match crate::get_bincode().serialize(message) {
            Ok(buffer) => self.send_buffer(session_id, buffer),
            Err(err) => {
                error!("Failed to serialize message: {:?}", err);
            }
        }
    }

    /// Sends a message with its content
    pub fn serialize_message<S: Serialize>(
        &self,
        base: &BaseMessage,
        content: &S,
        session_id: SessionTrackerId,
    ) {
        match crate::get_bincode().serialize(content) {
            Ok(buffer) => {
                let mut message = BaseMessage::empty_from(&base);
                message.content = buffer.to_vec();
                self.send_message(&message, session_id);
            }
            Err(err) => {
                error!("Failed to serialize message: {:?}", err);
            }
        }
    }
}

// A message sender that uses a std::sync::Sender.
#[derive(Clone)]
pub struct StdSender {
    sender: Sender<MessageKind>,
}

impl StdSender {
    pub fn new(sender: &Sender<MessageKind>) -> Self {
        Self {
            sender: sender.clone(),
        }
    }
}

impl MessageEmitter for StdSender {
    /// Sends a raw message
    fn send_raw_message(&self, message: MessageKind) {
        if let Err(err) = self.sender.send(message) {
            error!("Failed to send message from StdSender! err={:?}", err);
        }
    }

    fn close_session(&self) -> Result<(), SendMessageError> {
        self.sender.send(MessageKind::Close).map_err(|e| e.into())
    }
}

#[derive(Clone)]
pub struct SessionSupport {
    session_id: SessionTrackerId,
    sender: MessageSender,
    id_factory: SharedIdFactory,
    event_map: SharedEventMap,
}

impl SessionSupport {
    pub fn new(
        session_id: SessionTrackerId,
        sender: MessageSender,
        id_factory: SharedIdFactory,
        event_map: SharedEventMap,
    ) -> Self {
        Self {
            session_id,
            sender,
            id_factory,
            event_map,
        }
    }

    pub fn new_with_session(&self, session_id: SessionTrackerId) -> Self {
        Self {
            session_id,
            sender: self.sender.clone(),
            id_factory: self.id_factory(),
            event_map: self.event_map(),
        }
    }

    pub fn session_tracker_id(&self) -> SessionTrackerId {
        self.session_id
    }

    pub fn session_id(&self) -> u32 {
        self.session_id.session()
    }

    /// Sends a bincode serialized message.
    pub fn send_message<T: ::serde::Serialize>(&self, message: &T) {
        self.sender.send_message(message, self.session_id)
    }

    /// Sends a message with its content
    pub fn serialize_message<S: Serialize>(&self, base: &BaseMessage, content: &S) {
        self.sender
            .serialize_message(base, content, self.session_id)
    }

    pub fn close_session(&self) -> Result<(), SendMessageError> {
        self.sender.close_session()
    }

    pub fn id_factory(&self) -> SharedIdFactory {
        debug!("session::id_factory()");
        self.id_factory.clone()
    }

    pub fn event_map(&self) -> SharedEventMap {
        debug!("session::event_map()");
        self.event_map.clone()
    }

    pub fn bytes_sent(&self) -> usize {
        self.sender.bytes_sent()
    }
}

pub type TrackerId = u32; // Simple tracker type.

pub trait SimpleObjectTracker {
    fn id(&self) -> TrackerId {
        0
    }
}

pub trait ObjectTrackerKey: Copy {
    fn first() -> Self;
    fn next(&self) -> Self;
}

impl ObjectTrackerKey for TrackerId {
    fn first() -> Self {
        1 // Starting at 1 because 0 is reserved for "no object" in the protocol.
    }

    fn next(&self) -> Self {
        self + 1
    }
}

// Tracks objects while also keeping track of the session.
#[derive(Hash, PartialEq, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SessionTrackerId {
    session: u32,
    id: u32,
}

impl ObjectTrackerKey for SessionTrackerId {
    fn first() -> Self {
        SessionTrackerId { session: 1, id: 1 }
    }

    fn next(&self) -> Self {
        SessionTrackerId {
            session: self.session,
            id: self.id + 1,
        }
    }
}

impl Eq for SessionTrackerId {}

impl SessionTrackerId {
    pub fn from(session: u32, id: u32) -> Self {
        SessionTrackerId { session, id }
    }

    pub fn service(self) -> u32 {
        self.id
    }

    pub fn session(self) -> u32 {
        self.session
    }

    pub fn set_session(&mut self, session: u32) {
        self.session = session;
    }
}

pub trait ObjectTrackerMethods<T, K: Eq + Hash + ObjectTrackerKey> {
    fn next_id(&self) -> K;
    fn track(&mut self, obj: T) -> K;
    fn untrack(&mut self, id: K) -> bool;
    fn get(&self, id: K) -> Option<&T>;
    fn get_mut(&mut self, id: K) -> Option<&mut T>;
    fn clear(&mut self);
    fn track_with(&mut self, obj: T, key: K);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OriginAttributes {
    identity: String,
    permissions: HashSet<String>,
}

impl OriginAttributes {
    pub fn new(identity: &str, permissions: HashSet<String>) -> Self {
        OriginAttributes {
            identity: identity.into(),
            permissions,
        }
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    pub fn identity(&self) -> String {
        self.identity.clone()
    }
}

// A context that is shared by all the sessions.
#[derive(Debug)]
pub struct SessionContext {
    high_priority_services_count: i32, // Count the number of services that qualify to toggle high priority mode.
    system: SystemState,
}

impl Default for SessionContext {
    fn default() -> Self {
        Self {
            high_priority_services_count: 0,
            system: SystemState::default(),
        }
    }
}

impl SessionContext {
    pub fn enter_high_priority_service(&mut self) {
        self.high_priority_services_count += 1;
        if self.high_priority_services_count == 1 {
            self.system.enter_high_priority();
        }
    }

    pub fn leave_high_priority_service(&mut self) {
        self.high_priority_services_count -= 1;
        match self.high_priority_services_count.cmp(&0) {
            std::cmp::Ordering::Equal => self.system.leave_high_priority(),
            std::cmp::Ordering::Less => error!(
                "High priority services count is now {} !!",
                self.high_priority_services_count
            ),
            _ => {}
        }
    }
}

pub trait StateLogger {
    fn log(&self) {}
}

impl StateLogger for () {}

pub type SharedSessionContext = Shared<SessionContext>;

pub trait Service<S> {
    /// The type of the shared state if multiple instances of this service need to
    /// share access.
    type State: StateLogger;

    /// Called once we have checked that BaseMessage was targetted at this service.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage);

    /// Called when we need a human readable representation of the request.
    fn format_request(&mut self, transport: &SessionSupport, message: &BaseMessage) -> String;

    /// Called when the client side asked us to forcibly untrack an object.
    /// Returns true if successfull, false otherwise.
    fn release_object(&mut self, object_id: u32) -> bool;

    /// Sets the identity of the session user.
    fn create(
        _origin_attributes: &OriginAttributes,
        _context: SharedSessionContext,
        _state: Shared<Self::State>,
        _helper: SessionSupport,
    ) -> Result<S, String> {
        Err("NotImplemented".into())
    }

    /// Sets the identity of the session user.
    fn create_remote(
        _session_id: SessionTrackerId,
        _origin_attributes: &OriginAttributes,
        _context: SharedSessionContext,
        _manager: SharedRemoteServiceManager,
        _service: &str,
        _fingerprint: &str,
    ) -> Result<S, String> {
        Err("NotImplemented".into())
    }

    /// Factory to create a shared state instance.
    /// This will be called only once per process for a given service.
    fn shared_state() -> Shared<Self::State>;
}

// Utility to simplify Arc<Mutex<T>> patterns.
pub struct Shared<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> Shared<T> {
    pub fn lock(&self) -> MutexGuard<T> {
        self.inner.lock()
    }

    pub fn adopt(what: T) -> Self {
        Shared {
            inner: Arc::new(Mutex::new(what)),
        }
    }

    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Default for Shared<T>
where
    T: Default,
{
    fn default() -> Self {
        Shared {
            inner: Arc::new(Mutex::new(T::default())),
        }
    }
}

pub trait CommonResponder {
    fn get_transport(&self) -> &SessionSupport;
    fn get_base_message(&self) -> &BaseMessage;

    fn permission_error(&self, permission: &str, message: &str) {
        let message = BaseMessage::permission_error(permission, message, self.get_base_message());
        let empty: Vec<bool> = vec![];
        self.get_transport().serialize_message(&message, &empty);
    }

    fn maybe_send_permission_error(
        &self,
        origin_attributes: &OriginAttributes,
        permission: &str,
        message: &str,
    ) -> bool {
        let identity = origin_attributes.identity();
        if identity == "uds" {
            // All permissions are granted to uds sessions, so
            // no permission error will ever be sent.
            false
        } else {
            let no_permission = !origin_attributes.has_permission(permission);
            if no_permission {
                error!(
                    "Failed to {}: {} lacks the {} permission.",
                    message, identity, permission
                );
                self.permission_error(permission, message);
            }
            no_permission
        }
    }
}
