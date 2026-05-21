use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::fairs_repository;
use crate::models::fair::Fair;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct FairResponse {
    pub fair_id: i32,
    pub project_id: i32,
    pub details: String,
    #[schema(value_type = String)]
    pub start_date: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String)]
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub min_purchases: i32,
    pub is_active: bool,
}

impl From<DbState<Fair>> for FairResponse {
    fn from(state: DbState<Fair>) -> Self {
        let active = fairs_repository::is_active(&state);
        let f = DbState::into_inner(state);
        Self {
            fair_id: f.fair_id,
            project_id: f.project_id,
            details: f.details,
            start_date: f.start_date,
            end_date: f.end_date,
            min_purchases: f.min_purchases,
            is_active: active,
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/admins/fairs/{fair_id}",
    params(("fair_id" = i32, Path, description = "Fair ID")),
    responses(
        (status = 200, description = "Fair details", body = FairResponse),
        (status = 404, description = "Fair not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("AdminAuth" = [])),
    tag = "Fairs management",
)]
#[actix_web_grants::protect(any(
    "ROLE_ADMIN_ROOT",
    "ROLE_ADMIN_PROFESSOR",
    "ROLE_ADMIN_COORDINATOR"
))]
pub(in crate::api::v1) async fn get_fair_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let fair_id = path.into_inner();

    let state = fairs_repository::get_by_id(&data.db, fair_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("DB error fetching fair {}: {}", fair_id, e),
                "Failed to fetch fair",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &fair_id,
            )
        })?
        .ok_or_else(|| "Fair not found".to_json_error(StatusCode::NOT_FOUND))?;

    Ok(HttpResponse::Ok().json(FairResponse::from(state)))
}

#[utoipa::path(
    get,
    path = "/v1/admins/fairs/project/{project_id}",
    params(("project_id" = i32, Path, description = "Project ID")),
    responses(
        (status = 200, description = "Fair for the project", body = FairResponse),
        (status = 404, description = "No fair for this project", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("AdminAuth" = [])),
    tag = "Fairs management",
)]
#[actix_web_grants::protect(any(
    "ROLE_ADMIN_ROOT",
    "ROLE_ADMIN_PROFESSOR",
    "ROLE_ADMIN_COORDINATOR"
))]
pub(in crate::api::v1) async fn get_fair_by_project_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();

    let state = fairs_repository::get_by_project_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("DB error fetching fair for project {}: {}", project_id, e),
                "Failed to fetch fair",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &project_id,
            )
        })?
        .ok_or_else(|| "No fair found for this project".to_json_error(StatusCode::NOT_FOUND))?;

    Ok(HttpResponse::Ok().json(FairResponse::from(state)))
}
