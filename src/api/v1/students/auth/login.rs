use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::jwt::token::create_student_token;
use crate::models::student::Student;
use actix_web::cookie::time::Duration;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::HttpResponse;
use log::error;
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

const WRONG_CREDENTIALS: &str = "Incorrect email or password";

/// Represents data needed for login
#[derive(Deserialize, ToSchema)]
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
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    tag = "Student authentication",
)]
pub(crate) async fn students_login_handler(
    req: Json<LoginStudentsSchema>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // common unauthorized response
    let unauthorized = Err(WRONG_CREDENTIALS.to_json_error(StatusCode::UNAUTHORIZED));

    // look up student by email
    let mut rows = Student::where_col(|s| s.email.equal(&req.email))
        .run(&data.db)
        .await
        .map_err(|e| {
            error!("unable to fetch student from database: {e}");
            database_error()
        })?;

    // 2) not found
    let state = match rows.pop() {
        Some(s) => s, // DbState<Student>
        None => return unauthorized,
    };

    let user: Student = DbState::into_inner(state);

    // 3) wrong password
    if verify_password(&req.password, &user.password_hash).is_err() {
        return unauthorized;
    }

    // create JWT
    let token = create_student_token(
        user.student_id,
        data.config.jwt_secret().as_bytes(),
        Duration::days(data.config.jwt_validity_days()).whole_seconds(),
    )
    .map_err(|e| {
        error!("unable to create student token: {e}");
        database_error()
    })?;

    Ok(HttpResponse::Ok().json(LoginStudentsResponse { token }))
}
