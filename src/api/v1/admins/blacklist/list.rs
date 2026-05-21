use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::blacklist_repository;
use crate::models::blacklist::Blacklist;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::HttpResponse;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListBlacklistResponse {
    pub blacklist: Vec<Blacklist>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/blacklist",
    responses(
        (status = 200, description = "Blacklist entries retrieved successfully", body = ListBlacklistResponse),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin blacklist management",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(in crate::api::v1) async fn list_blacklist_handler(
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let rows = blacklist_repository::get_all(&data.db).await.map_err(|e| {
        error_with_log_id(
            format!("unable to retrieve blacklist entries from database: {}", e),
            "Failed to retrieve blacklist entries",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let blacklist = rows.into_iter().map(DbState::into_inner).collect();

    Ok(HttpResponse::Ok().json(ListBlacklistResponse { blacklist }))
}
