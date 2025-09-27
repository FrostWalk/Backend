use crate::app_data::AppData;
use crate::common::json_error::{database_error, ToJsonError};
use crate::jwt::token::decode_token;
use crate::models::admin::Admin;
use crate::models::admin_role::AvailableAdminRole;
use crate::models::student::Student;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::http::StatusCode;
use actix_web::{web, HttpMessage};
use futures_util::future::{ready, LocalBoxFuture};
use futures_util::FutureExt;
use log::{error, warn};
use std::rc::Rc;
use std::task::{Context, Poll};
use welds::state::DbState;

pub(crate) const ADMIN_HEADER_NAME: &str = "X-Admin-Token";
pub(crate) const STUDENT_HEADER_NAME: &str = "X-Student-Token";

/// Middleware responsible for handling authentication and user information extraction.
pub(crate) struct AuthMiddleware<const N: usize, S> {
    pub(super) service: Rc<S>,
    pub(super) require_admin: bool,
    pub(super) authentication_only: bool,
    pub(super) allowed_roles: Rc<[AvailableAdminRole; N]>,
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
        const INVALID_TOKEN: &str = "Invalid token";

        // Extract token based on authentication mode
        let token = if self.authentication_only {
            // Try both headers when authentication_only is true
            req.headers()
                .get(STUDENT_HEADER_NAME)
                .or_else(|| req.headers().get(ADMIN_HEADER_NAME))
                .map(|h| h.to_str().unwrap().to_string())
        } else if self.require_admin {
            req.headers()
                .get(ADMIN_HEADER_NAME)
                .map(|h| h.to_str().unwrap().to_string())
        } else {
            req.headers()
                .get(STUDENT_HEADER_NAME)
                .map(|h| h.to_str().unwrap().to_string())
        };

        // Return early if no token found
        if token.is_none() {
            return Box::pin(ready(Err("jwt token not provided"
                .to_json_error(StatusCode::UNAUTHORIZED)
                .into())));
        }

        let app_state = req.app_data::<web::Data<AppData>>().unwrap();

        // Decode token
        let token = match decode_token(token.unwrap(), app_state.config.jwt_secret().as_bytes()) {
            Ok(t) => t,
            Err(e) => {
                warn!("unable to decode jwt token: {}", e);
                return Box::pin(ready(Err(INVALID_TOKEN
                    .to_json_error(StatusCode::UNAUTHORIZED)
                    .into())));
            }
        };

        let cloned_app_state = app_state.clone();
        let srv = Rc::clone(&self.service);
        let require_admin = self.require_admin;
        let authentication_only = self.authentication_only;
        let allowed_admin_roles = self.allowed_roles.clone();

        async move {
            // Determine if we should process as admin based on token and requirements
            let process_as_admin = if authentication_only {
                token.adm // Use whatever the token indicates
            } else {
                require_admin // Use the middleware configuration
            };

            if process_as_admin {
                // Admin processing
                if !token.adm {
                    return Err(INVALID_TOKEN.to_json_error(StatusCode::UNAUTHORIZED).into());
                }

                let role: AvailableAdminRole = match token.rl.try_into() {
                    Ok(role) => role,
                    Err(_) => {
                        return Err(INVALID_TOKEN.to_json_error(StatusCode::UNAUTHORIZED).into())
                    }
                };

                // Only check roles if not in authentication_only mode
                if !authentication_only && !allowed_admin_roles.contains(&role) {
                    return Err("user does not have the necessary permissions"
                        .to_json_error(StatusCode::FORBIDDEN)
                        .into());
                }

                let admin = match Admin::where_col(|a| a.admin_id.equal(token.sub))
                    .run(&cloned_app_state.db)
                    .await
                {
                    Ok(mut rows) => match rows.pop() {
                        Some(state) => DbState::into_inner(state),
                        None => {
                            warn!("login attempt with non existing admin");
                            return Err(INVALID_TOKEN
                                .to_json_error(StatusCode::UNAUTHORIZED)
                                .into());
                        }
                    },
                    Err(e) => {
                        error!("unable to fetch admin from database: {}", e);
                        return Err("unable to fetch admin from database"
                            .to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
                            .into());
                    }
                };

                req.extensions_mut().insert::<Admin>(admin);
            } else {
                // Student processing
                let student = match Student::where_col(|s| s.student_id.equal(token.sub))
                    .run(&cloned_app_state.db)
                    .await
                {
                    Ok(mut rows) => match rows.pop() {
                        Some(state) => DbState::into_inner(state),
                        None => {
                            warn!("login attempt with non existing student");
                            return Err(INVALID_TOKEN
                                .to_json_error(StatusCode::UNAUTHORIZED)
                                .into());
                        }
                    },
                    Err(e) => {
                        error!("unable to fetch student from database: {}", e);
                        return Err(database_error().into());
                    }
                };

                req.extensions_mut().insert::<Student>(student);
            }

            // Call the wrapped service
            srv.call(req).await
        }
        .boxed_local()
    }
}
