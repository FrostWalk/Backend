use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, error_with_log_id_and_payload, JsonError};
use crate::database::repositories::{
    groups_repository, projects_repository, student_deliverable_selections_repository,
    student_deliverables_repository,
};
use crate::jwt::get_user::LoggedUser;
use crate::models::student_deliverable_selection::StudentDeliverableSelection;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateStudentDeliverableSelectionRequest {
    #[schema(example = 8)]
    pub student_deliverable_id: i32,
    #[schema(example = 2)]
    pub project_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateStudentDeliverableSelectionResponse {
    pub student_deliverable_selection_id: i32,
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/v1/students/deliverable-selection",
    request_body = CreateStudentDeliverableSelectionRequest,
    responses(
        (status = 201, description = "Deliverable selected successfully", body = CreateStudentDeliverableSelectionResponse),
        (status = 400, description = "Invalid request or deadline passed", body = JsonError),
        (status = 403, description = "Student not in a group for this project", body = JsonError),
        (status = 404, description = "Deliverable or project not found", body = JsonError),
        (status = 409, description = "Student already has a selection for this project", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Student Deliverable Selections",
)]
/// Create a student deliverable selection (requires group membership)
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(in crate::api::v1) async fn create_student_deliverable_selection(
    req: HttpRequest, body: Json<CreateStudentDeliverableSelectionRequest>, data: Data<AppData>,
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

    // 1. CRITICAL: Verify the student is a member of a group in the specified project (Q1 requirement)
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
            "You must be a member of a group in this project to select a deliverable",
            StatusCode::FORBIDDEN,
            log::Level::Warn,
        ));
    }

    // 2. Verify the student hasn't already selected a deliverable for this project
    let has_selection = student_deliverable_selections_repository::has_selection_for_project(
        &data.db,
        user.student_id,
        body.project_id,
    )
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
            format!(
                "Student {} already has a deliverable selection for project {}",
                user.student_id, body.project_id
            ),
            "You have already selected a deliverable for this project. Use PATCH to update it.",
            StatusCode::CONFLICT,
            log::Level::Warn,
        ));
    }

    // 3. Verify the student_deliverable_id exists and belongs to the same project
    let deliverable =
        student_deliverables_repository::get_by_id(&data.db, body.student_deliverable_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("Database error fetching deliverable: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?
            .ok_or_else(|| {
                error_with_log_id(
                    format!(
                        "Student deliverable {} not found",
                        body.student_deliverable_id
                    ),
                    "Deliverable not found",
                    StatusCode::NOT_FOUND,
                    log::Level::Warn,
                )
            })
            .map(DbState::into_inner)?;

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
    let project = projects_repository::get_by_id(&data.db, body.project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("Database error fetching project: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .ok_or_else(|| {
            error_with_log_id(
                format!("Project {} not found", body.project_id),
                "Project not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            )
        })
        .map(DbState::into_inner)?;

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

    // Create the selection using repository function
    let selection = StudentDeliverableSelection {
        student_deliverable_selection_id: 0,
        student_id: user.student_id,
        student_deliverable_id: body.student_deliverable_id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match student_deliverable_selections_repository::create(&data.db, selection).await {
        Ok(selection_state) => {
            let selection = DbState::into_inner(selection_state);
            Ok(
                HttpResponse::Created().json(CreateStudentDeliverableSelectionResponse {
                    student_deliverable_selection_id: selection.student_deliverable_selection_id,
                    message: "Deliverable selected successfully".to_string(),
                }),
            )
        }
        Err(e) => Err(error_with_log_id_and_payload(
            format!("Failed to create student deliverable selection: {}", e),
            "Failed to create deliverable selection",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )),
    }
}
