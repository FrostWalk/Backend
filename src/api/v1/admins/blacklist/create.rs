use crate::app_data::AppData;
use crate::common::json_error::{
    error_with_log_id, error_with_log_id_and_payload, JsonError, ToJsonError,
};
use crate::database::repositories::{blacklist_repository, students_repository};
use crate::models::blacklist::Blacklist;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct AddToBlacklistScheme {
    #[schema(example = 42)]
    pub student_id: i32,
    #[schema(example = "Copied homework repeatedly")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct AddToBlacklistResponse {
    pub blacklist: BlacklistDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct BlacklistDto {
    pub blacklist_id: i32,
    pub university_id: i32,
    pub description: String,
    pub first_name: String,
    pub last_name: String,
    #[schema(value_type = String, example = "2026-05-21T12:34:56Z")]
    pub banned_at: chrono::DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/v1/admins/blacklist",
    request_body = AddToBlacklistScheme,
    responses(
        (status = 201, description = "Student added to blacklist", body = AddToBlacklistResponse),
        (status = 400, description = "Invalid request body", body = JsonError),
        (status = 404, description = "Student not found", body = JsonError),
        (status = 409, description = "Student already blacklisted", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin blacklist management",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(in crate::api::v1) async fn add_to_blacklist_handler(
    body: Json<AddToBlacklistScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    if body.student_id <= 0 {
        return Err("student_id must be greater than 0".to_json_error(StatusCode::BAD_REQUEST));
    }

    let student = students_repository::get_by_id(&data.db, body.student_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to retrieve student from database: {}", e),
                "Failed to add student to blacklist",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?
        .ok_or_else(|| "Student not found".to_json_error(StatusCode::NOT_FOUND))?;

    if blacklist_repository::get_by_university_id(&data.db, student.university_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to check blacklist by university_id: {}", e),
                "Failed to add student to blacklist",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .is_some()
    {
        return Err("Student already blacklisted".to_json_error(StatusCode::CONFLICT));
    }

    let entry = Blacklist {
        blacklist_id: 0,
        university_id: student.university_id,
        description: body.description.clone().unwrap_or_default(),
        first_name: student.first_name.clone(),
        last_name: student.last_name.clone(),
        banned_at: Utc::now(),
    };

    let created = blacklist_repository::create(&data.db, entry)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to create blacklist entry: {}", e),
                "Failed to add student to blacklist",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    Ok(HttpResponse::Created().json(AddToBlacklistResponse {
        blacklist: to_dto(created),
    }))
}

fn to_dto(state: DbState<Blacklist>) -> BlacklistDto {
    let item = DbState::into_inner(state);
    BlacklistDto {
        blacklist_id: item.blacklist_id,
        university_id: item.university_id,
        description: item.description,
        first_name: item.first_name,
        last_name: item.last_name,
        banned_at: item.banned_at,
    }
}
