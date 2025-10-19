use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, error_with_log_id_and_payload, JsonError};
use crate::database::repositories::{group_deliverable_selections_repository, groups_repository};
use crate::jwt::get_user::LoggedUser;
use crate::models::group_deliverable::GroupDeliverable;
use crate::models::group_deliverable_selection::GroupDeliverableSelection;
use crate::models::project::Project;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateGroupDeliverableSelectionRequest {
    #[schema(example = 5)]
    pub group_deliverable_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateGroupDeliverableSelectionResponse {
    pub group_deliverable_selection_id: i32,
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/v1/students/group-deliverable-selections/{group_id}",
    request_body = CreateGroupDeliverableSelectionRequest,
    responses(
        (status = 201, description = "Deliverable selected successfully", body = CreateGroupDeliverableSelectionResponse),
        (status = 400, description = "Invalid request", body = JsonError),
        (status = 403, description = "Not authorized - must be group leader", body = JsonError),
        (status = 404, description = "Group or deliverable not found", body = JsonError),
        (status = 409, description = "Group already has a selection or link already in use", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Group Deliverable Selections",
)]
/// Create a group deliverable selection (Group Leaders only)
pub(in crate::api::v1) async fn create_group_deliverable_selection(
    req: HttpRequest, 
    path: Path<i32>,
    body: Json<CreateGroupDeliverableSelectionRequest>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let group_id = path.into_inner();

    // Get the logged-in user
    let user = req.extensions().get_student().map_err(|_| {
        error_with_log_id(
            "entered a protected route without a user loaded in the request",
            "Authentication error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    // Validate input - no additional validation needed for simplified request

    // 1. Verify the user is a Group Leader of the group
    let is_leader = groups_repository::is_group_leader(&data.db, user.student_id, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("Database error checking group leader status: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if !is_leader {
        return Err(error_with_log_id(
            format!(
                "Student {} is not a group leader of group {}",
                user.student_id, group_id
            ),
            "Only group leaders can select deliverables",
            StatusCode::FORBIDDEN,
            log::Level::Warn,
        ));
    }

    // 2. Verify the group exists and get its details
    let group_state = groups_repository::get_by_id(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("Database error fetching group: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .ok_or_else(|| {
            error_with_log_id(
                format!("Group {} not found", group_id),
                "Group not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            )
        })?;

    let group = DbState::into_inner(group_state);

    // 3. Verify the group hasn't already selected a deliverable
    let has_selection = group_deliverable_selections_repository::has_selection(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("Database error checking existing selection: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if has_selection {
        return Err(error_with_log_id(
            format!("Group {} already has a deliverable selection", group_id),
            "Group has already selected a deliverable (immutable)",
            StatusCode::CONFLICT,
            log::Level::Warn,
        ));
    }

    // 4. Verify the group_deliverable_id exists and belongs to the same project
    let mut deliverable_rows =
        GroupDeliverable::where_col(|gd| gd.group_deliverable_id.equal(body.group_deliverable_id))
            .run(&data.db)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("Database error fetching deliverable: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    let deliverable_state = deliverable_rows.pop().ok_or_else(|| {
        error_with_log_id(
            format!("Group deliverable {} not found", body.group_deliverable_id),
            "Deliverable not found",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        )
    })?;

    let deliverable = DbState::into_inner(deliverable_state);

    if deliverable.project_id != group.project_id {
        return Err(error_with_log_id(
            format!(
                "Deliverable {} belongs to project {}, but group {} belongs to project {}",
                body.group_deliverable_id, deliverable.project_id, group_id, group.project_id
            ),
            "Deliverable does not belong to the same project as the group",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    // 5. Verify the project's deliverable_selection_deadline has not passed (if set)
    let mut project_rows = Project::where_col(|p| p.project_id.equal(group.project_id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("Database error fetching project: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let project_state = project_rows.pop().ok_or_else(|| {
        error_with_log_id(
            format!("Project {} not found", group.project_id),
            "Project not found",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        )
    })?;

    let project = DbState::into_inner(project_state);

    if let Some(deadline) = project.deliverable_selection_deadline {
        if Utc::now() > deadline {
            return Err(error_with_log_id(
                format!(
                    "Deliverable selection deadline {} has passed for project {}",
                    deadline, group.project_id
                ),
                "Deliverable selection deadline has passed",
                StatusCode::BAD_REQUEST,
                log::Level::Warn,
            ));
        }
    }

    // Create the selection
    let mut selection_state = DbState::new_uncreated(GroupDeliverableSelection {
        group_deliverable_selection_id: 0,
        group_id,
        group_deliverable_id: body.group_deliverable_id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });

    match selection_state.save(&data.db).await {
        Ok(_) => {
            let selection = DbState::into_inner(selection_state);
            Ok(
                HttpResponse::Created().json(CreateGroupDeliverableSelectionResponse {
                    group_deliverable_selection_id: selection.group_deliverable_selection_id,
                    message: "Deliverable selected successfully".to_string(),
                }),
            )
        }
        Err(e) => Err(error_with_log_id_and_payload(
            format!("Failed to create group deliverable selection: {}", e),
            "Failed to create deliverable selection",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )),
    }
}
