use crate::api::v1::students::uploads::status::get_upload_status_handler;
use crate::api::v1::students::uploads::upload::upload_project_zip_handler;
use actix_web::{web, Scope};

pub(crate) mod status;
pub(crate) mod upload;

pub(super) fn uploads_scope() -> Scope {
    web::scope("")
        .route(
            "/projects/{project_id}/upload",
            web::post().to(upload_project_zip_handler),
        )
        .route(
            "/projects/{project_id}/upload",
            web::get().to(get_upload_status_handler),
        )
}
