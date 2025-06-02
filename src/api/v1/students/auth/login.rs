use crate::app_data::AppData;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::jwt::token::create_student_token;
use actix_web::cookie::time::Duration;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::{Error, HttpResponse};
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

/// Authenticates a user and returns a JWT.
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
    tag = "Auth"
)]
pub(crate) async fn students_login_handler(
    req: Json<LoginStudentsSchema>, data: Data<AppData>,
) -> Result<HttpResponse, Error> {
    // convenience variable storing error in case of wrong credentials or user not found
    let unauthorized = Err(ErrorUnauthorized(WRONG_CREDENTIALS.to_json_error()));

    // find the user in the db
    let opt = data
        .repositories
        .students
        .get_from_mail(&req.email)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_json_error()))?;

    // user is not found
    if opt.is_none() {
        return unauthorized;
    }

    let user = opt.unwrap();

    // password is incorrect
    if verify_password(&user.password_hash, &req.password).is_err() {
        return unauthorized;
    }

    // create jwt from user data if the creation fails return error 500
    let token = create_student_token(
        user.student_id,
        data.config.jwt_secret().as_bytes(),
        Duration::days(data.config.jwt_validity_days()).whole_seconds(),
    )
    .map_err(|e| ErrorInternalServerError(e.to_json_error()))?;

    // return status code 200 with cookie
    Ok(HttpResponse::Ok().json(LoginStudentsResponse { token }))
}
