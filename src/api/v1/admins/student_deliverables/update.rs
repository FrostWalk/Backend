use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::models::student_deliverable::StudentDeliverable;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{web, HttpResponse};
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
pub(super) async fn update_student_deliverable_handler(
    path: web::Path<i32>, payload: Json<UpdateStudentDeliverableScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();
    let scheme = payload.into_inner();
    let original_payload = Json(UpdateStudentDeliverableScheme {
        name: scheme.name.clone(),
    });

    // Find the existing part by ID
    let mut rows = StudentDeliverable::where_col(|sp| sp.student_deliverable_id.equal(id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to load student deliverable: {}", e),
                "Failed to update part",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &original_payload,
            )
        })?;

    let mut part_state = match rows.pop() {
        Some(s) => s,
        None => return Err("Student part not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // Check if another part with this name already exists for the same project
    let existing = StudentDeliverable::where_col(|sp| sp.project_id.equal(part_state.project_id))
        .where_col(|sp| sp.name.equal(&scheme.name))
        .where_col(|sp| sp.student_deliverable_id.not_equal(id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to check existing student deliverable: {}", e),
                "Failed to update part",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &original_payload,
            )
        })?;

    if !existing.is_empty() {
        return Err("Part with this name already exists for the project"
            .to_json_error(StatusCode::CONFLICT));
    }

    // Update the name
    part_state.name = scheme.name;

    part_state.save(&data.db).await.map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to update student deliverable: {}", e),
            "Failed to update part",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &original_payload,
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}
