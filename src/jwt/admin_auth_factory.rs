use crate::database::repositories::admins_repository::AdminRole;
use crate::jwt::auth_middleware::AuthMiddleware;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::{ready, Ready};
use std::rc::Rc;

pub(crate) struct Admin<const N: usize> {
    allowed_roles: Rc<[AdminRole; N]>,
}

impl<const N: usize> Admin<N> {
    pub(crate) fn require_roles(allowed_roles: [AdminRole; N]) -> Self {
        Self {
            allowed_roles: Rc::new(allowed_roles),
        }
    }
}

impl<const N: usize, S> Transform<S, ServiceRequest> for Admin<N>
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

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            require_admin: true,
            service: Rc::new(service),
            authentication_only: false,
            allowed_roles: self.allowed_roles.clone(),
        }))
    }
}
