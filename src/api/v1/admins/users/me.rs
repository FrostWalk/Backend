use crate::api::v1::admins::users::AdminResponseScheme;
use crate::common::json_error::{JsonError, ToJsonError};
use actix_web::error::ErrorNotFound;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use entity::admins;

#[utoipa::path(
    get,
    path = "/v1/admins/users/me",
    responses(
        (status = 200, description = "Successfully retrieved user profile", body = AdminResponseScheme),
        (status = 404, description = "User not found in request context", body = JsonError),
        (status = 500, description = "Internal server error during serialization or database query", body = JsonError)
    ),
    tag = "Users",
)]
/// Retrieves the profile information of the currently authenticated admin.
///
/// This endpoint is designed to return detailed information about the admin making the request.
/// It extracts the admin's data from the request context, which should be populated by middleware
/// responsible for authentication and authorization.
pub(super) async fn admins_me_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user = match req.extensions().get::<admins::Model>() {
        None => return Err(ErrorNotFound("user does not exists".to_json_error())),
        Some(u) => u.clone(),
    };

    let response: AdminResponseScheme = user.into();
    Ok(HttpResponse::Ok().json(response))
}
