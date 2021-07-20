//! Middleware for setting a Cache-Control: no-cache header to 4xx responses.
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{HeaderValue, CACHE_CONTROL};
use actix_web::Error;
use futures_util::future::{ok, FutureExt, LocalBoxFuture, Ready};
use std::task::{Context, Poll};

#[derive(Clone, Default)]
pub(crate) struct NoCacheForErrors {}

impl<S, B> Transform<S> for NoCacheForErrors
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = NoCacheMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(NoCacheMiddleware { service })
    }
}

pub(crate) struct NoCacheMiddleware<S> {
    service: S,
}

impl<S, B> Service for NoCacheMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    #[allow(clippy::borrow_interior_mutable_const)]
    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        async move {
            let mut res = fut.await?;

            if res.status().is_client_error() && !res.headers().contains_key(CACHE_CONTROL) {
                res.headers_mut()
                    .insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
            }

            Ok(res)
        }
        .boxed_local()
    }
}
