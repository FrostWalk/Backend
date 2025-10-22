use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::students_repository;
use crate::jwt::token::create_student_token;
use crate::logging::payload_capture::capture_response_status;
use actix_web::cookie::time::Duration;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::HttpResponse;
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

const WRONG_CREDENTIALS: &str = "Incorrect email or password";

/// Represents data needed for login
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct LoginStudentsSchema {
    #[schema(example = "user@example.com")]
    email: String,
    #[schema(example = "password123")]
    password: String,
}
/// Represents the response structure for a successful login.
///
/// This struct includes a JWT token that can be used for later authenticated requests.
#[derive(Serialize, ToSchema)]
pub(crate) struct LoginStudentsResponse {
    /// JSON Web Token (JWT) to be used for authentication in later requests.
    #[schema(example = "eyJhbGc9...")]
    token: String,
}

/// Authenticates a student and returns a JWT.
///
/// This endpoint validates user credentials and issues a JWT upon successful authentication.
#[utoipa::path(
    post,
    path = "/v1/students/auth/login",
    request_body = LoginStudentsSchema,
    responses(
        (status = 200, description = "Login successful", body = LoginStudentsResponse),
        (status = 401, description = "Wrong credentials", body = JsonError),
        (status = 403, description = "Account pending email confirmation", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    tag = "Student authentication",
)]
pub(crate) async fn students_login_handler(
    body: Json<LoginStudentsSchema>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // common unauthorized response
    let unauthorized = Err(WRONG_CREDENTIALS.to_json_error(StatusCode::UNAUTHORIZED));

    // look up student by email
    let student_state = students_repository::get_by_email(&data.db, &body.email)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to fetch student from database: {}", e),
                "Authentication failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    // 2) not found
    let user = match student_state {
        Some(state) => DbState::into_inner(state),
        None => return unauthorized,
    };

    // 3) wrong password
    if verify_password(&body.password, &user.password_hash).is_err() {
        return unauthorized;
    }

    // 4) check if account is pending email confirmation
    if user.is_pending {
        return Err(
            "Account pending email confirmation. Please check your email to confirm your account."
                .to_json_error(StatusCode::FORBIDDEN),
        );
    }

    // create JWT
    let token = create_student_token(
        user.student_id,
        data.config.jwt_secret().as_bytes(),
        Duration::days(data.config.jwt_validity_days()).whole_seconds(),
    )
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to create student token: {}", e),
            "Authentication failed",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    // Capture successful response status
    capture_response_status(200);

    Ok(HttpResponse::Ok().json(LoginStudentsResponse { token }))
}
