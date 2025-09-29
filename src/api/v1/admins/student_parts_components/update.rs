use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::models::student_parts_component::StudentPartsComponent;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateStudentPartComponentScheme {
    #[schema(example = "10")]
    pub quantity: i32,
}

#[utoipa::path(
    patch,
    path = "/v1/admins/student-parts-components/{id}",
    request_body = UpdateStudentPartComponentScheme,
    responses(
        (status = 200, description = "Student part component relationship updated successfully"),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Relationship not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student parts-components management",
)]
/// Updates the quantity of a component in a student part.
///
/// This endpoint allows authenticated admins to modify the quantity of a component in a student part by ID.
pub(super) async fn update_student_part_component_handler(
    path: web::Path<i32>,
    payload: Json<UpdateStudentPartComponentScheme>,
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();
    let scheme = payload.into_inner();
    let original_payload = Json(UpdateStudentPartComponentScheme {
        quantity: scheme.quantity,
    });

    // Find the existing relationship by ID
    let mut rows = StudentPartsComponent::where_col(|spc| spc.id.equal(id))
        .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to load student part component relationship: {}", e),
            "Failed to update relationship",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &original_payload,
        )
    })?;

    let mut relationship_state = match rows.pop() {
        Some(s) => s,
        None => return Err("Relationship not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // Update the quantity
    relationship_state.quantity = scheme.quantity;

    relationship_state.save(&data.db).await.map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to update student part component relationship: {}", e),
            "Failed to update relationship",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &original_payload,
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}
