use crate::api::v1::admins::student_deliverable_components::create::create_student_component_handler;
use crate::api::v1::admins::student_deliverable_components::delete::delete_student_component_handler;
use crate::api::v1::admins::student_deliverable_components::read::{
    get_all_student_components_handler, get_deliverables_for_student_component_handler,
    get_student_component_handler, get_student_components_for_project_handler,
};
use crate::api::v1::admins::student_deliverable_components::update::update_student_component_handler;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn student_deliverable_components_scope() -> Scope {
    web::scope("/student-deliverable-components")
        .route("", web::get().to(get_all_student_components_handler))
        .route("", web::post().to(create_student_component_handler))
        .route(
            "/project/{project_id}",
            web::get().to(get_student_components_for_project_handler),
        )
        .route("/{id}", web::get().to(get_student_component_handler))
        .route(
            "/{id}/deliverables",
            web::get().to(get_deliverables_for_student_component_handler),
        )
        .route("/{id}", web::patch().to(update_student_component_handler))
        .route("/{id}", web::delete().to(delete_student_component_handler))
}
