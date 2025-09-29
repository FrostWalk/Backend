use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::models::student_parts_component::StudentPartsComponent;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateStudentPartComponentScheme {
    #[schema(example = "1")]
    pub student_part_id: i32,
    #[schema(example = "2")]
    pub students_component_id: i32,
    #[schema(example = "5")]
    pub quantity: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateStudentPartComponentResponse {
    #[schema(example = "123")]
    pub id: i32,
    #[schema(example = "1")]
    pub student_part_id: i32,
    #[schema(example = "2")]
    pub students_component_id: i32,
    #[schema(example = "5")]
    pub quantity: i32,
}

#[utoipa::path(
    post,
    path = "/v1/admins/student-parts-components",
    request_body = CreateStudentPartComponentScheme,
    responses(
        (status = 200, description = "Student part component relationship created successfully", body = CreateStudentPartComponentResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 409, description = "Relationship already exists", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student parts-components management",
)]
/// Creates a new student part-component relationship.
///
/// This endpoint allows authenticated admins to add components to student parts with specified quantities.
pub(super) async fn create_student_part_component_handler(
    payload: Json<CreateStudentPartComponentScheme>,
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let scheme = payload.into_inner();
    let original_payload = Json(CreateStudentPartComponentScheme {
        student_part_id: scheme.student_part_id,
        students_component_id: scheme.students_component_id,
        quantity: scheme.quantity,
    });

    // Check if relationship already exists
    let existing = StudentPartsComponent::where_col(|spc| spc.student_part_id.equal(scheme.student_part_id))
        .where_col(|spc| spc.students_component_id.equal(scheme.students_component_id))
        .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to check existing relationship: {}", e),
            "Failed to create relationship",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &original_payload,
        )
    })?;

    if !existing.is_empty() {
        return Err("Relationship already exists".to_json_error(StatusCode::CONFLICT));
    }

    let mut state = DbState::new_uncreated(StudentPartsComponent {
        id: 0,
        student_part_id: scheme.student_part_id,
        students_component_id: scheme.students_component_id,
        quantity: scheme.quantity,
    });

    if let Err(e) = state.save(&data.db).await {
        return Err(error_with_log_id_and_payload(
            format!("unable to create student part component relationship: {}", e),
            "Failed to create relationship",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &original_payload,
        ));
    }

    Ok(HttpResponse::Ok().json(CreateStudentPartComponentResponse {
        id: state.id,
        student_part_id: scheme.student_part_id,
        students_component_id: scheme.students_component_id,
        quantity: scheme.quantity,
    }))
}
