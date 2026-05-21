use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::fairs_repository;
use crate::models::fair::Fair;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateFairRequest {
    #[schema(example = 1)]
    pub project_id: i32,
    #[schema(example = "End-of-semester component fair")]
    pub details: String,
    #[schema(value_type = String, example = "2026-06-01T09:00:00Z")]
    pub start_date: DateTime<Utc>,
    #[schema(value_type = String, example = "2026-06-01T18:00:00Z")]
    pub end_date: DateTime<Utc>,
    #[schema(example = 3)]
    pub min_purchases: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateFairResponse {
    pub fair_id: i32,
}

#[utoipa::path(
    post,
    path = "/v1/admins/fairs",
    request_body = CreateFairRequest,
    responses(
        (status = 201, description = "Fair created successfully", body = CreateFairResponse),
        (status = 400, description = "Invalid request data", body = JsonError),
        (status = 409, description = "A fair already exists for this project", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("AdminAuth" = [])),
    tag = "Fairs management",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(in crate::api::v1) async fn create_fair_handler(
    body: Json<CreateFairRequest>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    if body.details.is_empty() {
        return Err("Details field is mandatory".to_json_error(StatusCode::BAD_REQUEST));
    }
    if body.end_date <= body.start_date {
        return Err("end_date must be after start_date".to_json_error(StatusCode::BAD_REQUEST));
    }
    if body.min_purchases < 1 {
        return Err("min_purchases must be at least 1".to_json_error(StatusCode::BAD_REQUEST));
    }

    let existing = fairs_repository::get_by_project_id(&data.db, body.project_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("DB error checking existing fair: {}", e),
                "Failed to check existing fair",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    if existing.is_some() {
        return Err(
            "A fair already exists for this project. Use the update endpoint to modify it."
                .to_json_error(StatusCode::CONFLICT),
        );
    }

    let fair = Fair {
        fair_id: 0,
        project_id: body.project_id,
        details: body.details.clone(),
        start_date: body.start_date,
        end_date: body.end_date,
        min_purchases: body.min_purchases,
    };

    let created = fairs_repository::create(&data.db, fair)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("Failed to create fair: {}", e),
                "Failed to create fair",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    Ok(HttpResponse::Created().json(CreateFairResponse {
        fair_id: created.fair_id,
    }))
}
