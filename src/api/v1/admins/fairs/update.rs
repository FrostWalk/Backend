use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::fairs_repository;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateFairRequest {
    #[schema(example = "Updated fair description")]
    pub details: Option<String>,
    #[schema(value_type = Option<String>, example = "2026-06-01T09:00:00Z")]
    pub start_date: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>, example = "2026-06-01T18:00:00Z")]
    pub end_date: Option<DateTime<Utc>>,
    #[schema(example = 5)]
    pub min_purchases: Option<i32>,
}

#[utoipa::path(
    patch,
    path = "/v1/admins/fairs/{fair_id}",
    params(("fair_id" = i32, Path, description = "Fair ID")),
    request_body = UpdateFairRequest,
    responses(
        (status = 200, description = "Fair updated successfully"),
        (status = 400, description = "Invalid request data", body = JsonError),
        (status = 404, description = "Fair not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("AdminAuth" = [])),
    tag = "Fairs management",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(in crate::api::v1) async fn update_fair_handler(
    path: Path<i32>, body: Json<UpdateFairRequest>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let fair_id = path.into_inner();

    let mut state = fairs_repository::get_by_id(&data.db, fair_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("DB error fetching fair {}: {}", fair_id, e),
                "Failed to fetch fair",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?
        .ok_or_else(|| "Fair not found".to_json_error(StatusCode::NOT_FOUND))?;

    if let Some(details) = &body.details {
        if details.is_empty() {
            return Err("Details cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
        }
        state.details = details.clone();
    }
    if let Some(start_date) = body.start_date {
        state.start_date = start_date;
    }
    if let Some(end_date) = body.end_date {
        state.end_date = end_date;
    }
    if let Some(min_purchases) = body.min_purchases {
        if min_purchases < 1 {
            return Err("min_purchases must be at least 1".to_json_error(StatusCode::BAD_REQUEST));
        }
        state.min_purchases = min_purchases;
    }

    if state.end_date <= state.start_date {
        return Err("end_date must be after start_date".to_json_error(StatusCode::BAD_REQUEST));
    }

    fairs_repository::update(&data.db, &mut state)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("Failed to update fair {}: {}", fair_id, e),
                "Failed to update fair",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    Ok(HttpResponse::Ok().finish())
}
