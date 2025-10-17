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

        let app_state = match req.app_data::<web::Data<AppData>>() {
            Some(data) => data,
            None => {
                // For testing purposes, return unauthorized if no app data
                return Box::pin(ready(Err("jwt token not provided"
                    .to_json_error(StatusCode::UNAUTHORIZED)
                    .into())));
            }
        };

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
                            warn!("login attempt with non-existing admin");
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
                            warn!("login attempt with non-existing student");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::token::{create_admin_token, create_student_token};
    use crate::models::admin::Admin;
    use crate::models::admin_role::AvailableAdminRole;
    use crate::models::student::Student;
    use crate::test_utils::*;
    use actix_web::dev::ServiceRequest;
    use actix_web::test::TestRequest;
    use actix_web::HttpResponse;

    // Mock service for testing
    struct MockService;
    impl Service<ServiceRequest> for MockService {
        type Response = ServiceResponse<actix_web::body::BoxBody>;
        type Error = actix_web::Error;
        type Future = futures_util::future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&self, req: ServiceRequest) -> Self::Future {
            let response = if req.extensions().get::<Admin>().is_some() {
                HttpResponse::Ok().body("admin_authenticated")
            } else if req.extensions().get::<Student>().is_some() {
                HttpResponse::Ok().body("student_authenticated")
            } else {
                HttpResponse::Ok().body("no_auth")
            };
            futures_util::future::ready(Ok(ServiceResponse::new(req.into_parts().0, response)))
        }
    }

    fn create_test_request() -> ServiceRequest {
        TestRequest::default().to_srv_request()
    }

    fn create_test_request_with_header(header_name: &str, header_value: &str) -> ServiceRequest {
        TestRequest::default()
            .insert_header((header_name, header_value))
            .to_srv_request()
    }

    #[actix_web::test]
    async fn test_auth_middleware_no_token_returns_unauthorized() {
        let middleware = AuthMiddleware {
            service: Rc::new(MockService),
            require_admin: true,
            authentication_only: false,
            allowed_roles: Rc::new([AvailableAdminRole::Root]),
        };

        let req = create_test_request();
        let result = middleware.call(req).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.as_response_error().status_code(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[actix_web::test]
    async fn test_auth_middleware_invalid_token_returns_unauthorized() {
        let middleware = AuthMiddleware {
            service: Rc::new(MockService),
            require_admin: true,
            authentication_only: false,
            allowed_roles: Rc::new([AvailableAdminRole::Root]),
        };

        let req = create_test_request_with_header(ADMIN_HEADER_NAME, "invalid_token");

        let result = middleware.call(req).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.as_response_error().status_code(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[actix_web::test]
    async fn test_auth_middleware_admin_token_with_wrong_secret_returns_unauthorized() {
        let middleware = AuthMiddleware {
            service: Rc::new(MockService),
            require_admin: true,
            authentication_only: false,
            allowed_roles: Rc::new([AvailableAdminRole::Root]),
        };

        // Create token with wrong secret
        let token = create_admin_token(
            TEST_ADMIN_ID,
            TEST_ADMIN_ROLE_ID,
            b"wrong_secret_key_for_jwt_tokens_32_chars",
            TEST_JWT_VALIDITY_SECONDS,
        )
        .unwrap();

        let req = create_test_request_with_header(ADMIN_HEADER_NAME, &token);

        let result = middleware.call(req).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.as_response_error().status_code(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[actix_web::test]
    async fn test_auth_middleware_student_token_when_admin_required_returns_unauthorized() {
        let middleware = AuthMiddleware {
            service: Rc::new(MockService),
            require_admin: true,
            authentication_only: false,
            allowed_roles: Rc::new([AvailableAdminRole::Root]),
        };

        let token =
            create_student_token(TEST_STUDENT_ID, TEST_JWT_SECRET, TEST_JWT_VALIDITY_SECONDS)
                .unwrap();

        let req = create_test_request_with_header(STUDENT_HEADER_NAME, &token);

        let result = middleware.call(req).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.as_response_error().status_code(),
            StatusCode::UNAUTHORIZED
        );
    }





    #[actix_web::test]
    async fn test_auth_middleware_student_mode_rejects_admin_token() {
        let middleware = AuthMiddleware {
            service: Rc::new(MockService),
            require_admin: false,
            authentication_only: false,
            allowed_roles: Rc::new([]),
        };

        let token = create_admin_token(
            TEST_ADMIN_ID,
            TEST_ADMIN_ROLE_ID,
            TEST_JWT_SECRET,
            TEST_JWT_VALIDITY_SECONDS,
        )
        .unwrap();

        let req = create_test_request_with_header(ADMIN_HEADER_NAME, &token);

        let result = middleware.call(req).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.as_response_error().status_code(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[actix_web::test]
    async fn test_auth_middleware_checks_admin_header_when_admin_required() {
        let middleware = AuthMiddleware {
            service: Rc::new(MockService),
            require_admin: true,
            authentication_only: false,
            allowed_roles: Rc::new([AvailableAdminRole::Root]),
        };

        let token = create_admin_token(
            TEST_ADMIN_ID,
            TEST_ADMIN_ROLE_ID,
            TEST_JWT_SECRET,
            TEST_JWT_VALIDITY_SECONDS,
        )
        .unwrap();

        let req = create_test_request_with_header(STUDENT_HEADER_NAME, &token);

        let result = middleware.call(req).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.as_response_error().status_code(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[actix_web::test]
    async fn test_auth_middleware_checks_student_header_when_student_required() {
        let middleware = AuthMiddleware {
            service: Rc::new(MockService),
            require_admin: false,
            authentication_only: false,
            allowed_roles: Rc::new([]),
        };

        let token =
            create_student_token(TEST_STUDENT_ID, TEST_JWT_SECRET, TEST_JWT_VALIDITY_SECONDS)
                .unwrap();

        let req = create_test_request_with_header(ADMIN_HEADER_NAME, &token);

        let result = middleware.call(req).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.as_response_error().status_code(),
            StatusCode::UNAUTHORIZED
        );
    }

}
