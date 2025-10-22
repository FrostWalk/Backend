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
pub(crate) struct DeleteComponentImplementationDetailRequest {
    #[schema(example = 5)]
    pub group_deliverable_component_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct DeleteComponentImplementationDetailResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/v1/students/group-component-implementation-details/{group_id}",
    request_body = DeleteComponentImplementationDetailRequest,
    responses(
        (status = 200, description = "Component implementation detail deleted successfully", body = DeleteComponentImplementationDetailResponse),
        (status = 400, description = "Invalid request", body = JsonError),
        (status = 403, description = "Not authorized - must be group leader", body = JsonError),
        (status = 404, description = "Group, selection, or implementation detail not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Group Component Implementation Details",
)]
/// Delete implementation details for a component (Group Leaders only)
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(in crate::api::v1) async fn delete_component_implementation_detail(
    req: HttpRequest, path: Path<i32>, body: Json<DeleteComponentImplementationDetailRequest>,
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
            "Only group leaders can delete component implementation details",
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

    // 3. Delete the implementation detail
    let deleted = group_component_implementation_details_repository::delete(
        &data.db,
        selection.group_deliverable_selection_id,
        body.group_deliverable_component_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("Failed to delete component implementation detail: {}", e),
            "Failed to delete implementation detail",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    if !deleted {
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
        HttpResponse::Ok().json(DeleteComponentImplementationDetailResponse {
            message: "Component implementation detail deleted successfully".to_string(),
        }),
    )
}
