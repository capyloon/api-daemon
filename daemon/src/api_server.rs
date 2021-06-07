/// Actix WebSocket and HTTP server
use crate::global_context::GlobalContext;
use crate::session::Session;
use actix::{Actor, Addr, AsyncContext, Handler, StreamHandler};
use actix_cors::Cors;
use actix_http::ws::Codec;
use actix_web::http::header;
use actix_web::middleware::{Compress, Logger};
use actix_web::web::{Bytes, Data};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws::{self, WebsocketContext};
use async_std::fs::File;
use common::traits::{
    IdFactory, MessageEmitter, MessageKind, MessageSender, SendMessageError, Shared,
};
use futures_core::{
    future::Future,
    ready,
    stream::Stream,
    task::{Context, Poll},
};
use futures_util::io::AsyncReadExt;
use log::{debug, error};
use parking_lot::Mutex;
use vhost_server::vhost_handler::{maybe_not_modified, vhost, AppData};

// When telemetry is enabled, we need to get a handle to the
// event sender, but the type is not available when the telemetry
// feature is disabled.
#[cfg(feature = "device-telemetry")]
pub(crate) type TelemetrySender = telemetry::KillEventSender;

#[cfg(not(feature = "device-telemetry"))]
pub(crate) type TelemetrySender = ();

async fn etag_for_file(file: &File) -> String {
    if let Ok(metadata) = file.metadata().await {
        match metadata.modified().map(|modified| {
            modified
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Modified is earlier than time::UNIX_EPOCH!")
        }) {
            Ok(modified) => format!(
                "W/\"{}.{}-{}\"",
                modified.as_secs(),
                modified.subsec_nanos(),
                metadata.len()
            ),
            _ => format!("W/\"{}\"", metadata.len()),
        }
    } else {
        String::new()
    }
}

#[derive(Clone)]
struct ActorSender {
    sender: Addr<WsHandler>,
}

impl MessageEmitter for ActorSender {
    /// Sends a raw message
    fn send_raw_message(&self, message: MessageKind) {
        if let Err(err) = self.sender.try_send(message) {
            error!("Failed to send message from ActorSender! err={:?}", err);
        }
    }

    fn close_session(&self) -> Result<(), SendMessageError> {
        self.sender
            .try_send(MessageKind::Close)
            .map_err(|e| e.into())
    }
}

/// Define our WS actor, keeping track of the session.
struct WsHandler {
    session: Session,
    #[allow(dead_code)]
    telemetry: TelemetrySender,
}

impl Actor for WsHandler {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Create an ActorSender with our address and use it to replace
        // the session sender.
        self.session
            .replace_sender(MessageSender::new(Box::new(ActorSender {
                sender: ctx.address(),
            })));
    }
}

// Handler for our messages.
impl Handler<MessageKind> for WsHandler {
    type Result = ();

    fn handle(&mut self, msg: MessageKind, ctx: &mut Self::Context) {
        match msg {
            MessageKind::Data(_, val) => ctx.binary(val),
            MessageKind::ChildDaemonCrash(name, exit_code, pid) => {
                error!(
                    "Child daemon `{}` (pid {}) died with exit code {}, closing websocket connection",
                    name, pid, exit_code
                );

                #[cfg(feature = "device-telemetry")]
                self.telemetry
                    .send(&format!("child-{}", name), exit_code, pid);

                ctx.close(None);
            }
            MessageKind::Close => ctx.close(None),
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsHandler {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Binary(bin)) => {
                // Relay the message to the session.
                self.session.on_message(&bin);
            }
            Ok(ws::Message::Close(_)) => {
                debug!("Close WS client message {:?}", msg);
                self.session.close();
                ctx.close(None);
            }
            _ => {
                error!("Unexpected WS client message {:?}", msg);
                self.session.close();
                ctx.close(None);
            }
        }
    }
}

// #[derive(Clone)]
pub struct SharedWsData {
    pub global_context: GlobalContext,
    session_id_factory: Mutex<IdFactory>,
    telemetry: TelemetrySender,
}

// A dummy message sender used when we initially create the session.
// It is replaced by the real one once the actor starts.
#[derive(Clone)]
struct DummySender {}

impl MessageEmitter for DummySender {
    fn send_raw_message(&self, _message: MessageKind) {}
    fn close_session(&self) -> Result<(), SendMessageError> {
        Ok(())
    }
}

// Starts a WS session.
async fn ws_index(
    data: Data<SharedWsData>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let global_context = &data.global_context;

    let session = Session::websocket(
        data.session_id_factory.lock().next_id() as u32,
        &global_context.config,
        MessageSender::new(Box::new(DummySender {})),
        global_context.tokens_manager.clone(),
        global_context.session_context.clone(),
        global_context.remote_service_manager.clone(),
        global_context.service_state(),
    );

    let mut res = ws::handshake(&req)?;
    // Use a max size of 10M for messages.
    let codec = Codec::new().max_size(1_000_000);
    Ok(res.streaming(WebsocketContext::with_codec(
        WsHandler {
            session,
            telemetry: data.telemetry,
        },
        stream,
        codec,
    )))
}

// Returns the File and whether this is the gzip version.
async fn open_file(path: &str, gzip: bool) -> Result<(File, bool), ::std::io::Error> {
    // First test if we have a gzipped version.
    if gzip {
        let file = File::open(path.to_owned() + ".gz").await;
        if file.is_ok() {
            return Ok((file.unwrap(), true));
        }
    }

    File::open(path).await.map(|file| (file, false))
}

const CHUNK_SIZE: usize = 16 * 1024;

struct ChunkedFile {
    reader: File, //BufReader<File>,
}

impl ChunkedFile {
    fn new(file: File) -> Self {
        Self {
            reader: file, //BufReader::with_capacity(CHUNK_SIZE, file),
        }
    }
}

use std::pin::Pin;
impl Stream for ChunkedFile {
    type Item = Result<Bytes, ()>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut buffer: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];

        let read = ready!(Future::poll(
            Pin::new(&mut self.reader.read(&mut buffer)),
            cx
        ))
        .map_err(|_| ())?;

        if read == 0 {
            // We reached EOF
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Self::Item::Ok(Bytes::copy_from_slice(
                &buffer[0..read],
            ))))
        }
    }
}

async fn http_index(data: Data<SharedWsData>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut full_path = data.global_context.config.http.root_path.clone();
    full_path.push_str(req.path());

    let gzip_support = match req
        .headers()
        .get(::actix_web::http::header::ACCEPT_ENCODING)
    {
        Some(header_value) => match header_value.to_str() {
            Ok(value) => {
                let values: Vec<_> = value.split(',').map(|e| e.trim()).collect();
                values.into_iter().any(|encoding| encoding == "gzip")
            }
            Err(_) => false,
        },
        None => false,
    };

    match open_file(&full_path, gzip_support).await {
        Ok((mut file, gzipped)) => {
            // Send the file as a byte stream.
            let content_length = file.metadata().await?.len();
            let content_type = mime_guess::from_path(req.path()).first_or_octet_stream();

            let etag = etag_for_file(&file).await;
            let if_none_match = req.headers().get(header::IF_NONE_MATCH);
            if let Some(response) = maybe_not_modified(if_none_match, &etag, &content_type, None) {
                return Ok(response);
            }

            let mut ok = HttpResponse::Ok();
            let builder = ok
                .header(header::ETAG, etag)
                .header(header::CONTENT_LENGTH, content_length)
                .header(header::CONTENT_TYPE, content_type);

            if gzipped {
                builder.header(header::CONTENT_ENCODING, "gzip");
            }

            let response = if content_length <= CHUNK_SIZE as _ {
                // If the file is small enough, read it all and send it as body.
                let mut content = Vec::with_capacity(CHUNK_SIZE);
                file.read_to_end(&mut content).await?;
                builder.body(Bytes::from(content))
            } else {
                // Otherwise wrap the file in a chunked stream.
                builder.streaming(ChunkedFile::new(file))
            };

            Ok(response)
        }
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

// A Guard that checks if a request should be handled by the vhost server.
// This checks if the Host header and path are of the pattern:
// xxxxx.localhost:$port or localhost:$port/redirect/xxxxx,
// or if running on the default http port (80),
// xxxxx.localhost or localhost/redirect/xxx.
struct VhostChecker {
    check: String,
    redirect: String,
}

impl VhostChecker {
    fn new(port: u16) -> Self {
        if port != 80 {
            Self {
                check: format!("localhost:{}", port),
                redirect: "redirect".to_owned(),
            }
        } else {
            Self {
                check: "localhost".to_owned(),
                redirect: "redirect".to_owned(),
            }
        }
    }
}

impl actix_web::guard::Guard for VhostChecker {
    fn check(&self, request: &actix_web::dev::RequestHead) -> bool {
        if let Some(host) = request.headers().get("Host") {
            let parts: Vec<&str> = host.to_str().unwrap_or(&"").split('.').collect();
            if parts.len() == 1 && parts[0] == self.check {
                let path = request.uri.path();
                let paths: Vec<&str> = path.split('/').collect();
                if paths.len() > 2 && paths[1] == self.redirect {
                    return true;
                }
            }
            parts.len() == 2 && parts[1] == self.check
        } else {
            false
        }
    }
}

pub fn start(
    global_context: &GlobalContext,
    vhost_data: Shared<AppData>,
    telemetry: TelemetrySender,
) {
    let config = global_context.config.clone();
    let port = config.general.port;
    let addr = format!("{}:{}", config.general.host, port);

    let sys = actix_rt::System::new("ws-server");
    let shared_data = Data::new(SharedWsData {
        global_context: global_context.clone(),
        session_id_factory: Mutex::new(IdFactory::new(0)),
        telemetry,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(shared_data.clone())
            .app_data(vhost_data.clone())
            .wrap(Logger::new("\"%r\" %{Host}i %s %b %D")) // Custom log to display the vhost
            .wrap(Cors::default().allow_any_origin().send_wildcard())
            .wrap(Compress::default())
            .service(
                web::scope("/")
                    .guard(VhostChecker::new(port))
                    .data(vhost_data.clone())
                    .route("*", web::post().to(HttpResponse::MethodNotAllowed))
                    .route("/{filename:.*}", web::get().to(vhost)),
            )
            .service(
                web::scope("/")
                    .route("*", web::post().to(HttpResponse::MethodNotAllowed))
                    .route("/ws", web::get().to(ws_index))
                    .route("/*", web::get().to(http_index)),
            )
    })
    .keep_alive(60) // To prevent slow request timeout 408 errors when under load
    .bind(addr)
    .expect("Failed to bind to actix http")
    .disable_signals() // For now, since that's causing issues with Ctrl-C
    .run();

    let _ = sys.run();
}

#[cfg(test)]
mod test {
    use crate::api_server;
    use crate::config::Config;
    use crate::global_context::GlobalContext;
    use common::traits::Shared;
    use reqwest::header::{CONTENT_ENCODING, CONTENT_TYPE};
    use reqwest::StatusCode;
    use std::net::TcpStream;
    use std::{thread, time};
    use vhost_server::vhost_handler::AppData;

    fn find_available_port(start_at: u16) -> u16 {
        loop {
            let port = start_at + (rand::random::<u16>() % 10000);
            if TcpStream::connect(format!("127.0.0.1:{}", port)).is_err() {
                return port;
            }
        }
    }

    fn start_server() -> u16 {
        let port = find_available_port(8000);
        // Create a new ws server.
        thread::spawn(move || {
            api_server::start(
                &GlobalContext::new(&Config::test_on_port(port)),
                Shared::adopt(AppData::default()),
                (),
            );
        });

        // Wait for the server to start.
        thread::sleep(time::Duration::from_millis(1000));
        port
    }

    #[test]
    fn test_http_post_request() {
        let port = start_server();

        // Check that POST requests return a BadRequest status
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(&format!("http://127.0.0.1:{}/test", port))
            .send()
            .unwrap();

        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn test_not_found_http_get_request() {
        let port = start_server();

        // Check that GET requests return a NotFound status
        let resp =
            reqwest::blocking::get(&format!("http://127.0.0.1:{}/test/not_here", port)).unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_valid_http_get_request() {
        let port = start_server();

        // Check that GET requests return a ok status
        let resp =
            reqwest::blocking::get(&format!("http://127.0.0.1:{}/core/index.js", port)).unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        {
            let content_type = resp.headers().get(CONTENT_TYPE).unwrap();
            assert_eq!(content_type.as_bytes(), b"application/javascript");
        }
        dbg!(resp.headers());
        assert_eq!(resp.headers()["content-length"], "21");
        assert_eq!(resp.text().unwrap(), r#"console.log("Test!");"#);
    }

    #[test]
    fn test_octet_stream_http_get_request() {
        let port = start_server();

        // Check that GET requests return a ok status
        let resp =
            reqwest::blocking::get(&format!("http://127.0.0.1:{}/core/data.dat", port)).unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        {
            let content_type = resp.headers().get(CONTENT_TYPE).unwrap();
            assert_eq!(content_type.as_bytes(), b"application/octet-stream");
        }

        assert_eq!(resp.headers()["content-length"], "0");
    }

    #[test]
    fn test_gzip_http_get_request() {
        let port = start_server();

        // Check that GET requests return a ok status with a gzip ContentEncoding
        let client = reqwest::blocking::Client::builder().build().unwrap();

        let resp = client
            .get(&format!("http://127.0.0.1:{}/core/data.dat", port))
            .header("Accept-Encoding", "gzip")
            .send()
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let headers = resp.headers();
        {
            let content_type = headers.get(CONTENT_TYPE).unwrap();
            assert_eq!(content_type.as_bytes(), b"application/octet-stream");
        }
        {
            let mut content_encodings = headers
                .get(CONTENT_ENCODING)
                .unwrap()
                .to_str()
                .unwrap()
                .split(',');
            assert!(content_encodings.any(|e| e == "gzip"));
        }

        assert_eq!(resp.headers()["content-length"], "29");
    }
}
