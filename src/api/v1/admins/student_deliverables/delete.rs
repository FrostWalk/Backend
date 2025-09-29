use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::models::student_deliverable::StudentDeliverable;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};

#[utoipa::path(
    delete,
    path = "/v1/admins/student-deliverables/{id}",
    responses(
        (status = 200, description = "Student deliverable deleted successfully"),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Student deliverable not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverables management",
)]
/// Deletes a student deliverable.
///
/// This endpoint allows authenticated admins to remove a student deliverable by ID.
pub(super) async fn delete_student_deliverable_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    // Find the existing part by ID
    let mut rows = StudentDeliverable::where_col(|sp| sp.student_deliverable_id.equal(id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to load student deliverable: {}", e),
                "Failed to delete part",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut part_state = match rows.pop() {
        Some(s) => s,
        None => return Err("Student part not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    part_state.delete(&data.db).await.map_err(|e| {
        error_with_log_id(
            format!("unable to delete student deliverable: {}", e),
            "Failed to delete part",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}
