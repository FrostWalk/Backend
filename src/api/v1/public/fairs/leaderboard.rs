use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::fairs_repository;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use serde::Serialize;
use sqlx::Row;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct LeaderboardEntry {
    pub rank: i32,
    pub group_id: i32,
    pub group_name: String,
    pub total_sales: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct LeaderboardResponse {
    pub fair_id: i32,
    pub is_active: bool,
    pub leaderboard: Vec<LeaderboardEntry>,
}

#[utoipa::path(
    get,
    path = "/v1/fairs/{fair_id}/leaderboard",
    params(("fair_id" = i32, Path, description = "Fair ID")),
    responses(
        (status = 200, description = "Sales leaderboard for the fair", body = LeaderboardResponse),
        (status = 404, description = "Fair not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    tag = "Fairs leaderboard",
)]
pub(in crate::api::v1) async fn leaderboard_handler(
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

    let active = fairs_repository::is_active(&fair_state);
    let pool = data.db.as_sqlx_pool();

    let rows = sqlx::query(
        r#"
        SELECT
            g.group_id,
            g.name AS group_name,
            COUNT(t.transaction_id) AS total_sales
        FROM groups g
        LEFT JOIN group_deliverable_selections gds ON gds.group_id = g.group_id
        LEFT JOIN transactions t
            ON t.group_deliverable_selection_id = gds.group_deliverable_selection_id
            AND t.fair_id = $1
        WHERE g.project_id = (SELECT project_id FROM fairs WHERE fair_id = $1)
        GROUP BY g.group_id, g.name
        ORDER BY total_sales DESC, g.name ASC
        "#,
    )
    .bind(fair_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("DB error fetching leaderboard: {}", e),
            "Failed to fetch leaderboard",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &fair_id,
        )
    })?;

    let leaderboard: Vec<LeaderboardEntry> = rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| LeaderboardEntry {
            rank: (i + 1) as i32,
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            total_sales: r.get::<Option<i64>, _>("total_sales").unwrap_or(0),
        })
        .collect();

    Ok(HttpResponse::Ok().json(LeaderboardResponse {
        fair_id,
        is_active: active,
        leaderboard,
    }))
}
