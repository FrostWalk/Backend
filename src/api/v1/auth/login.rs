use crate::app_state::AppState;
use crate::common::json_error::JsonError;
use crate::jwt::token::create_token;
use crate::jwt::COOKIE_NAME;
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::{Error, HttpResponse};
use password_auth::verify_password;
use serde::Deserialize;

const WRONG_CREDENTIALS: &str = "Incorrect email or password";

/// Represents data needed for login
#[derive(Deserialize)]
pub(super) struct LoginUserSchema {
    email: String,
    password: String,
}
/// Authenticates a user and returns a JWT token cookie.
///
/// This endpoint validates user credentials and issues a JWT token upon successful authentication.
/// The token is returned as an HTTP-only cookie for enhanced security.
///
/// # Flow
/// 1. Validate email/password credentials
/// 2. Verify user exists in database
/// 3. Check password hash matches
/// 4. Verify user has an active role in a project
/// 5. Generate JWT token with user claims
/// 6. Set token in secure HTTP-only cookie
///
/// # Arguments
/// * `req` - JSON payload containing user credentials (`LoginUserSchema`)
/// * `app_state` - Shared application state containing database connections and configuration
///
/// # Returns
/// * `200 OK` - With JWT token in `COOKIE_NAME` cookie
/// * Error responses with appropriate status codes
///
/// # Errors
/// * `400 Bad Request` - Invalid request format
/// * `401 Unauthorized` - Multiple scenarios:
///   - Invalid email/password combination
///   - User not found in database
///   - User has no assigned project role
/// * `500 Internal Server Error` - Database errors or token generation failures
///
/// # Example Request
/// ```json
/// POST /api/auth/login
/// {
///     "email": "user@example.com",
///     "password": "securepassword123"
/// }
/// ```
///
/// # Example Response
/// ```http
/// HTTP/1.1 200 OK
/// Set-Cookie: token=eyJhbGci...; HttpOnly; Secure; Path=/; Max-Age=3600
/// ```
///
/// # Security
/// - Uses HTTP-only cookies to prevent XSS attacks
/// - Requires secure flag when running in production
/// - Password verification uses constant-time comparison
/// - Sensitive credentials are never stored in plain text
///
/// # Notes
/// - JWT configuration (secret, expiration) comes from application config
/// - Cookie security settings depend on environment configuration
/// - User roles are determined by their latest project participation
pub(super) async fn login_handler(
    req: Json<LoginUserSchema>, app_state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    // convenience variable storing error in case of wrong credentials or user not found
    let unauthorized = Err(ErrorUnauthorized(WRONG_CREDENTIALS.to_json_error()));

    // find the user in the db
    let opt = app_state
        .repositories
        .users_repository
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

    // get the user's role in the last project he/she participated in
    let role_opt = app_state
        .repositories
        .users_repository
        .get_current_role(user.id)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_json_error()))?;

    // if the user don't have a role in any project return error
    let role = role_opt.ok_or(ErrorUnauthorized(
        "User has no role in any project".to_json_error(),
    ))?;

    // create jwt from user data, if the creation fails return error 500
    let token = create_token(
        user.id,
        role,
        app_state.config.jwt_secret().as_bytes(),
        app_state.config.jwt_expires_in(),
    )
    .map_err(|e| ErrorInternalServerError(e.to_json_error()))?;

    // create cookie with token, in production change `secure_cookie` to true in `config.toml`
    let jwt_cookie = Cookie::build(COOKIE_NAME, token)
        .path("/")
        .secure(app_state.config.secure_cookie())
        .http_only(true)
        .max_age(Duration::new(app_state.config.jwt_expires_in(), 0))
        .finish();

    // return status code 200 with cookie
    Ok(HttpResponse::Ok().cookie(jwt_cookie).finish())
}
