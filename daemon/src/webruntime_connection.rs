// Manages the connection with the web runtime to register tokens.

use crate::api_server::SharedWsData;
use crate::tokens::SharedTokensManager;
use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use common::traits::OriginAttributes;
use log::{debug, error};
use serde::Deserialize;
use serde_json;
use std::collections::HashSet;
use std::sync::RwLock;

/// Define our WS actor, holding the tokens manager.
struct WsHandler {
    token_manager: SharedTokensManager,
}

impl Actor for WsHandler {
    type Context = ws::WebsocketContext<Self>;
}

#[derive(Deserialize)]
struct ClientMessage {
    token: String,
    identity: String,
    permissions: Option<Vec<String>>,
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsHandler {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                debug!("Got text frame: {}", text);
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(msg) => {
                        let permissions = match msg.permissions {
                            Some(permissions) => {
                                // Turn the Vec<String> into a HashSet<String>
                                let mut set = HashSet::new();
                                for perm in permissions {
                                    set.insert(perm);
                                }
                                set
                            }
                            None => HashSet::new(),
                        };

                        ctx.text(format!(
                            "{{ \"result\": {} }}",
                            self.token_manager.lock().register(
                                &msg.token,
                                OriginAttributes::new(&msg.identity, permissions)
                            )
                        ));
                    }
                    Err(_) => ctx.close(None),
                }
            }
            Ok(ws::Message::Close(_)) => {
                debug!("Close WS runtime message {:?}", msg);
                ctx.close(None);
            }
            _ => {
                error!("Unexpected WS runtime message {:?}", msg);
                ctx.close(None);
            }
        }
    }
}

pub struct WebRuntimeConnection;

impl WebRuntimeConnection {
    /// When not on Android, use the WS_RUNTIME_TOKEN environment variable.
    #[cfg(not(target_os = "android"))]
    fn runtime_token() -> String {
        ::std::env::var("WS_RUNTIME_TOKEN").unwrap_or_else(|_| "".into())
    }

    /// On Android, use the kaios.services.runtime.token property.
    #[cfg(target_os = "android")]
    fn runtime_token() -> String {
        use std::fs::File;
        use std::io::Read;

        if cfg!(feature = "fake-tokens") {
            ::std::env::var("WS_RUNTIME_TOKEN").unwrap_or_else(|_| "".into())
        } else {
            // First try to load the token from a file, and fallback on the Android property if that fails.
            if let Ok(mut file) = File::open("/data/local/service/api-daemon/.runtime_token") {
                let mut token = String::new();
                if let Ok(_) = file.read_to_string(&mut token) {
                    return token;
                }
            }
            
            "".into()
        }
    }

    pub(crate) async fn runtime_index(
        data: web::Data<RwLock<SharedWsData>>,
        req: HttpRequest,
        stream: web::Payload,
    ) -> Result<HttpResponse, Error> {
        // Check that the url matches the expected one set by Gecko.
        if req.path() != format!("/{}", Self::runtime_token()) {
            return Ok(HttpResponse::BadRequest().finish());
        }
        let data = data
            .read()
            .map_err(|_| HttpResponse::InternalServerError().finish())?;
        let global_context = &data.global_context;

        let resp = ws::start(
            WsHandler {
                token_manager: global_context.tokens_manager.clone(),
            },
            &req,
            stream,
        );
        resp
    }
}

#[cfg(test)]
mod test {
    use crate::api_server;
    use crate::config::Config;
    use crate::global_context::GlobalContext;
    use actix_web::client::{Client, WsClientError};
    use async_std::stream::StreamExt;
    use actix_web_actors::ws::{Message, Frame};
    use std::{thread, time};
    use futures_util::sink::SinkExt;

    async fn test_message(msg: &str, result: Option<&'static str>) {
        let test_res = match result {
            Some(v) => Frame::Text(v.into()),
            None => Frame::Close(None),
        };

        let client = Client::default();
        match client.ws("ws://localhost:8881/runtime").connect().await {
            Ok((_response, mut framed)) => {
                // 1. send our request.
                let _ = framed.send(Message::Text(msg.into())).await;

                // 2. receive the response.
                let item = framed.next().await.unwrap().unwrap();
                assert_eq!(item, test_res);
            }
            Err(err) => panic!("Connecting to /runtime should not fail! {}", err),
        }
    }

    #[actix_rt::test]
    async fn test_webruntime_connection() {
        // Set the environment variable.
        ::std::env::set_var("WS_RUNTIME_TOKEN", "runtime");

        // Create a new ws server.
        thread::spawn(move || {
            let config = Config::test_on_port(8881);
            api_server::start(&GlobalContext::new(&config));
        });

        // Wait for the server to start.
        thread::sleep(time::Duration::from_millis(1000));

        // Fail to connect on the wrong path.
        let client = Client::default();
        match client.ws("ws://localhost:8881/wrong_path").connect().await {
            Err(WsClientError::InvalidResponseStatus(status)) => {
                assert_eq!(status, 400);
            }
            _ => panic!("Connecting to /wrong_runtime should fail"),
        }

        // Now test client connections to the correct url with various inputs.

        // Failure when not sending JSON.
        test_message("Not a JSON message!", None).await;

        // Add a new token -> identity mapping.
        let msg = r#"{ "token": "test-token", "identity": "test-identity" }"#;
        test_message(msg, Some(r#"{ "result": true }"#)).await;

        // Try to add a mapping for the same token again.
        let msg = r#"{ "token": "test-token", "identity": "test-identity-2" }"#;
        test_message(msg, Some(r#"{ "result": false }"#)).await;

        // Add a different token -> identity mapping.
        let msg = r#"{ "token": "test-token-2", "identity": "test-identity-2" }"#;
        test_message(msg, Some(r#"{ "result": true }"#)).await;
    }
}
