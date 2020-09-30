use crate::generated::common::*;
use crate::private_traits::*;

use common::traits::{Shared, SimpleObjectTracker, TrackerId};
use log::{debug, error, info};
use mio::net::TcpStream;
use mio::unix::UnixReady;
use mio::{Poll, PollOpt, Ready, Token};
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Result, Write};
use std::net::{Shutdown, ToSocketAddrs};
use std::time::SystemTime;

pub type TokenMap = Shared<HashMap<Token, (TrackerId, TcpSocketFactoryOpenResponder)>>;

#[derive(Debug, Copy, Clone)]
pub enum EventType {
    Close = 0,
    Data = 1,
    Drain = 2,
    Error = 3,
}

pub struct TcpSocket {
    id: TrackerId,
    client: TcpStream,
    ready: Cell<bool>,
    closed: bool,
    // Each item in the queue is a (request_id, data) tuple.
    send_queue: Shared<VecDeque<(u64, Vec<u8>)>>,
    event_dispatcher: Shared<TcpSocketEventDispatcher>,
}

impl TcpSocket {
    pub fn new(
        id: TrackerId,
        addr: SocketAddress,
        event_dispatcher: Shared<TcpSocketEventDispatcher>,
    ) -> Option<Self> {
        info!("TcpSocket create new! {:?}", addr);
        match format!("{}:{}", addr.host, addr.port).to_socket_addrs() {
            Ok(mut addr_iter) => match TcpStream::connect(&addr_iter.next().unwrap()) {
                Ok(socket) => Some(TcpSocket {
                    id,
                    client: socket,
                    ready: Cell::new(false),
                    send_queue: Shared::default(),
                    closed: false,
                    event_dispatcher,
                }),
                Err(err) => {
                    error!("Socket connection failed: {:?}", err);
                    None
                }
            },

            Err(err) => {
                error!("Invalid ip address: {}", err);
                None
            }
        }
    }

    // Tries to send data, and returns how much data has been written.
    fn do_send(&mut self, request_id: u64, data: &[u8]) -> usize {
        // Send data
        let timer = SystemTime::now();
        let data = &data;
        debug!(
            "Sending {} bytes, {} data {:?}",
            data.len(),
            request_id,
            data
        );
        match self.client.write(data) {
            Ok(len) => {
                debug!("{} bytes written", len);
                if data.len() == len {
                } else {
                    // We didn't send the whole data, make sure we queue the remaining part
                    // for later sending, and only then send the response back.
                    return len;
                }
            }
            Err(err) => {
                error!("Error in send: {:?}", err);
                if let ErrorKind::WouldBlock = err.kind() {
                    // Don't send a response now:
                    // The writable state of the socket will trigger a drain_queue() call
                    // which then lead to sending the data again.
                    return 0;
                } else {
                    // Other error, directly return an error to the caller.
                }
            }
        }

        // Debug sending elapsed
        let elapsed = timer.elapsed().unwrap();
        let millis = elapsed.as_secs() * 1000 + u64::from(elapsed.subsec_millis());
        debug!("Sending socket data took {}ms", millis);

        // Everything went fine, nothing more to send.
        data.len()
    }
}

impl PrivateTrait for TcpSocket {
    // Use to register socket handler to polling event queue
    fn register(&self, poll: &Poll) {
        debug!("Registering socket {}", self.id);
        match poll.register(
            &self.client,
            Token(self.id as usize),
            Ready::readable() | Ready::writable() | UnixReady::hup(),
            PollOpt::edge(),
        ) {
            Ok(_) => {}
            Err(err) => {
                error!("Registration with the eventloop failed: {:?}", err);
            }
        }
    }

    fn close_internal(&mut self) {
        debug!("Closing socket {}", self.id);
        if self.closed {
            return;
        }

        match self.client.shutdown(Shutdown::Both) {
            Ok(_) => {
                debug!("Socket shutdown success");
                self.closed = true;
            }
            Err(err) => {
                error!("Socket shutdown error: {:?}", err);
            }
        }
    }

    fn post_close(&mut self, poll: &Poll) {
        match poll.deregister(&self.client) {
            Ok(_) => debug!("Deregister of tcpsocket {} successful", self.id),
            Err(err) => error!(
                "Failed to deregister socket {} from the event loop {:?}",
                self.id, err
            ),
        }
    }

    fn is_ready(&self) -> bool {
        self.ready.get()
    }

    // Reply tcpsocket open response with connecting status.
    fn set_ready(&self, status: bool) -> bool {
        debug!("Socket set_ready: current status is {}", status);
        let mut ret = false;
        if self.is_ready() {
            return true;
        }

        match self.client.take_error() {
            Ok(socket_error) => match socket_error {
                None => ret = status,
                Some(err) => {
                    debug!("Socket connection error: {}", err);
                    ret = false;
                }
            },
            Err(err) => error!("Failed to retrieve erro state from socket: {:?}", err),
        }
        self.ready.set(true);
        ret
    }

    fn on_event(&self, etype: EventType, data: Vec<u8>) {
        // Send event to listened client
        debug!("Sending back socket event {:?} len={}", etype, data.len());
        match etype {
            EventType::Data => {
                debug!("Dispatch data event");
                let event_dispatcher_lock = self.event_dispatcher.lock();
                event_dispatcher_lock.dispatch_data(data);
            }
            EventType::Error => match String::from_utf8(data) {
                Ok(data) => {
                    info!("Dispatch error event");
                    let event_dispatcher_lock = self.event_dispatcher.lock();
                    event_dispatcher_lock.dispatch_error(data);
                }
                Err(err) => {
                    debug!("Failed to get error string: {:?}", err);
                }
            },
            EventType::Close => {
                info!("Dispatch close event");
                let event_dispatcher_lock = self.event_dispatcher.lock();
                event_dispatcher_lock.dispatch_close();
            }
            _ => {
                error!("Unsupported event type: {:?}", etype);
                // Nothing event to dispatch.
            }
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.client.read(buf)
    }

    fn drain_queue(&mut self) {
        let send_queue = self.send_queue.clone();
        let mut send_queue_lock = send_queue.lock();
        debug!(
            "drain_queue for socket {}, empty: {}",
            self.id,
            send_queue_lock.is_empty()
        );
        while !send_queue_lock.is_empty() {
            let (request_id, mut data) = send_queue_lock.pop_front().unwrap();
            let size = data.len();
            let written = self.do_send(request_id, &data);

            if written != size {
                // We could not send the full packet, queue a partial one.
                if written == 0 {
                    // Just queue the full message back.
                    send_queue_lock.push_front((request_id, data));
                } else {
                    // Queue a partial message.
                    send_queue_lock.push_front((request_id, data.drain(written..).collect()))
                }
                // Exit the loop.
                break;
            }
        }
    }

    fn send_queue(&mut self, request_id: u64, data: Vec<u8>) {
        debug!("send for socket {}", self.id);
        let mut send_queue = self.send_queue.lock();
        send_queue.push_back((request_id, data));
    }
}

impl SimpleObjectTracker for TcpSocket {
    fn id(&self) -> TrackerId {
        self.id
    }
}

impl TcpSocketMethods for TcpSocket {
    fn close(&mut self, responder: &TcpSocketCloseResponder) {
        info!("Close the socket");
        self.close_internal();
        responder.resolve();
    }

    fn resume(&mut self, responder: &TcpSocketResumeResponder) {
        info!("Resume the connection");
        responder.resolve();
    }

    fn send(&mut self, responder: &TcpSocketSendResponder, data: Vec<u8>) {
        info!("Send data");
        let request = responder.base_message.response();
        self.send_queue(request, data);
        // TODO, use thread pool.
        self.drain_queue();
        responder.resolve(true);
    }

    fn suspend(&mut self, responder: &TcpSocketSuspendResponder) {
        info!("Suspend the connection");
        responder.resolve();
    }
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        debug!(
            "Dropping TcpSocket obj #{} (closed: {})",
            self.id, self.closed
        );
        // If the socket was not closed yet, shut it down now.
        if !self.closed {
            self.close_internal();
        }
    }
}
