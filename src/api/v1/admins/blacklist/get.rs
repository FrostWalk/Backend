use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::blacklist_repository;
use crate::models::blacklist::Blacklist;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetBlacklistResponse {
    pub blacklist: Blacklist,
}

#[utoipa::path(
    get,
    path = "/v1/admins/blacklist/{blacklist_id}",
    responses(
        (status = 200, description = "Blacklist entry retrieved successfully", body = GetBlacklistResponse),
        (status = 404, description = "Blacklist entry not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    params(
        ("blacklist_id" = i32, Path, description = "Blacklist entry id")
    ),
    security(("AdminAuth" = [])),
    tag = "Admin blacklist management",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(in crate::api::v1) async fn get_blacklist_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let blacklist_id = path.into_inner();

    let entry = blacklist_repository::get_by_id(&data.db, blacklist_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve blacklist entry from database: {}", e),
                "Failed to retrieve blacklist entry",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .ok_or_else(|| "Blacklist entry not found".to_json_error(StatusCode::NOT_FOUND))?;

    Ok(HttpResponse::Ok().json(GetBlacklistResponse {
        blacklist: DbState::into_inner(entry),
    }))
}
