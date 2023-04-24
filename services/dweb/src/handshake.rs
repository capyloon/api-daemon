/// Handshake messages
use crate::generated::common::Peer;
use crate::service::State;
use common::traits::Shared;
use common::JsonValue;
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
struct DialRequest {
    peer: Peer, // The initiator peer
    params: JsonValue,
}

#[derive(Deserialize, Serialize, Debug)]
enum Request {
    Pairing(PairingRequest),
    Action(DialRequest),
}

trait AsRequest {
    fn as_request(self) -> Request;
}

impl AsRequest for PairingRequest {
    fn as_request(self) -> Request {
        Request::Pairing(self)
    }
}

impl AsRequest for DialRequest {
    fn as_request(self) -> Request {
        Request::Action(self)
    }
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
    fn with_status(status: Status) -> Self;
    fn from_response(req: Response) -> Option<Self>
    where
        Self: Sized;
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

    fn with_status(status: Status) -> Self {
        Self { status }
    }

    fn from_response(req: Response) -> Option<Self> {
        if let Response::Pairing(pairing) = req {
            Some(pairing)
        } else {
            None
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct DialResponse {
    status: Status,
    result: JsonValue,
}

impl ResponseOut for DialResponse {
    type Out = JsonValue;

    fn status(&self) -> Status {
        self.status.clone()
    }

    fn output(&self) -> Self::Out {
        self.result.clone()
    }

    fn with_status(status: Status) -> Self {
        Self {
            status,
            result: serde_json::Value::Null.into(),
        }
    }

    fn from_response(req: Response) -> Option<Self> {
        if let Response::Action(action) = req {
            Some(action)
        } else {
            None
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
enum Response {
    Pairing(PairingResponse),
    Action(DialResponse),
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
                        Response::Pairing(PairingResponse::with_status(Status::InternalError))
                    }
                    Request::Action(_) => {
                        Response::Action(DialResponse::with_status(Status::InternalError))
                    }
                };
                return Self::send_response(stream, response);
            }
        };

        match request {
            Request::Pairing(PairingRequest { peer }) => {
                // Process the call to pair
                if let Ok(result) = provider.hello(&peer).recv() {
                    match result {
                        Ok(false) => Self::send_response(
                            stream,
                            Response::Pairing(PairingResponse::with_status(Status::Denied)),
                        ),
                        Ok(true) => {
                            // Create a session with this peer since we accepter the connection.
                            state.lock().create_session(peer);
                            Self::send_response(
                                stream,
                                Response::Pairing(PairingResponse::with_status(Status::Granted)),
                            )
                        }
                        Err(_) => Self::send_response(
                            stream,
                            Response::Pairing(PairingResponse::with_status(Status::Denied)),
                        ),
                    }
                } else {
                    Self::send_response(
                        stream,
                        Response::Pairing(PairingResponse::with_status(Status::InternalError)),
                    )
                }
            }
            Request::Action(DialRequest { peer, params }) => {
                // Process the call to provide_answer();
                if let Ok(result) = provider.on_dialed(&peer, &params).recv() {
                    match result {
                        Ok(result) => Self::send_response(
                            stream,
                            Response::Action(DialResponse {
                                status: Status::Granted,
                                result,
                            }),
                        ),
                        Err(_) => Self::send_response(
                            stream,
                            Response::Action(DialResponse::with_status(Status::Denied)),
                        ),
                    }
                } else {
                    Self::send_response(
                        stream,
                        Response::Action(DialResponse::with_status(Status::InternalError)),
                    )
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
        Self { addr: *addr }
    }

    // Manages a request / response flow.
    fn request<I: Serialize + AsRequest, O: DeserializeOwned + ResponseOut>(
        &self,
        input: I,
    ) -> Result<O::Out, Status> {
        info!("Sending request to {:?}", self.addr);
        let mut stream = TcpStream::connect(self.addr).map_err(|err| {
            error!("Failed to connect to {:?} : {:?}", self.addr, err);
            Status::NotConnected
        })?;

        let encoded = bincode::serialize(&input.as_request()).map_err(|_| Status::InternalError)?;
        stream
            .write_all(&encoded)
            .map_err(|_| Status::InternalError)?;

        let response: Result<Response, _> = bincode::deserialize_from(stream);

        match response {
            Ok(response) => {
                if let Some(resp) = O::from_response(response) {
                    match resp.status() {
                        Status::Granted => Ok(resp.output()),
                        _ => {
                            error!("Request denied: {:?}", resp.status());
                            Err(Status::Denied)
                        }
                    }
                } else {
                    error!("Response is not the expected type");
                    Err(Status::InternalError)
                }
            }
            Err(err) => {
                error!("Error decoding response: {}", err);
                Err(Status::InternalError)
            }
        }
    }

    // Blocking call to send a pairing request.
    pub fn pair_with(&self, peer: Peer) -> Result<(), Status> {
        let request = PairingRequest { peer };
        self.request::<PairingRequest, PairingResponse>(request)
    }

    // Blocking call to send a dial request.
    pub fn dial(&self, peer: Peer, params: JsonValue) -> Result<JsonValue, Status> {
        let request = DialRequest { peer, params };
        self.request::<DialRequest, DialResponse>(request)
    }
}
