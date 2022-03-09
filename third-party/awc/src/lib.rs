//! `awc` is an asynchronous HTTP and WebSocket client library.
//!
//! # `GET` Requests
//! ```no_run
//! # #[actix_rt::main]
//! # async fn main() -> Result<(), awc::error::SendRequestError> {
//! // create client
//! let mut client = awc::Client::default();
//!
//! // construct request
//! let req = client.get("http://www.rust-lang.org")
//!     .insert_header(("User-Agent", "awc/3.0"));
//!
//! // send request and await response
//! let res = req.send().await?;
//! println!("Response: {:?}", res);
//! # Ok(())
//! # }
//! ```
//!
//! # `POST` Requests
//! ## Raw Body
//! ```no_run
//! # #[actix_rt::main]
//! # async fn main() -> Result<(), awc::error::SendRequestError> {
//! let mut client = awc::Client::default();
//! let response = client.post("http://httpbin.org/post")
//!     .send_body("Raw body contents")
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## JSON
//! ```no_run
//! # #[actix_rt::main]
//! # async fn main() -> Result<(), awc::error::SendRequestError> {
//! let request = serde_json::json!({
//!     "lang": "rust",
//!     "body": "json"
//! });
//!
//! let mut client = awc::Client::default();
//! let response = client.post("http://httpbin.org/post")
//!     .send_json(&request)
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## URL Encoded Form
//! ```no_run
//! # #[actix_rt::main]
//! # async fn main() -> Result<(), awc::error::SendRequestError> {
//! let params = [("foo", "bar"), ("baz", "quux")];
//!
//! let mut client = awc::Client::default();
//! let response = client.post("http://httpbin.org/post")
//!     .send_form(&params)
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Response Compression
//! All [official][iana-encodings] and common content encoding codecs are supported, optionally.
//!
//! The `Accept-Encoding` header will automatically be populated with enabled codecs and added to
//! outgoing requests, allowing servers to select their `Content-Encoding` accordingly.
//!
//! Feature flags enable these codecs according to the table below. By default, all `compress-*`
//! features are enabled.
//!
//! | Feature           | Codecs        |
//! | ----------------- | ------------- |
//! | `compress-brotli` | brotli        |
//! | `compress-gzip`   | gzip, deflate |
//! | `compress-zstd`   | zstd          |
//!
//! [iana-encodings]: https://www.iana.org/assignments/http-parameters/http-parameters.xhtml#content-coding
//!
//! # WebSockets
//! ```no_run
//! # #[actix_rt::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use futures_util::{sink::SinkExt as _, stream::StreamExt as _};
//!
//! let (_resp, mut connection) = awc::Client::new()
//!     .ws("ws://echo.websocket.org")
//!     .connect()
//!     .await?;
//!
//! connection
//!     .send(awc::ws::Message::Text("Echo".into()))
//!     .await?;
//!
//! let response = connection.next().await.unwrap()?;
//! assert_eq!(response, awc::ws::Frame::Text("Echo".into()));
//! # Ok(())
//! # }
//! ```

#![deny(rust_2018_idioms, nonstandard_style)]
#![warn(future_incompatible)]
#![allow(
    clippy::type_complexity,
    clippy::borrow_interior_mutable_const,
    clippy::needless_doctest_main
)]
#![doc(html_logo_url = "https://actix.rs/img/logo.png")]
#![doc(html_favicon_url = "https://actix.rs/favicon.ico")]

pub use actix_http::body;

#[cfg(feature = "cookies")]
pub use cookie;

mod any_body;
mod builder;
mod client;
mod connect;
pub mod error;
mod frozen;
pub mod middleware;
mod request;
mod responses;
mod sender;
pub mod test;
pub mod ws;

pub mod http {
    //! Various HTTP related types.

    // TODO: figure out how best to expose http::Error vs actix_http::Error
    pub use actix_http::{
        header, uri, ConnectionType, Error, Method, StatusCode, Uri, Version,
    };
}

pub use self::builder::ClientBuilder;
pub use self::client::{Client, Connector};
pub use self::connect::{BoxConnectorService, BoxedSocket, ConnectRequest, ConnectResponse};
pub use self::frozen::{FrozenClientRequest, FrozenSendBuilder};
pub use self::request::ClientRequest;
#[allow(deprecated)]
pub use self::responses::{ClientResponse, JsonBody, MessageBody, ResponseBody};
pub use self::sender::SendClientRequest;

pub(crate) type BoxError = Box<dyn std::error::Error>;
