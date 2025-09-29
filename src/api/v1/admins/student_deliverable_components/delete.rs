use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};

#[utoipa::path(
    delete,
    path = "/v1/admins/student-deliverable-components/{id}",
    responses(
        (status = 200, description = "Student component deleted successfully"),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Student component not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverable components management",
)]
/// Deletes a student component.
///
/// This endpoint allows authenticated admins to remove a student component by ID.
pub(super) async fn delete_student_component_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    // Find the existing component by ID
    let mut rows =
        StudentDeliverableComponent::where_col(|sc| sc.student_deliverable_component_id.equal(id))
            .run(&data.db)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to load student component: {}", e),
                    "Failed to delete component",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    let mut component_state = match rows.pop() {
        Some(s) => s,
        None => return Err("Student component not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    component_state.delete(&data.db).await.map_err(|e| {
        error_with_log_id(
            format!("unable to delete student component: {}", e),
            "Failed to delete component",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}
