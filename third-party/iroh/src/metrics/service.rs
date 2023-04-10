use std::{io, net::SocketAddr, pin::Pin};

use futures::Future;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Error, Request, Response, Server,
};

use tracing::info;

use super::core::CORE;

/// Start a HTTP server to report metrics.
pub async fn run(metrics_addr: SocketAddr) -> Result<(), Error> {
    info!("Starting metrics server on {metrics_addr}");
    Server::bind(&metrics_addr)
        .serve(make_service_fn(move |_conn| async move {
            let handler = make_handler();
            Ok::<_, io::Error>(service_fn(handler))
        }))
        .await
}

/// This function returns an HTTP handler fn that will respond with the
/// OpenMetrics encoding of our metrics.
fn make_handler(
) -> impl Fn(Request<Body>) -> Pin<Box<dyn Future<Output = io::Result<Response<Body>>> + Send>> {
    // This closure accepts a request and responds with the OpenMetrics encoding of our metrics.
    move |_req: Request<Body>| {
        Box::pin(async move {
            CORE.encode()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                .map(|r| {
                    let body = Body::from(r);
                    Response::builder()
                        .header(
                            hyper::header::CONTENT_TYPE,
                            "application/openmetrics-text; version=1.0.0; charset=utf-8",
                        )
                        .body(body)
                        .expect("Failed to build response")
                })
        })
    }
}
