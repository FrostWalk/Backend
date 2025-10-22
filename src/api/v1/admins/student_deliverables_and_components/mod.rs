use crate::api::v1::admins::student_deliverables_and_components::create::create_student_deliverable_component_handler;
use crate::api::v1::admins::student_deliverables_and_components::delete::delete_student_deliverable_component_handler;
use crate::api::v1::admins::student_deliverables_and_components::read::{
    get_components_for_deliverable_handler, get_deliverables_for_component_handler,
};
use crate::api::v1::admins::student_deliverables_and_components::update::update_student_deliverable_component_handler;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn student_deliverables_components_scope() -> Scope {
    web::scope("/student-deliverables-components")
        .route(
            "",
            web::post().to(create_student_deliverable_component_handler),
        )
        .route(
            "/components/{deliverable_id}",
            web::get().to(get_components_for_deliverable_handler),
        )
        .route(
            "/deliverables/{component_id}",
            web::get().to(get_deliverables_for_component_handler),
        )
        .route(
            "/{id}",
            web::patch().to(update_student_deliverable_component_handler),
        )
        .route(
            "/{id}",
            web::delete().to(delete_student_deliverable_component_handler),
        )
}
