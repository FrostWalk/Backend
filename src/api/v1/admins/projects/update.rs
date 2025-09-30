use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::projects_repository;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateProjectScheme {
    pub name: Option<String>,
    pub max_student_uploads: Option<i32>,
    pub max_group_size: Option<i32>,
    pub active: Option<bool>,
}
#[utoipa::path(
    patch,
    path = "/v1/admins/projects/{id}",
    request_body = UpdateProjectScheme,
    responses(
        (status = 200, description = "Project updated successfully"),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Projects management",
)]
/// Update a project details
pub(in crate::api::v1) async fn update_project_handler(
    path: web::Path<i32>, payload: Json<UpdateProjectScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();
    let scheme = payload.into_inner();
    let original_payload = Json(UpdateProjectScheme {
        name: scheme.name.clone(),
        max_student_uploads: scheme.max_student_uploads,
        max_group_size: scheme.max_group_size,
        active: scheme.active,
    });

    let state_opt = projects_repository::get_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to load project {}: {}", id, e),
                "Failed to update project",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &original_payload,
            )
        })?;

    let mut state = match state_opt {
        Some(s) => s,
        None => return Err("Project not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // 2) Apply only provided fields
    if let Some(v) = scheme.name {
        state.name = v;
    }
    if let Some(v) = scheme.max_student_uploads {
        state.max_student_uploads = v;
    }
    if let Some(v) = scheme.max_group_size {
        state.max_group_size = v;
    }
    if let Some(v) = scheme.active {
        state.active = v;
    }

    state.save(&data.db).await.map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to update project {}: {}", id, e),
            "Failed to update project",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &original_payload,
        )
    })?;

    Ok(HttpResponse::Ok().json((*state).clone()))
}
