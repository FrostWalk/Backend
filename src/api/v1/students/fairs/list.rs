use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::{fairs_repository, groups_repository, transactions_repository};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path, Query};
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize)]
pub(crate) struct ListTransactionsQuery {
    pub group_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct TransactionEntry {
    pub transaction_id: i32,
    pub group_deliverable_selection_id: i32,
    pub group_deliverable_component_id: i32,
    #[schema(value_type = String)]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListTransactionsResponse {
    pub transactions: Vec<TransactionEntry>,
    pub min_purchases_required: i32,
    pub purchases_fulfilled: bool,
}

#[utoipa::path(
    get,
    path = "/v1/students/fairs/{fair_id}/transactions",
    params(
        ("fair_id" = i32, Path, description = "Fair ID"),
        ("group_id" = i32, Query, description = "Group ID"),
    ),
    responses(
        (status = 200, description = "List of transactions made by the group", body = ListTransactionsResponse),
        (status = 403, description = "Not a member of the group", body = JsonError),
        (status = 404, description = "Fair not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("StudentAuth" = [])),
    tag = "Fair transactions",
)]
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(in crate::api::v1) async fn list_transactions_handler(
    req: HttpRequest, path: Path<i32>, query: Query<ListTransactionsQuery>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let fair_id = path.into_inner();
    let group_id = query.group_id;
    let student = req.extensions().get_student().map_err(|e| {
        error_with_log_id_and_payload(
            format!("Failed to extract student: {}", e),
            "Authentication error",
            StatusCode::UNAUTHORIZED,
            log::Level::Warn,
            &fair_id,
        )
    })?;

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

    let members = groups_repository::get_members(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("DB error fetching group members: {}", e),
                "Failed to fetch group members",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &group_id,
            )
        })?;

    let is_member = members.iter().any(|m| m.student_id == student.student_id);

    if !is_member {
        return Err("You are not a member of this group".to_json_error(StatusCode::FORBIDDEN));
    }

    let raw_txns = transactions_repository::get_by_fair_and_buyer(&data.db, fair_id, group_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("DB error fetching transactions: {}", e),
                "Failed to fetch transactions",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &fair_id,
            )
        })?;

    let distinct_count: i64 = {
        use std::collections::HashSet;
        let pairs: HashSet<(i32, i32)> = raw_txns
            .iter()
            .map(|t| {
                (
                    t.group_deliverable_selection_id,
                    t.group_deliverable_component_id,
                )
            })
            .collect();
        pairs.len() as i64
    };

    let transactions = raw_txns
        .into_iter()
        .map(|t| TransactionEntry {
            transaction_id: t.transaction_id,
            group_deliverable_selection_id: t.group_deliverable_selection_id,
            group_deliverable_component_id: t.group_deliverable_component_id,
            timestamp: t.timestamp,
        })
        .collect();

    Ok(HttpResponse::Ok().json(ListTransactionsResponse {
        transactions,
        min_purchases_required: fair_state.min_purchases,
        purchases_fulfilled: distinct_count >= fair_state.min_purchases as i64,
    }))
}
