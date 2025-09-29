use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::group::Group;
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
    security(("UserAuth" = [])),
    tag = "Groups management",
)]
/// Check if a group name already exists in a project
///
/// This endpoint allows students to check if a group name is already taken
/// within a specific project before creating a group.
pub(super) async fn check_name(
    req: HttpRequest, data: Data<AppData>, body: Json<CheckNameRequest>,
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

    // Query directly for groups with matching project_id AND name
    let existing_groups = match Group::where_col(|g| g.project_id.equal(body.project_id))
        .where_col(|g| g.name.equal(&body.name))
        .run(&data.db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to check group name availability: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let exists = !existing_groups.is_empty();

    Ok(HttpResponse::Ok().json(CheckNameResponse { exists }))
}
