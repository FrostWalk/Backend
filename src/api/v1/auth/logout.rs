use crate::app_state::AppState;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::jwt::COOKIE_NAME;
use actix_web::cookie::Cookie;
use actix_web::error::ErrorBadRequest;
use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::{Error, HttpRequest};

#[utoipa::path(
    post,
    path = "/v1/auth/logout",
    responses(
        (status = 200, description = "Logout successful", 
            headers(("Set-Cookie" = String, description = "Expired authentication cookie"))),
        (status = 400, description = "Missing authentication cookie", body = JsonError)
    ),
    tag = "Authentication",
    security(
        ("cookieAuth" = [])
    )
)]
/// Handles user logout by invalidating the authentication cookie.
///
/// # Behavior
/// - Checks for existing authentication cookie
/// - Returns 400 Bad Request if cookie is missing
/// - Returns 200 OK with an expired cookie to clear client-side authentication
pub(super) async fn logout_handler(
    req: HttpRequest, app_state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    if req.cookie(COOKIE_NAME).is_none() {
        return Err(ErrorBadRequest("cookie not found".to_json_error()));
    }

    let mut remove_cookie = Cookie::build(COOKIE_NAME, "")
        .path("/")
        .secure(app_state.config.secure_cookie())
        .http_only(true)
        .finish();

    remove_cookie.make_removal();

    Ok(HttpResponse::Ok().cookie(remove_cookie).finish())
}
