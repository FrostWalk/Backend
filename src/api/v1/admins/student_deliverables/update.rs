use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::student_deliverables_repository;
use actix_web::http::StatusCode;
use actix_web::web::Path;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateStudentDeliverableScheme {
    #[schema(example = "Updated Motor")]
    pub name: String,
}

#[utoipa::path(
    patch,
    path = "/v1/admins/student-deliverables/{id}",
    request_body = UpdateStudentDeliverableScheme,
    responses(
        (status = 200, description = "Student deliverable updated successfully"),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Student deliverable not found", body = JsonError),
        (status = 409, description = "Deliverable with this name already exists for the project", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverables management",
)]
/// Updates a student deliverable.
///
/// This endpoint allows authenticated admins to modify the name of a student deliverable by ID.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn update_student_deliverable_handler(
    path: Path<i32>, body: Json<UpdateStudentDeliverableScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    // Find the existing deliverable by ID
    let deliverable_state = student_deliverables_repository::get_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to load student deliverable: {}", e),
                "Failed to update deliverable",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?
        .ok_or_else(|| "Student deliverable not found".to_json_error(StatusCode::NOT_FOUND))?;

    // Check if another deliverable with this name already exists for the same project
    let exists = student_deliverables_repository::check_name_exists_excluding(
        &data.db,
        deliverable_state.project_id,
        &body.name,
        id,
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to check existing student deliverable: {}", e),
            "Failed to update deliverable",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    if exists {
        return Err("Deliverable with this name already exists for the project"
            .to_json_error(StatusCode::CONFLICT));
    }

    // Update the name using repository function
    student_deliverables_repository::update_by_id(&data.db, id, &body.name)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to update student deliverable: {}", e),
                "Failed to update deliverable",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    Ok(HttpResponse::Ok().finish())
}
