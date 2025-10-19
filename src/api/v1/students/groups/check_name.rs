use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::groups_repository;
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CheckNameRequest {
    pub project_id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CheckNameResponse {
    pub exists: bool,
}

#[utoipa::path(
    post,
    path = "/v1/students/groups/check-name",
    request_body = CheckNameRequest,
    responses(
        (status = 200, description = "A boolean indicating if name exists already", body = CheckNameResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Groups management",
)]
/// Check if a group name already exists in a project
///
/// This endpoint allows students to check if a group name is already taken
/// within a specific project before creating a group.
pub(super) async fn check_name(
    req: HttpRequest, 
    body: Json<CheckNameRequest>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _user = match req.extensions().get_student() {
        Ok(user) => user,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without a user loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    // Check if the group name already exists for this project
    let exists = groups_repository::name_exists_for_project(&data.db, body.project_id, &body.name)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to check group name availability: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    Ok(HttpResponse::Ok().json(CheckNameResponse { exists }))
}
