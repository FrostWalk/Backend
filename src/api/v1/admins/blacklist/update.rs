use crate::app_data::AppData;
use crate::common::json_error::{
    error_with_log_id, error_with_log_id_and_payload, JsonError, ToJsonError,
};
use crate::database::repositories::blacklist_repository;
use crate::models::blacklist::Blacklist;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateBlacklistScheme {
    #[schema(example = "Updated reason")]
    pub description: Option<String>,
    #[schema(example = "Mario")]
    pub first_name: Option<String>,
    #[schema(example = "Rossi")]
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct UpdateBlacklistResponse {
    pub blacklist: Blacklist,
}

#[utoipa::path(
    patch,
    path = "/v1/admins/blacklist/{blacklist_id}",
    request_body = UpdateBlacklistScheme,
    responses(
        (status = 200, description = "Blacklist entry updated successfully", body = UpdateBlacklistResponse),
        (status = 400, description = "Invalid request body", body = JsonError),
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
pub(in crate::api::v1) async fn update_blacklist_handler(
    path: Path<i32>, body: Json<UpdateBlacklistScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    if body.description.is_none() && body.first_name.is_none() && body.last_name.is_none() {
        return Err("At least one field must be provided".to_json_error(StatusCode::BAD_REQUEST));
    }

    let blacklist_id = path.into_inner();

    let updated = blacklist_repository::update_by_id(
        &data.db,
        blacklist_id,
        body.description.clone(),
        body.first_name.clone(),
        body.last_name.clone(),
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to update blacklist entry in database: {}", e),
            "Failed to update blacklist entry",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?
    .ok_or_else(|| {
        error_with_log_id(
            format!("blacklist entry {} not found", blacklist_id),
            "Blacklist entry not found",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        )
    })?;

    Ok(HttpResponse::Ok().json(UpdateBlacklistResponse {
        blacklist: DbState::into_inner(updated),
    }))
}
