use crate::common::json_error::{JsonError, ToJsonError};
use actix_web::error::ErrorNotFound;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use entity::admins;
use serde::Serialize;
use utoipa::ToSchema;


#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetMeAdminResponse {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "Jane")]
    pub first_name: String,
    #[schema(example = "Doe")]
    pub last_name: String,
    #[schema(format = "email", example = "jane.doe@admin.com")]
    pub email: String,
    #[schema(example = 1)]
    pub role_id: i32,
}

#[utoipa::path(
    get,
    path = "/v1/admins/users/me",
    responses(
        (status = 200, description = "Successfully retrieved user profile", body = GetMeAdminResponse),
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

    let response: GetMeAdminResponse = user.into();
    Ok(HttpResponse::Ok().json(response))
}

impl From<admins::Model> for GetMeAdminResponse {
    fn from(value: admins::Model) -> Self {
        Self {
            id: value.admin_id,
            first_name: value.first_name,
            last_name: value.last_name,
            email: value.email,
            role_id: value.admin_role_id,
        }
    }
}
