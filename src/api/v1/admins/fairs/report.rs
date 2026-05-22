use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::fairs_repository;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path, Query};
use actix_web::HttpResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use utoipa::ToSchema;

#[derive(Debug, Deserialize)]
pub(crate) struct ReportQuery {
    pub group_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct SaleEntry {
    pub transaction_id: i32,
    pub component_name: String,
    pub buyer_group_name: String,
    #[schema(value_type = String)]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct PurchaseEntry {
    pub transaction_id: i32,
    pub component_name: String,
    pub seller_group_name: String,
    #[schema(value_type = String)]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupFairReport {
    pub group_id: i32,
    pub group_name: String,
    pub fair_id: i32,
    pub sold: Vec<SaleEntry>,
    pub bought: Vec<PurchaseEntry>,
    pub min_purchases_required: i32,
    pub purchases_fulfilled: bool,
}

#[utoipa::path(
    get,
    path = "/v1/admins/fairs/{fair_id}/report",
    params(
        ("fair_id" = i32, Path, description = "Fair ID"),
        ("group_id" = i32, Query, description = "Group ID to generate the report for"),
    ),
    responses(
        (status = 200, description = "Sales report for the group", body = GroupFairReport),
        (status = 404, description = "Fair or group not found", body = JsonError),
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
pub(in crate::api::v1) async fn fair_report_handler(
    path: Path<i32>, query: Query<ReportQuery>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let fair_id = path.into_inner();
    let group_id = query.group_id;

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

    let pool = data.db.as_sqlx_pool();

    let group_name = sqlx::query_scalar::<_, String>("SELECT name FROM groups WHERE group_id = $1")
        .bind(group_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("DB error fetching group {}: {}", group_id, e),
                "Failed to fetch group",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &group_id,
            )
        })?
        .ok_or_else(|| "Group not found".to_json_error(StatusCode::NOT_FOUND))?;

    let sold_rows = sqlx::query(
        r#"
        SELECT
            t.transaction_id,
            gdc.name AS component_name,
            bg.name  AS buyer_group_name,
            t.timestamp
        FROM transactions t
        JOIN group_deliverable_selections gds
            ON t.group_deliverable_selection_id = gds.group_deliverable_selection_id
        JOIN group_deliverable_components gdc
            ON t.group_deliverable_component_id = gdc.group_deliverable_component_id
        JOIN groups bg ON t.buyer_group_id = bg.group_id
        WHERE gds.group_id = $1 AND t.fair_id = $2
        ORDER BY t.timestamp
        "#,
    )
    .bind(group_id)
    .bind(fair_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("DB error fetching sold transactions: {}", e),
            "Failed to fetch sales",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &fair_id,
        )
    })?;

    let bought_rows = sqlx::query(
        r#"
        SELECT
            t.transaction_id,
            gdc.name AS component_name,
            sg.name  AS seller_group_name,
            t.timestamp
        FROM transactions t
        JOIN group_deliverable_selections gds
            ON t.group_deliverable_selection_id = gds.group_deliverable_selection_id
        JOIN group_deliverable_components gdc
            ON t.group_deliverable_component_id = gdc.group_deliverable_component_id
        JOIN groups sg ON gds.group_id = sg.group_id
        WHERE t.buyer_group_id = $1 AND t.fair_id = $2
        ORDER BY t.timestamp
        "#,
    )
    .bind(group_id)
    .bind(fair_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("DB error fetching bought transactions: {}", e),
            "Failed to fetch purchases",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &fair_id,
        )
    })?;

    let distinct_row: (Option<i64>,) = sqlx::query_as(
        r#"
        SELECT COUNT(DISTINCT (t.group_deliverable_selection_id, t.group_deliverable_component_id))
        FROM transactions t
        WHERE t.fair_id = $1 AND t.buyer_group_id = $2
        "#,
    )
    .bind(fair_id)
    .bind(group_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("DB error counting purchases: {}", e),
            "Failed to count purchases",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &fair_id,
        )
    })?;
    let distinct_purchases: i64 = distinct_row.0.unwrap_or(0);

    let sold: Vec<SaleEntry> = sold_rows
        .into_iter()
        .map(|r| SaleEntry {
            transaction_id: r.get("transaction_id"),
            component_name: r.get("component_name"),
            buyer_group_name: r.get("buyer_group_name"),
            timestamp: r.get("timestamp"),
        })
        .collect();

    let bought: Vec<PurchaseEntry> = bought_rows
        .into_iter()
        .map(|r| PurchaseEntry {
            transaction_id: r.get("transaction_id"),
            component_name: r.get("component_name"),
            seller_group_name: r.get("seller_group_name"),
            timestamp: r.get("timestamp"),
        })
        .collect();

    Ok(HttpResponse::Ok().json(GroupFairReport {
        group_id,
        group_name,
        fair_id,
        sold,
        bought,
        min_purchases_required: fair_state.min_purchases,
        purchases_fulfilled: distinct_purchases >= fair_state.min_purchases as i64,
    }))
}
