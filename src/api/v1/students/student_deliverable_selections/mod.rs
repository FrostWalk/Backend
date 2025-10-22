use crate::api::v1::students::student_deliverable_selections::create::create_student_deliverable_selection;
use crate::api::v1::students::student_deliverable_selections::delete::delete_student_deliverable_selection;
use crate::api::v1::students::student_deliverable_selections::read::get_student_deliverable_selection;
use crate::api::v1::students::student_deliverable_selections::update::update_student_deliverable_selection;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn student_deliverable_selections_scope() -> Scope {
    web::scope("/deliverable-selection")
        .route("", web::post().to(create_student_deliverable_selection))
        .route("", web::patch().to(update_student_deliverable_selection))
        .route(
            "/project/{project_id}",
            web::get().to(get_student_deliverable_selection),
        )
        .route(
            "/project/{project_id}",
            web::delete().to(delete_student_deliverable_selection),
        )
}
