use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::fairs_repository;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use chrono::Utc;

#[utoipa::path(
    post,
    path = "/v1/admins/fairs/{fair_id}/enable",
    params(("fair_id" = i32, Path, description = "Fair ID")),
    responses(
        (status = 200, description = "Fair enabled — start_date set to now"),
        (status = 400, description = "end_date is in the past; update dates first", body = JsonError),
        (status = 404, description = "Fair not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("AdminAuth" = [])),
    tag = "Fairs management",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(in crate::api::v1) async fn enable_fair_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let fair_id = path.into_inner();

    let fair_state = fairs_repository::get_by_id(&data.db, fair_id)
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

    if fair_state.end_date <= Utc::now() {
        return Err(
            "The fair's end_date is in the past. Update the fair dates before enabling."
                .to_json_error(StatusCode::BAD_REQUEST),
        );
    }

    fairs_repository::enable(&data.db, fair_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("Failed to enable fair {}: {}", fair_id, e),
                "Failed to enable fair",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &fair_id,
            )
        })?;

    Ok(HttpResponse::Ok().finish())
}
