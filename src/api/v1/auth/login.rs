use crate::app_state::AppState;
use crate::common::json_error::{JsonError, ToJsonError};
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
use utoipa::ToSchema;

const WRONG_CREDENTIALS: &str = "Incorrect email or password";

/// Represents data needed for login
#[derive(Deserialize, ToSchema)]
pub(crate) struct LoginUserSchema {
    #[schema(example = "user@example.com")]
    email: String,
    #[schema(example = "password123")]
    password: String,
}
/// Authenticates a user and returns a JWT token cookie.
///
/// This endpoint validates user credentials and issues a JWT token upon successful authentication.
#[utoipa::path(
    post,
    path = "/v1/auth/login",
    request_body = LoginUserSchema,
    responses(
        (status = 200, description = "Login successful",
            headers(
                ("Set-Cookie" = String, description = "JWT token in a cookie")
            )
        ),
        (status = 401, description = "Wrong credentials or no role in any project", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    tag = "Auth"
)]
pub(crate) async fn login_handler(
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
        Duration::days(app_state.config.jwt_validity_days()).whole_seconds(),
    )
    .map_err(|e| ErrorInternalServerError(e.to_json_error()))?;

    // create cookie with token, in production change `secure_cookie` to true in `config.toml`
    let jwt_cookie = Cookie::build(COOKIE_NAME, token)
        .path("/")
        .secure(app_state.config.secure_cookie())
        .http_only(true)
        .max_age(Duration::days(app_state.config.jwt_validity_days()))
        .finish();

    // return status code 200 with cookie
    Ok(HttpResponse::Ok().cookie(jwt_cookie).finish())
}
