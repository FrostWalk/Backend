use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{
    projects_repository, student_deliverable_selections_repository, students_repository,
};
use crate::jwt::get_user::LoggedUser;
use crate::models::student_deliverable::StudentDeliverable;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentDeliverableSelectionsResponse {
    pub project_id: i32,
    pub project_name: String,
    pub selections: Vec<StudentSelectionInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentSelectionInfo {
    pub student_deliverable_selection_id: i32,
    pub student_id: i32,
    pub student_name: String,
    pub student_email: String,
    pub student_deliverable_id: i32,
    pub student_deliverable_name: String,
}

#[utoipa::path(
    get,
    path = "/v1/admins/projects/{project_id}/student-deliverable-selections",
    responses(
        (status = 200, description = "List of student deliverable selections", body = StudentDeliverableSelectionsResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Student Deliverable Selections",
)]
/// List all student deliverable selections for a project
///
/// This endpoint allows admins to view all student deliverable selections for a specific project,
/// including which deliverables each student has chosen.
pub(super) async fn get_student_deliverable_selections(
    req: HttpRequest, 
    path: Path<i32>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _admin = match req.extensions().get_admin() {
        Ok(admin) => admin,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without an admin loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let project_id = path.into_inner();

    // Verify the project exists
    let project_state = projects_repository::get_by_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let project = match project_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("project {} not found", project_id),
                "Project not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Get all student deliverable selections for this project
    let selections =
        student_deliverable_selections_repository::get_by_project_id(&data.db, project_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!(
                        "unable to fetch student deliverable selections for project {}: {}",
                        project_id, e
                    ),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    let mut selection_infos = Vec::new();

    for selection_state in selections {
        let selection = DbState::into_inner(selection_state);

        // Get student details
        let student_state = students_repository::get_by_id(&data.db, selection.student_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to fetch student {}: {}", selection.student_id, e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

        let student = match student_state {
            Some(state) => DbState::into_inner(state),
            None => {
                // Log warning but continue processing other selections
                log::warn!(
                    "Student {} not found for selection {}",
                    selection.student_id,
                    selection.student_deliverable_selection_id
                );
                continue;
            }
        };

        // Get the deliverable details
        let deliverable_state =
            StudentDeliverable::find_by_id(&data.db, selection.student_deliverable_id)
                .await
                .map_err(|e| {
                    error_with_log_id(
                        format!(
                            "unable to fetch student deliverable {}: {}",
                            selection.student_deliverable_id, e
                        ),
                        "Database error",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                    )
                })?;

        let deliverable = match deliverable_state {
            Some(state) => DbState::into_inner(state),
            None => {
                // Log warning but continue processing other selections
                log::warn!(
                    "Student deliverable {} not found for selection {}",
                    selection.student_deliverable_id,
                    selection.student_deliverable_selection_id
                );
                continue;
            }
        };

        selection_infos.push(StudentSelectionInfo {
            student_deliverable_selection_id: selection.student_deliverable_selection_id,
            student_id: student.student_id,
            student_name: format!("{} {}", student.first_name, student.last_name),
            student_email: student.email,
            student_deliverable_id: selection.student_deliverable_id,
            student_deliverable_name: deliverable.name,
        });
    }

    Ok(
        HttpResponse::Ok().json(StudentDeliverableSelectionsResponse {
            project_id,
            project_name: project.name,
            selections: selection_infos,
        }),
    )
}
