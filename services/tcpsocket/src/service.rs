use crate::generated::common::*;
use crate::generated::service::*;

use crate::tcpsocket::{EventType, TcpSocket, TokenMap};

/// Implementation of the tcpsocket service.
use common::core::BaseMessage;
use common::object_tracker::ObjectTracker;
use common::traits::{
    EmptyConfig, EmptyState, ObjectTrackerMethods, OriginAttributes, Service, SessionSupport,
    Shared, SharedServiceState, SharedSessionContext, SimpleObjectTracker, TrackerId,
};

use log::{debug, error, info};
use mio::unix::UnixReady;
use mio::{Event, Events, Poll, Token};
use parking_lot::Mutex;
use std::io::ErrorKind;
use std::sync::Arc;
use std::thread;
use threadpool::ThreadPool;

const MAX_SOCKET_NUM: usize = 1024;
const RECV_BUFF_SIZE: usize = 16384;

use crate::private_traits::PrivateTrait;
type Msocket<'a> = parking_lot::lock_api::MutexGuard<
    'a,
    parking_lot::RawMutex,
    (dyn PrivateTrait + std::marker::Send + 'static),
>;

fn read_on_socket<'a>(socket: &mut Msocket<'a>, buf: &mut [u8], len: usize) {
    // Data
    debug!("Reading data len={}", len);
    let mut v = buf[0..len].to_vec();
    socket.on_event(EventType::Data, v);
    // Draining readiness
    loop {
        match socket.read(buf) {
            Ok(0) => {
                break;
            }
            Ok(len) => {
                v = buf[0..len].to_vec();
                socket.on_event(EventType::Data, v);
            }
            Err(err) => {
                if err.kind() == ErrorKind::WouldBlock {
                    break;
                } else {
                    // TODO: add more specified error process
                    error!("Error draining read buffer: {:?}", err);
                }
            }
        }
    }
}

fn process_event(
    event: &Event,
    is_connected: bool,
    buf: &mut [u8],
    poll: Arc<Poll>,
    token_map: &TokenMap,
    tracker: Arc<Mutex<TcpSocketManagerTrackerType>>,
) -> bool {
    let token = event.token();
    debug!("Event: {:?}", token);
    let tracker_id;
    let responder;

    let mut is_connected = is_connected;

    if let Some((id, res)) = token_map.lock().get(&token) {
        tracker_id = *id;
        responder = res.clone();
    } else {
        error!("can't get tracker from map for Events {:?}", token);
        return is_connected;
    }
    let mut untrack = false;
    let mut tracker_lock = tracker.lock();
    if let Some(obj) = tracker_lock.get_mut(tracker_id) {
        let TcpSocketManagerTrackedObject::TcpSocket(ctxt) = obj;
        {
            let mut socket = ctxt.lock();
            if !socket.is_ready() {
                if socket.set_ready(event.readiness().is_writable()) {
                    is_connected = true;
                } else {
                    // Relase local resource
                    socket.post_close(&poll);
                    token_map.lock().remove(&Token(tracker_id as usize));
                    debug!("map size on release_object: {:?}", token_map.lock().len());
                    untrack = true;
                    responder.reject();
                }
            }
        }
        if is_connected {
            is_connected = false;
            responder.resolve(ctxt.clone());
            return is_connected;
        }
        let socket_state: UnixReady = event.readiness().into();
        if socket_state.is_hup() {
            debug!("Got HUP, peer closed connection.");
            // The socket was closed by the peer.
            let mut socket = ctxt.lock();
            socket.post_close(&poll);
            token_map.lock().remove(&Token(tracker_id as usize));
            debug!(
                "Sockets managed by this event loop after HUP: {}",
                token_map.lock().len()
            );
            untrack = true;
            // Send to client
            socket.on_event(EventType::Close, vec![0]);
        } else if socket_state.is_readable() {
            let mut socket = ctxt.lock();
            match socket.read(buf) {
                Ok(0) => {
                    // Close
                    debug!("Empty read, will close connection.");
                    // Relase local resource
                    socket.post_close(&poll);
                    token_map.lock().remove(&Token(tracker_id as usize));
                    debug!(
                        "Sockets managed by this event loop after Read(0): {}",
                        token_map.lock().len()
                    );
                    untrack = true;
                    // Send to client
                    let v = vec![0];
                    socket.on_event(EventType::Close, v);
                }
                Ok(len) => {
                    read_on_socket(&mut socket, buf, len);
                }
                Err(ref err) => {
                    // Silently ignore WouldBlock since it's not an
                    // unrecoverable error situation.
                    if err.kind() != ErrorKind::WouldBlock {
                        error!("Error reading data: {:?}", err);
                        // Error
                        let v = err.to_string().into_bytes();
                        socket.on_event(EventType::Error, v);
                    }
                }
            }
        } else if socket_state.is_writable() {
            let mut socket = ctxt.lock();
            debug!("Socket is writable");
            socket.drain_queue();
        }
        if untrack {
            info!("Untrack {}", tracker_id);
            tracker_lock.untrack(tracker_id);
        }
    }

    is_connected
}

pub fn start_event_loop(
    poll: Arc<Poll>,
    token_map: TokenMap,
    tracker: Arc<Mutex<TcpSocketManagerTrackerType>>,
) {
    info!(
        "Starting event loop with {} sockets.",
        token_map.lock().len()
    );
    // Not using a thread from the pool here because we never run
    // multiple event loop threads.
    let _ = thread::Builder::new()
        .name("TcpSocketEventLoop".into())
        .spawn(move || {
            let mut buf = [0; RECV_BUFF_SIZE];
            let mut events = Events::with_capacity(MAX_SOCKET_NUM);
            let mut is_connected = false;

            loop {
                match poll.clone().poll(&mut events, None) {
                    Ok(size) => debug!("Polled {} events", size),
                    Err(err) => error!("Error in poll: {:?}", err),
                }

                for event in &events {
                    is_connected = process_event(
                        &event,
                        is_connected,
                        &mut buf,
                        Arc::clone(&poll),
                        &token_map,
                        tracker.clone(),
                    );
                }
                // Used to exit running thread.
                // Usually, it's pending on poll event queue.
                // Flip poll ready to trigger exit poll region.
                // It will break at here since no socket relay on current queue.
                if token_map.lock().len() == 0 {
                    info!("All sockets are closed, exiting the event loop.");
                    break;
                }
            }
        });
}

pub struct TcpSocketService {
    poll: Arc<Poll>,
    tokenmap: TokenMap,
    id: TrackerId,
    tracker: Arc<Mutex<TcpSocketManagerTrackerType>>,
    helper: SessionSupport,
    pool: ThreadPool,
}

impl TcpSocketManager for TcpSocketService {
    fn get_tracker(&mut self) -> Arc<Mutex<TcpSocketManagerTrackerType>> {
        self.tracker.clone()
    }
}

impl SimpleObjectTracker for TcpSocketService {
    fn id(&self) -> TrackerId {
        self.id
    }
}

impl TcpSocketFactoryMethods for TcpSocketService {
    fn open(&mut self, responder: TcpSocketFactoryOpenResponder, addr: SocketAddress) {
        if addr.port <= 0 {
            responder.reject();
            error!("Invalid port number");
            return;
        }
        let token_map = self.tokenmap.clone();
        let poll = self.poll.clone();
        let tracker = self.tracker.clone();
        let id = tracker.lock().next_id();
        let helper = self.helper.clone();
        let event_dispatcher = Shared::adopt(TcpSocketEventDispatcher::from(helper, id));

        self.pool.execute(move || {
            if let Some(socket) = TcpSocket::new(id, addr, event_dispatcher) {
                {
                    let provider = Arc::new(Mutex::new(socket));
                    let mut tracker_lock = tracker.lock();
                    let obj_id =
                        tracker_lock.track(TcpSocketManagerTrackedObject::TcpSocket(provider));
                    token_map
                        .lock()
                        .insert(Token(obj_id as usize), (obj_id, responder));
                    if let Some(obj) = tracker_lock.get_mut(obj_id) {
                        let TcpSocketManagerTrackedObject::TcpSocket(ctxt) = obj;
                        let ctxt = ctxt.lock();
                        ctxt.register(&poll);
                    }
                }
                if token_map.lock().len() == 1 {
                    start_event_loop(poll, token_map, tracker);
                }
            } else {
                responder.reject();
                error!("Failed to create TcpSocket");
            }
        });
    }
}

common::impl_shared_state!(TcpSocketService, EmptyState, EmptyConfig);

impl Service<TcpSocketService> for TcpSocketService {
    fn create(
        _attrs: &OriginAttributes,
        _context: SharedSessionContext,
        helper: SessionSupport,
    ) -> Result<TcpSocketService, String> {
        let tracker = ObjectTracker::default();
        let object_id = tracker.next_id();
        // This is a little trick to create the event dispatcher eariler and used in tcpsocket later on.
        if let Ok(poll) = Poll::new() {
            Ok(TcpSocketService {
                tokenmap: Shared::default(),
                id: object_id,
                tracker: Arc::new(Mutex::new(tracker)),
                poll: Arc::new(poll),
                helper,
                pool: ThreadPool::with_name("TcpSocketService".into(), 5),
            })
        } else {
            error!("Creation of the Poll failed.");
            Err("Failed to create poller".into())
        }
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<TcpSocketManagerFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("TcpSocketService request: {:?}", req),
            Err(err) => format!("Unable to format TcpSocketService request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        // Check which request we received.
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        debug!("releasing object {}", object_id);
        let mut tracker_lock = self.tracker.lock();
        if let Some(obj) = tracker_lock.get_mut(object_id) {
            let TcpSocketManagerTrackedObject::TcpSocket(ctxt) = obj;
            let mut mut_ctxt = ctxt.lock();
            mut_ctxt.close_internal();
            true
        } else {
            false
        }
    }
}

impl Drop for TcpSocketService {
    fn drop(&mut self) {
        debug!(
            "Dropping TCPSocket Service #{}, {} sockets still open",
            self.id,
            self.tokenmap.lock().len()
        );
        // Close all the sockets bound to this session to ensure we clean up
        // all used resources properly and exit the event loop.
        let max_id = self.tracker.lock().next_id();
        // Iterate from 1 since 0 is reserved for internal use.
        for index in 1..max_id {
            self.release_object(index);
            debug!("Current socket count is {}", self.tokenmap.lock().len());
        }
    }
}
