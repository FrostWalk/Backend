use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, error_with_log_id_and_payload, JsonError};
use crate::database::repositories::{
    group_component_implementation_details_repository, group_deliverable_selections_repository,
    groups_repository,
};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateComponentImplementationDetailRequest {
    #[schema(example = 5)]
    pub group_deliverable_component_id: i32,
    #[schema(example = "# Updated Component Description\n\nThis component now handles...")]
    pub markdown_description: String,
    #[schema(example = "https://github.com/group1/component-updated")]
    pub repository_link: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct UpdateComponentImplementationDetailResponse {
    pub message: String,
}

#[utoipa::path(
    patch,
    path = "/v1/students/group-component-implementation-details/{group_id}",
    request_body = UpdateComponentImplementationDetailRequest,
    responses(
        (status = 200, description = "Component implementation detail updated successfully", body = UpdateComponentImplementationDetailResponse),
        (status = 400, description = "Invalid request", body = JsonError),
        (status = 403, description = "Not authorized - must be group leader", body = JsonError),
        (status = 404, description = "Group, selection, or implementation detail not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Group Component Implementation Details",
)]
/// Update implementation details for a single component (Group Leaders only)
pub(in crate::api::v1) async fn update_component_implementation_detail(
    req: HttpRequest, body: Json<UpdateComponentImplementationDetailRequest>, group_id: Path<i32>,
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
    if body.markdown_description.trim().is_empty() {
        return Err(JsonError::new(
            "Markdown description field is mandatory",
            StatusCode::BAD_REQUEST,
        ));
    }

    if body.repository_link.trim().is_empty() {
        return Err(JsonError::new(
            "Repository link field is mandatory",
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
            "Only group leaders can update component implementation details",
            StatusCode::FORBIDDEN,
            log::Level::Warn,
        ));
    }

    // 2. Verify the group has selected a deliverable
    let selection_state =
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
                    "Group must select a deliverable first",
                    StatusCode::NOT_FOUND,
                    log::Level::Warn,
                )
            })?;

    let selection = welds::state::DbState::into_inner(selection_state);

    // 3. Update the implementation detail
    let updated_detail = group_component_implementation_details_repository::update(
        &data.db,
        selection.group_deliverable_selection_id,
        body.group_deliverable_component_id,
        body.markdown_description.clone(),
        body.repository_link.clone(),
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("Failed to update component implementation detail: {}", e),
            "Failed to update implementation detail",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    if updated_detail.is_none() {
        return Err(error_with_log_id(
            format!(
                "Implementation details not found for component {}",
                body.group_deliverable_component_id
            ),
            "Implementation details not found for this component",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        ));
    }

    Ok(
        HttpResponse::Ok().json(UpdateComponentImplementationDetailResponse {
            message: "Component implementation detail updated successfully".to_string(),
        }),
    )
}
