use crate::app_data::AppData;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::database::repository_methods_trait::RepositoryMethods;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{web, HttpResponse};
use entity::projects;
use log::error;
use sea_orm::{ActiveValue, NotSet};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
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
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Projects management",
)]
/// Update the project details
pub(super) async fn update_project_handler(
    path: web::Path<i32>, payload: Json<UpdateProjectScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let scheme = payload.into_inner();
    let id = path.into_inner();

    let project_update = projects::ActiveModel {
        project_id: ActiveValue::Unchanged(id),
        name: scheme.name.map_or(NotSet, ActiveValue::Set),
        max_group_size: scheme.max_group_size.map_or(NotSet, ActiveValue::Set),
        max_student_uploads: scheme.max_student_uploads.map_or(NotSet, ActiveValue::Set),
        active: scheme.active.map_or(NotSet, ActiveValue::Set),
        year: NotSet,
    };

    data.repositories
        .projects
        .update(project_update)
        .await
        .map_err(|e| {
            error!("unable to update project: {}", e);
            "unable to update project scheme".to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    Ok(HttpResponse::Ok().finish())
}
