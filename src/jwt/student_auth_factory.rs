use crate::jwt::auth_middleware::AuthMiddleware;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::{ready, Ready};
use std::rc::Rc;

/// Middleware requirement for authentication and authorization.
pub(crate) struct RequireStudent {}

impl RequireStudent {
    /// Create a new instance of `RequireAuth` middleware.
    pub(crate) fn require_auth() -> Self {
        Self {}
    }
}

impl<S> Transform<S, ServiceRequest> for RequireStudent
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<0, S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Create a new `AuthMiddleware` using the provided service.
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            allowed_roles: Rc::new([]),
            require_admin: false,
        }))
    }
}
