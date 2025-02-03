use crate::jwt::auth_middleware::AuthMiddleware;
use crate::jwt::role::UserRole;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::{ready, Ready};
use std::rc::Rc;

/// Middleware requirement for authentication and authorization.
pub(crate) struct RequireAuth<const N: usize> {
    allowed_roles: Rc<[UserRole; N]>,
}

impl<const N: usize> RequireAuth<N> {
    /// Create a new instance of `RequireAuth` middleware.
    pub(crate) fn allowed_roles(allowed_roles: [UserRole; N]) -> Self {
        RequireAuth {
            allowed_roles: Rc::new(allowed_roles),
        }
    }
}

impl<const N: usize, S> Transform<S, ServiceRequest> for RequireAuth<N>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<N, S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Create a new `AuthMiddleware` using the provided service.
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            allowed_roles: self.allowed_roles.clone(),
        }))
    }
}
