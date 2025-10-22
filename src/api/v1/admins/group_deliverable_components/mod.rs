use crate::api::v1::admins::group_deliverable_components::create::create_group_component_handler;
use crate::api::v1::admins::group_deliverable_components::delete::delete_group_component_handler;
use crate::api::v1::admins::group_deliverable_components::read::{
    get_all_group_components_handler, get_deliverables_for_group_component_handler,
    get_group_component_handler, get_group_components_for_project_handler,
};
use crate::api::v1::admins::group_deliverable_components::update::update_group_component_handler;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn group_deliverable_components_scope() -> Scope {
    web::scope("/group-deliverable-components")
        .route("", web::get().to(get_all_group_components_handler))
        .route("", web::post().to(create_group_component_handler))
        .route(
            "/project/{project_id}",
            web::get().to(get_group_components_for_project_handler),
        )
        .route("/{id}", web::get().to(get_group_component_handler))
        .route(
            "/{id}/deliverables",
            web::get().to(get_deliverables_for_group_component_handler),
        )
        .route("/{id}", web::patch().to(update_group_component_handler))
        .route("/{id}", web::delete().to(delete_group_component_handler))
}
