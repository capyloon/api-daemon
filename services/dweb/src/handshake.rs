/// Handshake messages
use crate::generated::common::{Peer, PeerAction};
use crate::service::State;
use common::traits::Shared;
use log::{error, info};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;

#[derive(Deserialize, Serialize, Debug)]
struct PairingRequest {
    peer: Peer, // The initiator peer
}

#[derive(Deserialize, Serialize, Debug)]
struct ActionRequest {
    peer: Peer, // The initiator peer
    action: PeerAction,
    offer: String,
}

#[derive(Deserialize, Serialize, Debug)]
enum Request {
    Pairing(PairingRequest),
    Action(ActionRequest),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Status {
    Granted,
    Denied,
    NotConnected,
    InternalError,
}

trait ResponseOut {
    type Out;

    fn status(&self) -> Status;
    fn output(&self) -> Self::Out;
    fn error(status: Status) -> Self;
}

#[derive(Deserialize, Serialize, Debug)]
struct PairingResponse {
    status: Status,
}

impl ResponseOut for PairingResponse {
    type Out = ();

    fn status(&self) -> Status {
        self.status.clone()
    }

    fn output(&self) -> Self::Out {}

    fn error(status: Status) -> Self {
        Self { status }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct ActionResponse {
    status: Status,
    answer: String,
}

impl ResponseOut for ActionResponse {
    type Out = String;

    fn status(&self) -> Status {
        self.status.clone()
    }

    fn output(&self) -> Self::Out {
        self.answer.clone()
    }

    fn error(status: Status) -> Self {
        Self {
            status,
            answer: "".into(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
enum Response {
    Pairing(PairingResponse),
    Action(ActionResponse),
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

    // Handles a client request: calls the ui appropriate UI provider.
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
        let mut provider = match state.lock().get_p2p_provider() {
            Some(proxy) => proxy.clone(),
            None => {
                let response = match request {
                    Request::Pairing(_) => {
                        Response::Pairing(PairingResponse::error(Status::InternalError))
                    }
                    Request::Action(_) => {
                        Response::Action(ActionResponse::error(Status::InternalError))
                    }
                };
                return Self::send_response(stream, response);
            }
        };

        match request {
            Request::Pairing(PairingRequest { peer }) => {
                // Process the call to pair
                if let Ok(result) = provider.hello(peer.clone()).recv() {
                    match result {
                        Ok(false) => {
                            return Self::send_response(
                                stream,
                                Response::Pairing(PairingResponse::error(Status::Denied)),
                            )
                        }
                        Ok(true) => {}
                        Err(_) => {
                            return Self::send_response(
                                stream,
                                Response::Pairing(PairingResponse::error(Status::Denied)),
                            )
                        }
                    }
                } else {
                    return Self::send_response(
                        stream,
                        Response::Pairing(PairingResponse::error(Status::InternalError)),
                    );
                }
            }
            Request::Action(ActionRequest {
                peer,
                action,
                offer,
            }) => {
                // Process the call to provide_answer();
                if let Ok(result) = provider.provide_answer(peer, action, offer).recv() {
                    match result {
                        Ok(answer) => Self::send_response(
                            stream,
                            Response::Action(ActionResponse {
                                status: Status::Granted,
                                answer,
                            }),
                        ),
                        Err(_) => Self::send_response(
                            stream,
                            Response::Action(ActionResponse::error(Status::Denied)),
                        ),
                    }
                } else {
                    return Self::send_response(
                        stream,
                        Response::Action(ActionResponse::error(Status::InternalError)),
                    );
                }
            }
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
}

impl HandshakeClient {
    pub fn new(addr: &SocketAddr) -> Self {
        Self { addr: addr.clone() }
    }

    // Manages a request / response flow.
    fn request<I: Serialize, O: DeserializeOwned + ResponseOut>(
        &self,
        input: I,
    ) -> Result<O::Out, Status> {
        let mut stream = TcpStream::connect(self.addr).map_err(|err| {
            error!("Failed to connect to {:?} : {:?}", self.addr, err);
            Status::NotConnected
        })?;

        let encoded = bincode::serialize(&input).map_err(|_| Status::InternalError)?;
        stream
            .write_all(&encoded)
            .map_err(|_| Status::InternalError)?;

        let response: Result<O, _> = bincode::deserialize_from(stream);

        match response {
            Ok(response) => match response.status() {
                Status::Granted => Ok(response.output()),
                _ => {
                    error!("No anwser to the offer: {:?}", response.status());
                    Err(Status::Denied)
                }
            },
            Err(err) => {
                error!("Error decoding request: {}", err);
                Err(Status::InternalError)
            }
        }
    }

    // Blocking call to send a pairing request.
    pub fn pair_with(&self, peer: Peer) -> Result<(), Status> {
        let request = PairingRequest { peer: peer.clone() };
        self.request::<PairingRequest, PairingResponse>(request)
    }

    // Blocking call to send a webrtc request.
    pub fn get_answer(
        &self,
        peer: Peer,
        action: PeerAction,
        offer: String,
    ) -> Result<String, Status> {
        let request = ActionRequest {
            peer: peer.clone(),
            action: action.clone(),
            offer: offer.clone(),
        };
        self.request::<ActionRequest, ActionResponse>(request)
    }
}
