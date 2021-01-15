#![deny(unused_features)]
#![deny(deprecated)]
#![warn(unused_variables)]
#![warn(unused_imports)]
#![warn(dead_code)]
#![warn(missing_copy_implementations)]

mod channel;
pub mod helper;
mod middleware;
mod server;

pub use actix_web::{
    error::PayloadError, http::header::HeaderMap, web::Payload, HttpMessage, HttpRequest,
    HttpResponse,
};
pub use http; // re-export http crate
pub use server::{new, TestServer};
