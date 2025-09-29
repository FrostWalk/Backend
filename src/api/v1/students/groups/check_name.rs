use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::group::Group;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Query};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CheckNameQuery {
    pub project_id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CheckNameResponse {
    pub exists: bool,
    pub message: String,
}

#[utoipa::path(
    get,
    path = "/v1/students/groups/check-name",
    params(
        ("project_id" = i32, Query, description = "Project ID"),
        ("name" = String, Query, description = "Group name to check")
    ),
    responses(
        (status = 200, description = "Group name availability check result", body = CheckNameResponse),
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
    req: HttpRequest, data: Data<AppData>, query: Query<CheckNameQuery>,
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

    // Check if a group with this name exists in the project
    let existing_groups = match Group::where_col(|g| g.project_id.equal(query.project_id))
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

    // Filter by name
    let mut exists = false;
    for group_state in existing_groups {
        let group = DbState::into_inner(group_state);
        if group.name == query.name {
            exists = true;
            break;
        }
    }

    Ok(HttpResponse::Ok().json(CheckNameResponse {
        exists,
        message: if exists {
            format!("Group name '{}' already exists in this project", query.name)
        } else {
            format!("Group name '{}' is available", query.name)
        },
    }))
}
