use crate::api::v1::admins::uploads::download::download_student_upload_handler;
use crate::api::v1::admins::uploads::list::list_project_uploads_handler;
use actix_web::{web, Scope};

pub(crate) mod download;
pub(crate) mod list;

pub(super) fn uploads_scope() -> Scope {
    web::scope("/projects")
        .route(
            "/{project_id}/uploads",
            web::get().to(list_project_uploads_handler),
        )
        .route(
            "/{project_id}/students/{student_id}/upload",
            web::get().to(download_student_upload_handler),
        )
}
