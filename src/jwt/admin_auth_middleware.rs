use crate::app_state::AppState;
use crate::common::json_error::ToJsonError;
use crate::database::repositories::admins_repository::AdminRole;
use crate::database::repository_methods_trait::RepositoryMethods;
use crate::jwt::token::decode_token;
use crate::jwt::HEADER;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::error::{ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{http, web, HttpMessage};
use futures_util::future::{ready, LocalBoxFuture};
use futures_util::FutureExt;
use std::rc::Rc;
use std::task::{Context, Poll};

/// Middleware responsible for handling authentication and user information extraction.
pub(crate) struct AdminAuthMiddleware<const N: usize, S> {
    pub(crate) service: Rc<S>,
    pub(crate) allowed_roles: Rc<[AdminRole; N]>,
}

impl<const N: usize, S> Service<ServiceRequest> for AdminAuthMiddleware<N, S>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, actix_web::Error>>;

    /// Polls the readiness of the wrapped service.
    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    /// Handles incoming requests.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Attempt to extract token from cookie or authorization header
        let token = req
            .cookie(HEADER)
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        // If the token is missing, return unauthorized error
        if token.is_none() {
            return Box::pin(ready(Err(ErrorUnauthorized(
                "Token not provided".to_json_error(),
            ))));
        }

        let app_state = req.app_data::<web::Data<AppState>>().unwrap();

        // Decode token and handle errors
        let token = match decode_token(token.unwrap(), app_state.config.jwt_secret().as_bytes()) {
            Ok(id) => id,
            Err(e) => {
                return Box::pin(ready(Err(ErrorUnauthorized(e.to_json_error()))));
            }
        };

        let cloned_app_state = app_state.clone();
        let srv = Rc::clone(&self.service);
        let allowed_roles = self.allowed_roles.clone();

        // Handle user extraction and request processing
        async move {
            let role: AdminRole = match token.rl.try_into() {
                Ok(role) => role,
                Err(_) => return Err(ErrorUnauthorized("Invalid role".to_json_error())),
            };

            if !allowed_roles.contains(&role) {
                return Err(ErrorForbidden(
                    "user does not have the necessary permissions".to_json_error(),
                ));
            }

            let result = cloned_app_state
                .repositories
                .admins
                .get_from_id(token.sub)
                .await;

            let model = match result {
                Ok(m) => m,
                Err(e) => {
                    return Err(ErrorInternalServerError(e.to_json_error()));
                }
            };

            let user = match model {
                None => {
                    return Err(ErrorUnauthorized("user not found".to_json_error()));
                }
                Some(u) => u,
            };

            // Insert user information into request extensions
            req.extensions_mut().insert::<entity::admins::Model>(user);

            // Call the wrapped service to handle the request
            let res = srv.call(req).await?;
            Ok(res)
        }
        .boxed_local()
    }
}
