use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
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
pub(crate) struct ComplaintItem {
    pub complaint_id: i32,
    pub transaction_id: i32,
    pub to_group_id: i32,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/v1/students/groups/{group_id}/complaints",
    params(("group_id" = i32, Path, description = "Group ID")),
    responses(
        (status = 200, description = "Complaints filed by group", body = Vec<ComplaintItem>),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 403, description = "Not a member of this group", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("StudentAuth" = [])),
    tag = "Complaints management",
)]
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(in crate::api::v1) async fn list_group_filed_complaints_handler(
    req: HttpRequest, path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let student = req.extensions().get_student().map_err(|_| {
        error_with_log_id(
            "entered a protected route without a student loaded in request",
            "Authentication error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let group_id = path.into_inner();
    let members = groups_repository::get_group_members(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("failed to fetch group members for {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let is_member = members
        .into_iter()
        .any(|member_state| member_state.student_id == student.student_id);
    if !is_member {
        return Err(
            "You can only see complaints filed by groups you are member of"
                .to_json_error(StatusCode::FORBIDDEN),
        );
    }

    let complaints = complaints_repository::get_filed_by_group(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "failed to fetch filed complaints for group {}: {}",
                    group_id, e
                ),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let items = complaints
        .into_iter()
        .map(|complaint_state| {
            let complaint = DbState::into_inner(complaint_state);
            ComplaintItem {
                complaint_id: complaint.complaint_id,
                transaction_id: complaint.transaction_id,
                to_group_id: complaint.to_group_id,
                text: complaint.text,
                created_at: complaint.created_at,
            }
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(items))
}
