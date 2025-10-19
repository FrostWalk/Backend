use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::student_deliverable_selections_repository;
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct DeleteStudentDeliverableSelectionResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/v1/students/deliverable-selection/project/{project_id}",
    responses(
        (status = 200, description = "Selection deleted successfully", body = DeleteStudentDeliverableSelectionResponse),
        (status = 404, description = "No selection found to delete", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Student Deliverable Selections",
)]
/// Delete a student deliverable selection
pub(in crate::api::v1) async fn delete_student_deliverable_selection(
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

    // Check if selection exists before attempting to delete
    let has_selection = student_deliverable_selections_repository::has_selection_for_project(
        &data.db,
        user.student_id,
        project_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("Database error checking selection: {}", e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    if !has_selection {
        return Err(error_with_log_id(
            format!(
                "No deliverable selection found for student {} in project {}",
                user.student_id, project_id
            ),
            "No deliverable selection found to delete",
            StatusCode::NOT_FOUND,
            log::Level::Info,
        ));
    }

    // Delete the selection
    student_deliverable_selections_repository::delete_by_student_and_project(
        &data.db,
        user.student_id,
        project_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("Database error deleting selection: {}", e),
            "Failed to delete deliverable selection",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    Ok(
        HttpResponse::Ok().json(DeleteStudentDeliverableSelectionResponse {
            message: "Deliverable selection removed successfully".to_string(),
        }),
    )
}
