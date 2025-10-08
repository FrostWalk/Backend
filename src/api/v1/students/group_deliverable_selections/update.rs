use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, error_with_log_id_and_payload, JsonError};
use crate::database::repositories::{group_deliverable_selections_repository, groups_repository};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateGroupDeliverableSelectionRequest {
    #[schema(example = "https://github.com/group1/project-updated")]
    pub link: String,
    #[schema(example = "# Updated project approach\n\n...")]
    pub markdown_text: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct UpdateGroupDeliverableSelectionResponse {
    pub message: String,
}

#[utoipa::path(
    patch,
    path = "/v1/students/groups/{group_id}/deliverable-selection",
    request_body = UpdateGroupDeliverableSelectionRequest,
    responses(
        (status = 200, description = "Selection updated successfully", body = UpdateGroupDeliverableSelectionResponse),
        (status = 400, description = "Invalid request", body = JsonError),
        (status = 403, description = "Not authorized - must be group leader", body = JsonError),
        (status = 404, description = "Selection not found", body = JsonError),
        (status = 409, description = "Link already in use by another group", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Group Deliverable Selections",
)]
/// Update the link and markdown text of a group deliverable selection (Group Leaders only)
/// Note: The deliverable choice itself cannot be changed
pub(in crate::api::v1) async fn update_group_deliverable_selection(
    req: HttpRequest, body: Json<UpdateGroupDeliverableSelectionRequest>, group_id: Path<i32>,
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let group_id = group_id.into_inner();

    // Get the logged-in user
    let user = req.extensions().get_student().map_err(|_| {
        error_with_log_id(
            "entered a protected route without a user loaded in the request",
            "Authentication error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    // Validate input
    if body.link.trim().is_empty() {
        return Err(JsonError::new(
            "Link field is mandatory",
            StatusCode::BAD_REQUEST,
        ));
    }

    if body.markdown_text.trim().is_empty() {
        return Err(JsonError::new(
            "Markdown text field is mandatory",
            StatusCode::BAD_REQUEST,
        ));
    }

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
            "Only group leaders can update deliverable selections",
            StatusCode::FORBIDDEN,
            log::Level::Warn,
        ));
    }

    // 2. Get the existing selection
    let mut selection_state =
        group_deliverable_selections_repository::get_by_group_id(&data.db, group_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("Database error fetching selection: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?
            .ok_or_else(|| {
                error_with_log_id(
                    format!("No deliverable selection found for group {}", group_id),
                    "No deliverable selection found to update",
                    StatusCode::NOT_FOUND,
                    log::Level::Warn,
                )
            })?;

    // 3. Check if the new link is already in use by another group
    let link_exists_for_other =
        group_deliverable_selections_repository::link_exists_for_other_group(
            &data.db, &body.link, group_id,
        )
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("Database error checking link uniqueness: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if link_exists_for_other {
        return Err(error_with_log_id_and_payload(
            format!("Link '{}' is already in use by another group", body.link),
            "This link is already in use by another group",
            StatusCode::CONFLICT,
            log::Level::Warn,
            &body,
        ));
    }

    // Update the selection (only link and markdown_text, NOT group_deliverable_id)
    {
        let selection = selection_state.as_mut();
        selection.link = body.link.clone();
        selection.markdown_text = body.markdown_text.clone();
        selection.updated_at = Utc::now();
    }

    match selection_state.save(&data.db).await {
        Ok(_) => Ok(
            HttpResponse::Ok().json(UpdateGroupDeliverableSelectionResponse {
                message: "Selection details updated successfully".to_string(),
            }),
        ),
        Err(e) => Err(error_with_log_id_and_payload(
            format!("Failed to update group deliverable selection: {}", e),
            "Failed to update deliverable selection",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )),
    }
}
