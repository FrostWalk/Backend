use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, error_with_log_id_and_payload, JsonError};
use crate::database::repositories::{groups_repository, student_deliverable_selections_repository};
use crate::jwt::get_user::LoggedUser;
use crate::models::project::Project;
use crate::models::student_deliverable::StudentDeliverable;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateStudentDeliverableSelectionRequest {
    #[schema(example = 9)]
    pub student_deliverable_id: i32,
    #[schema(example = 2)]
    pub project_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct UpdateStudentDeliverableSelectionResponse {
    pub message: String,
}

#[utoipa::path(
    patch,
    path = "/v1/students/deliverable-selection",
    request_body = UpdateStudentDeliverableSelectionRequest,
    responses(
        (status = 200, description = "Selection updated successfully", body = UpdateStudentDeliverableSelectionResponse),
        (status = 400, description = "Invalid request or deadline passed", body = JsonError),
        (status = 403, description = "Student not in a group for this project", body = JsonError),
        (status = 404, description = "Selection, deliverable or project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Student Deliverable Selections",
)]
/// Update a student deliverable selection
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(in crate::api::v1) async fn update_student_deliverable_selection(
    req: HttpRequest, body: Json<UpdateStudentDeliverableSelectionRequest>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // Get the logged-in user
    let user = req.extensions().get_student().map_err(|_| {
        error_with_log_id(
            "entered a protected route without a user loaded in the request",
            "Authentication error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    // 1. CRITICAL: Verify the student is a member of a group in the specified project
    let is_in_project =
        groups_repository::is_student_in_project(&data.db, user.student_id, body.project_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("Database error checking project membership: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    if !is_in_project {
        return Err(error_with_log_id(
            format!(
                "Student {} is not a member of any group in project {}",
                user.student_id, body.project_id
            ),
            "You must be a member of a group in this project to update a deliverable selection",
            StatusCode::FORBIDDEN,
            log::Level::Warn,
        ));
    }

    // 2. Get the existing selection
    let mut selection_state =
        student_deliverable_selections_repository::get_by_student_and_project(
            &data.db,
            user.student_id,
            body.project_id,
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
                    user.student_id, body.project_id
                ),
                "No deliverable selection found to update",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            )
        })?;

    // 3. Verify the new student_deliverable_id exists and belongs to the same project
    let mut deliverable_rows = StudentDeliverable::where_col(|sd| {
        sd.student_deliverable_id.equal(body.student_deliverable_id)
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
                "Student deliverable {} not found",
                body.student_deliverable_id
            ),
            "Deliverable not found",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        )
    })?;

    let deliverable = DbState::into_inner(deliverable_state);

    if deliverable.project_id != body.project_id {
        return Err(error_with_log_id(
            format!(
                "Deliverable {} belongs to project {}, but request specified project {}",
                body.student_deliverable_id, deliverable.project_id, body.project_id
            ),
            "Deliverable does not belong to the specified project",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    // 4. Verify the project's deliverable_selection_deadline has not passed (if set)
    let mut project_rows = Project::where_col(|p| p.project_id.equal(body.project_id))
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
            format!("Project {} not found", body.project_id),
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
                    deadline, body.project_id
                ),
                "Deliverable selection deadline has passed",
                StatusCode::BAD_REQUEST,
                log::Level::Warn,
            ));
        }
    }

    // Update the selection
    {
        let selection = selection_state.as_mut();
        selection.student_deliverable_id = body.student_deliverable_id;
        selection.updated_at = Utc::now();
    }

    match selection_state.save(&data.db).await {
        Ok(_) => Ok(
            HttpResponse::Ok().json(UpdateStudentDeliverableSelectionResponse {
                message: "Deliverable selection updated successfully".to_string(),
            }),
        ),
        Err(e) => Err(error_with_log_id_and_payload(
            format!("Failed to update student deliverable selection: {}", e),
            "Failed to update deliverable selection",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )),
    }
}
