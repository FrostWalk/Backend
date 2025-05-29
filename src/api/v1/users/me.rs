use crate::app_state::AppState;
use crate::common::json_error::{JsonError, ToJsonError};
use actix_web::error::ErrorNotFound;
use actix_web::web::Data;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use entity::students;
use serde::Serialize;
use utoipa::ToSchema;

/// Schema for user profile with associated projects and roles
#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetMeResponse {}

#[utoipa::path(
    get,
    path = "/v1/users/me",
    responses(
        (status = 200, description = "Successfully retrieved user profile", body = GetMeResponse),
        (status = 404, description = "User not found in request context", body = JsonError),
        (status = 500, description = "Internal server error during serialization or database query", body = JsonError)
    ),
    tag = "Users",
)]
/// Returns authenticated user's profile information
///
/// Extracts user data from request extensions (set by auth middleware),
/// filters sensitive fields, and returns user data with associated projects and roles.
pub(super) async fn me_handler(
    req: HttpRequest, app_state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    let user = match req.extensions().get::<students::Model>() {
        None => return Err(ErrorNotFound("user does not exists".to_json_error())),
        Some(u) => u.clone(),
    };

    Ok(HttpResponse::Ok().json(GetMeResponse {}))
}
