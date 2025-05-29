use crate::app_state::AppState;
use crate::common::json_error::ToJsonError;
use crate::database::repositories::admins_repository::AdminRole;
use crate::database::repository_methods_trait::RepositoryMethods;
use crate::jwt::token::decode_token;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::error::{ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{web, HttpMessage};
use entity::{admins, students};
use futures_util::future::{ready, LocalBoxFuture};
use futures_util::FutureExt;
use log::warn;
use std::rc::Rc;
use std::task::{Context, Poll};

const ADMIN_HEADER_NAME: &str = "X-Admin-Auth";
const STUDENT_HEADER_NAME: &str = "X-Student-Auth";

/// Middleware responsible for handling authentication and user information extraction.
pub(crate) struct AuthMiddleware<const N: usize, S> {
    pub(crate) service: Rc<S>,
    pub(crate) require_admin: bool,
    pub(crate) allowed_roles: Rc<[AdminRole; N]>,
}

impl<const N: usize, S> Service<ServiceRequest> for AuthMiddleware<N, S>
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
        let token = if !self.require_admin {
            req.headers()
                .get(STUDENT_HEADER_NAME)
                .map(|h| h.to_str().unwrap().to_string())
        } else {
            req.headers()
                .get(ADMIN_HEADER_NAME)
                .map(|h| h.to_str().unwrap().to_string())
        };

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
        let require_admin = self.require_admin;
        let allowed_admin_roles = self.allowed_roles.clone();

        // Handle user extraction and request processing
        async move {
            const INVALID_TOKEN: &str = "Invalid token";

            if !require_admin {
                let db_record = cloned_app_state
                    .repositories
                    .students
                    .get_from_id(token.sub)
                    .await;

                let model = match db_record {
                    Ok(m) => m,
                    Err(e) => {
                        return Err(ErrorInternalServerError(e.to_json_error()));
                    }
                };

                let students = match model {
                    None => {
                        warn!("login attempt with non existing student",);
                        return Err(ErrorUnauthorized(INVALID_TOKEN.to_json_error()));
                    }
                    Some(u) => u,
                };

                req.extensions_mut().insert::<students::Model>(students);
            } else {
                if !token.adm {
                    return Err(ErrorUnauthorized(INVALID_TOKEN.to_json_error()));
                }

                let role: AdminRole = match token.rl.try_into() {
                    Ok(role) => role,
                    Err(_) => return Err(ErrorUnauthorized(INVALID_TOKEN.to_json_error())),
                };

                if !allowed_admin_roles.contains(&role) {
                    return Err(ErrorForbidden(
                        "user does not have the necessary permissions".to_json_error(),
                    ));
                };

                let db_record = cloned_app_state
                    .repositories
                    .admins
                    .get_from_id(token.sub)
                    .await;

                let model = match db_record {
                    Ok(m) => m,
                    Err(e) => {
                        return Err(ErrorInternalServerError(e.to_json_error()));
                    }
                };

                let admin = match model {
                    None => {
                        warn!("login attempt with non existing admin",);
                        return Err(ErrorUnauthorized(INVALID_TOKEN.to_json_error()));
                    }
                    Some(u) => u,
                };

                req.extensions_mut().insert::<admins::Model>(admin);
            }

            // Call the wrapped service to handle the request
            let res = srv.call(req).await?;
            Ok(res)
        }
        .boxed_local()
    }
}
