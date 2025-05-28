use crate::jwt::admin_auth_middleware::AdminAuthMiddleware;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::{ready, Ready};
use std::rc::Rc;
use crate::database::repositories::admins_repository::AdminRole;


/// Middleware requirement for authentication and authorization.
pub(crate) struct RequireAdmin<const N: usize> {
    allowed_roles: Rc<[AdminRole; N]>,
}

impl<const N: usize> RequireAdmin<N> {
    /// Create a new instance of `RequireAuth` middleware.
    pub(crate) fn allowed_roles(allowed_roles: [AdminRole; N]) -> Self {
        RequireAdmin {
            allowed_roles: Rc::new(allowed_roles),
        }
    }
}

impl<const N: usize, S> Transform<S, ServiceRequest> for RequireAdmin<N>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = AdminAuthMiddleware<N, S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Create a new `AuthMiddleware` using the provided service.
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminAuthMiddleware {
            service: Rc::new(service),
            allowed_roles: self.allowed_roles.clone(),
        }))
    }
}
