use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{complaints_repository, groups_repository};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupComplaintsResponse {
    pub group_id: i32,
    pub complaints_filed: Vec<ComplaintsFiledItem>,
    pub complaints_received: Vec<ComplaintsReceivedItem>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ComplaintsFiledItem {
    pub complaint_id: i32,
    pub transaction_id: i32,
    pub to_group_id: i32,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ComplaintsReceivedItem {
    pub complaint_id: i32,
    pub transaction_id: i32,
    pub from_group_id: i32,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/groups/{group_id}/complaints",
    params(("group_id" = i32, Path, description = "Group ID")),
    responses(
        (status = 200, description = "Complaints grouped by filed and received", body = GroupComplaintsResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Group not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Groups management",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_group_complaints(
    req: HttpRequest, path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _admin = req.extensions().get_admin().map_err(|_| {
        error_with_log_id(
            "entered a protected route without an admin loaded in the request",
            "Authentication error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let group_id = path.into_inner();

    let group_exists = groups_repository::get_by_id(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .is_some();
    if !group_exists {
        return Err(error_with_log_id(
            format!("group {} not found", group_id),
            "Group not found",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        ));
    }

    let filed = complaints_repository::get_filed_by_group(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to fetch filed complaints for group {}: {}",
                    group_id, e
                ),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let received = complaints_repository::get_received_by_group(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to fetch received complaints for group {}: {}",
                    group_id, e
                ),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let complaints_filed = filed
        .into_iter()
        .map(|complaint_state| {
            let complaint = DbState::into_inner(complaint_state);
            ComplaintsFiledItem {
                complaint_id: complaint.complaint_id,
                transaction_id: complaint.transaction_id,
                to_group_id: complaint.to_group_id,
                text: complaint.text,
                created_at: complaint.created_at,
            }
        })
        .collect::<Vec<_>>();

    let complaints_received = received
        .into_iter()
        .map(|complaint_state| {
            let complaint = DbState::into_inner(complaint_state);
            ComplaintsReceivedItem {
                complaint_id: complaint.complaint_id,
                transaction_id: complaint.transaction_id,
                from_group_id: complaint.from_group_id,
                text: complaint.text,
                created_at: complaint.created_at,
            }
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(GroupComplaintsResponse {
        group_id,
        complaints_filed,
        complaints_received,
    }))
}
