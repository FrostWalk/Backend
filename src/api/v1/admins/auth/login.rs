use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::admins_repository;
use crate::jwt::token::create_admin_token;
use crate::logging::payload_capture::capture_response_status;
use actix_web::cookie::time::Duration;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

const WRONG_CREDENTIALS: &str = "Incorrect email or password";

/// Represents data needed for login
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct LoginAdminsSchema {
    #[schema(example = "user@example.com")]
    email: String,
    #[schema(example = "password123")]
    password: String,
}
/// Represents the response structure for a successful login.
///
/// This struct includes a JWT token that can be used for later authenticated requests.
#[derive(Serialize, ToSchema)]
pub(crate) struct LoginAdminsResponse {
    /// JSON Web Token (JWT) to be used for authentication in later requests.
    #[schema(example = "eyJhbGc9...")]
    token: String,
}

/// Authenticates an admin and returns a JWT.
///
/// This endpoint validates user credentials and issues a JWT upon successful authentication.
#[utoipa::path(
    post,
    path = "/v1/admins/auth/login",
    request_body = LoginAdminsSchema,
    responses(
        (status = 200, description = "Login successful", body = LoginAdminsResponse),
        (status = 401, description = "Wrong credentials", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    tag = "Admin authentication"
)]
pub(crate) async fn admins_login_handler(
    req: web::Json<LoginAdminsSchema>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // common unauthorized response
    let unauthorized = Err(WRONG_CREDENTIALS.to_json_error(StatusCode::UNAUTHORIZED));

    // find the user by email
    let admin_state = admins_repository::get_by_email(&data.db, &req.email)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to fetch admin from database: {}", e),
                "Authentication failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &req,
            )
        })?;

    // 2) not found -> unauthorized
    let user = match admin_state {
        Some(state) => DbState::into_inner(state),
        None => return unauthorized,
    };

    // 3) wrong password
    if verify_password(&req.password, &user.password_hash).is_err() {
        return unauthorized;
    }

    // create JWT
    let token = create_admin_token(
        user.admin_id,
        user.admin_role_id,
        data.config.jwt_secret().as_bytes(),
        Duration::days(data.config.jwt_validity_days()).whole_seconds(),
    )
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to create admin jwt token: {}", e),
            "Authentication failed",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &req,
        )
    })?;

    // Capture successful response status
    capture_response_status(200);

    Ok(HttpResponse::Ok().json(LoginAdminsResponse { token }))
}
