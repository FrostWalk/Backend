use crate::api::v1::admins::users::AdminResponseScheme;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use log::error;

#[utoipa::path(
    get,
    path = "/v1/admins/users/me",
    responses(
        (status = 200, description = "Successfully retrieved user profile", body = AdminResponseScheme),
        (status = 404, description = "User not found in request context", body = JsonError),
        (status = 500, description = "Internal server error during serialization or database query", body = JsonError)
    ),
    tag = "Admin users management",
)]
/// Retrieves the profile information of the currently authenticated admin.
///
/// This endpoint is designed to return detailed information about the admin making the request.
/// It extracts the admin's data from the request context, which should be populated by middleware
/// responsible for authentication and authorization.
pub(super) async fn admins_me_handler(req: HttpRequest) -> Result<HttpResponse, JsonError> {
    let user = match req.extensions().get_admin() {
        Ok(user) => user,
        Err(e) => {
            error!("entered a protected route without a user loaded in the request");
            return Err(e.to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    let response: AdminResponseScheme = user.into();
    Ok(HttpResponse::Ok().json(response))
}
