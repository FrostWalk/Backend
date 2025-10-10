use crate::logging::context::{
    clear_request_context, extract_request_context, set_request_context,
};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse, Transform},
    Error, Result,
};
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Middleware to capture request context for logging
/// This middleware extracts request information and stores it in thread-local storage
/// so that all log entries within the request processing pipeline can include this context
pub struct RequestContextMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestContextMiddleware
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestContextMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestContextMiddlewareService { service }))
    }
}

pub struct RequestContextMiddlewareService<S> {
    service: S,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for RequestContextMiddlewareService<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract request context
        let context = extract_request_context(req.request());

        // Set the context for this thread
        set_request_context(context);

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            // Clear the context after processing
            clear_request_context();

            Ok(res)
        })
    }
}
