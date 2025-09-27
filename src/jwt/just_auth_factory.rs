use crate::jwt::auth_middleware::AuthMiddleware;
use crate::models::admin_role::ALL;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::{ready, Ready};
use std::rc::Rc;

pub(crate) struct User {}

impl User {
    /// Define a middleware that checks if the user is authenticated regardless of whether it is admin or student
    pub(crate) fn require_auth() -> Self {
        Self {}
    }
}

// implement Transform *only* for the 4-case
impl<S> Transform<S, ServiceRequest> for User
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<4, S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            require_admin: false,
            service: Rc::new(service),
            authentication_only: true,
            allowed_roles: Rc::new(ALL),
        }))
    }
}
