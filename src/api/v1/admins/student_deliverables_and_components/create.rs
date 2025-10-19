use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::models::student_deliverables_component::StudentDeliverablesComponent;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateStudentDeliverableComponentScheme {
    #[schema(example = "1")]
    pub student_deliverable_id: i32,
    #[schema(example = "2")]
    pub student_deliverable_component_id: i32,
    #[schema(example = "5")]
    pub quantity: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateStudentDeliverableComponentResponse {
    #[schema(example = "123")]
    pub id: i32,
    #[schema(example = "1")]
    pub student_deliverable_id: i32,
    #[schema(example = "2")]
    pub student_deliverable_component_id: i32,
    #[schema(example = "5")]
    pub quantity: i32,
}

#[utoipa::path(
    post,
    path = "/v1/admins/student-deliverables-components",
    request_body = CreateStudentDeliverableComponentScheme,
    responses(
        (status = 200, description = "Student deliverable-component relationship created successfully", body = CreateStudentDeliverableComponentResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 409, description = "Relationship already exists", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverables-components management",
)]
/// Creates a new student deliverable-component relationship.
///
/// This endpoint allows authenticated admins to add components to student deliverables with specified quantities.
pub(super) async fn create_student_deliverable_component_handler(
    body: Json<CreateStudentDeliverableComponentScheme>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // Check if relationship already exists
    let existing = StudentDeliverablesComponent::where_col(|spc| {
        spc.student_deliverable_id.equal(body.student_deliverable_id)
    })
    .where_col(|spc| {
        spc.student_deliverable_component_id
            .equal(body.student_deliverable_component_id)
    })
    .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to check existing relationship: {}", e),
            "Failed to create relationship",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    if !existing.is_empty() {
        return Err("Relationship already exists".to_json_error(StatusCode::CONFLICT));
    }

    let mut state = DbState::new_uncreated(StudentDeliverablesComponent {
        id: 0,
        student_deliverable_id: body.student_deliverable_id,
        student_deliverable_component_id: body.student_deliverable_component_id,
        quantity: body.quantity,
    });

    if let Err(e) = state.save(&data.db).await {
        return Err(error_with_log_id_and_payload(
            format!(
                "unable to create student deliverable component relationship: {}",
                e
            ),
            "Failed to create relationship",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        ));
    }

    Ok(
        HttpResponse::Ok().json(CreateStudentDeliverableComponentResponse {
            id: state.id,
            student_deliverable_id: body.student_deliverable_id,
            student_deliverable_component_id: body.student_deliverable_component_id,
            quantity: body.quantity,
        }),
    )
}
