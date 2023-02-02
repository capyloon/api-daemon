/// Handshake messages
use crate::generated::common::Peer;
use crate::service::State;
use common::traits::Shared;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;

#[derive(Deserialize, Serialize, Debug)]
struct Request {
    peer: Peer,    // The initiator DID
    offer: String, // ICE offer
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Status {
    Granted,
    Denied,
    NotConnected,
    InternalError,
}

#[derive(Deserialize, Serialize, Debug)]
struct Response {
    status: Status,
    answer: Option<String>,
}

impl Response {
    fn failed(status: Status) -> Self {
        Self {
            status,
            answer: None,
        }
    }
}

pub struct HandshakeHandler {
    addr: SocketAddr,
    state: Shared<State>,
}

impl HandshakeHandler {
    pub fn new(addr: SocketAddr, state: Shared<State>) -> Self {
        Self { addr, state }
    }

    fn send_response(mut stream: TcpStream, response: Response) {
        match bincode::serialize(&response) {
            Ok(data) => {
                if let Err(err) = stream.write_all(&data) {
                    error!("Failed to send response: {}", err);
                }
            }
            Err(err) => error!("Failed to encode response: {}", err),
        }
    }

    // Handles a client request: calls the ui provider.
    fn handle_client(stream: TcpStream, state: Shared<State>) {
        let reader = BufReader::new(stream.try_clone().unwrap());

        let request: Request = match bincode::deserialize_from(reader) {
            Ok(res) => res,
            Err(err) => {
                error!("Error decoding request: {}", err);
                return;
            }
        };

        // Check that the provider is available.
        let mut provider = match state.lock().get_webrtc_provider() {
            Some(proxy) => proxy.clone(),
            None => return Self::send_response(stream, Response::failed(Status::InternalError)),
        };

        // Process the call to hello()
        if let Ok(result) = provider.hello(request.peer.clone()).recv() {
            match result {
                Ok(false) => return Self::send_response(stream, Response::failed(Status::Denied)),
                Ok(true) => {}
                Err(_) => return Self::send_response(stream, Response::failed(Status::Denied)),
            }
        } else {
            return Self::send_response(stream, Response::failed(Status::InternalError));
        }

        // Process the call to provide_answer();
        if let Ok(result) = provider.provide_answer(request.peer, request.offer).recv() {
            match result {
                Ok(answer) => Self::send_response(
                    stream,
                    Response {
                        status: Status::Granted,
                        answer: Some(answer),
                    },
                ),
                Err(_) => Self::send_response(stream, Response::failed(Status::Denied)),
            }
        } else {
            return Self::send_response(stream, Response::failed(Status::InternalError));
        }
    }

    pub fn start(&self) -> Result<(), ()> {
        let addr = self.addr;
        let state = self.state.clone();
        let _handle = thread::Builder::new()
            .name("handshake listener".into())
            .spawn(move || {
                match TcpListener::bind(addr) {
                    Ok(listener) => {
                        for stream in listener.incoming() {
                            let state2 = state.clone();
                            let _ = thread::Builder::new()
                                .name("handshake session".into())
                                .spawn(move || {
                                    if let Ok(stream) = stream {
                                        Self::handle_client(stream, state2);
                                    }
                                });
                        }
                    }
                    Err(err) => error!("Failed to bind handshake socket: {}", err),
                }
                info!("Thread handshake listener stopped")
            })
            .map_err(|_| ())?;
        Ok(())
    }
}

pub struct HandshakeClient {
    addr: SocketAddr,
    peer: Peer,
}

impl HandshakeClient {
    pub fn new(addr: &SocketAddr, peer: Peer) -> Self {
        Self {
            addr: addr.clone(),
            peer,
        }
    }

    // Do a blocking call to send the offer.
    pub fn connect(&self, offer: &str) -> Result<String, Status> {
        let mut stream = TcpStream::connect(self.addr).map_err(|err| {
            error!("Failed to connect to {:?} : {:?}", self.addr, err);
            Status::NotConnected
        })?;

        let request = Request {
            peer: self.peer.clone(),
            offer: offer.to_owned(),
        };

        let encoded = bincode::serialize(&request).map_err(|_| Status::InternalError)?;
        stream
            .write_all(&encoded)
            .map_err(|_| Status::InternalError)?;

        let response: Result<Response, _> = bincode::deserialize_from(stream);

        match response {
            Ok(response) => match response.status {
                Status::Granted => Ok(response.answer.unwrap_or_default()),
                _ => {
                    error!("No anwser to the offer: {:?}", response.status);
                    Err(Status::Denied)
                }
            },
            Err(err) => {
                error!("Error decoding request: {}", err);
                Err(Status::InternalError)
            }
        }
    }
}
