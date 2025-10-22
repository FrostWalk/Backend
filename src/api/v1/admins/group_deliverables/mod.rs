use crate::api::v1::admins::group_deliverables::create::create_group_deliverable_handler;
use crate::api::v1::admins::group_deliverables::delete::delete_group_deliverable_handler;
use crate::api::v1::admins::group_deliverables::read::{
    get_all_group_deliverables_handler, get_components_for_group_deliverable_handler,
    get_group_deliverable_handler, get_group_deliverables_for_project_handler,
};
use crate::api::v1::admins::group_deliverables::update::update_group_deliverable_handler;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn group_deliverables_scope() -> Scope {
    web::scope("/group-deliverables")
        .route("", web::get().to(get_all_group_deliverables_handler))
        .route("", web::post().to(create_group_deliverable_handler))
        .route(
            "/project/{project_id}",
            web::get().to(get_group_deliverables_for_project_handler),
        )
        .route("/{id}", web::get().to(get_group_deliverable_handler))
        .route(
            "/{id}/components",
            web::get().to(get_components_for_group_deliverable_handler),
        )
        .route("/{id}", web::patch().to(update_group_deliverable_handler))
        .route("/{id}", web::delete().to(delete_group_deliverable_handler))
}
