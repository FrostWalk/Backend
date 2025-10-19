use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::student_deliverable_selections_repository;
use crate::jwt::get_user::LoggedUser;
use crate::models::student_deliverable::StudentDeliverable;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentDeliverableSelectionResponse {
    pub student_deliverable_selection_id: i32,
    pub student_id: i32,
    pub student_deliverable_id: i32,
    pub student_deliverable_name: String,
    pub project_id: i32,
}

#[utoipa::path(
    get,
    path = "/v1/students/deliverable-selection/project/{project_id}",
    responses(
        (status = 200, description = "Deliverable selection found", body = StudentDeliverableSelectionResponse),
        (status = 404, description = "No deliverable selected for this project", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Student Deliverable Selections",
)]
/// Get the student's deliverable selection for a project
pub(in crate::api::v1) async fn get_student_deliverable_selection(
    req: HttpRequest, 
    path: Path<i32>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();

    // Get the logged-in user
    let user = req.extensions().get_student().map_err(|_| {
        error_with_log_id(
            "entered a protected route without a user loaded in the request",
            "Authentication error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    // Get the selection
    let selection_state = student_deliverable_selections_repository::get_by_student_and_project(
        &data.db,
        user.student_id,
        project_id,
    )
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
            format!(
                "No deliverable selection found for student {} in project {}",
                user.student_id, project_id
            ),
            "No deliverable selected for this project",
            StatusCode::NOT_FOUND,
            log::Level::Info,
        )
    })?;

    let selection = DbState::into_inner(selection_state);

    // Get the deliverable name
    let mut deliverable_rows = StudentDeliverable::where_col(|sd| {
        sd.student_deliverable_id
            .equal(selection.student_deliverable_id)
    })
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
            format!(
                "Deliverable {} not found for selection",
                selection.student_deliverable_id
            ),
            "Deliverable not found",
            StatusCode::NOT_FOUND,
            log::Level::Error,
        )
    })?;

    let deliverable = DbState::into_inner(deliverable_state);

    Ok(
        HttpResponse::Ok().json(StudentDeliverableSelectionResponse {
            student_deliverable_selection_id: selection.student_deliverable_selection_id,
            student_id: selection.student_id,
            student_deliverable_id: selection.student_deliverable_id,
            student_deliverable_name: deliverable.name,
            project_id,
        }),
    )
}
