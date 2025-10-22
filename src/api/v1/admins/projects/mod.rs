use crate::api::v1::admins::projects::coordinators::{
    assign_coordinator, list_coordinators, remove_coordinator,
};
use crate::api::v1::admins::projects::create::create_project_handler;
use crate::api::v1::admins::projects::delete::delete_project_handler;
use crate::api::v1::admins::projects::read::{get_all_projects_handler, get_one_project_handler};
use crate::api::v1::admins::projects::update::update_project_handler;
use actix_web::{web, Scope};

pub(crate) mod coordinators;
pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn projects_scope() -> Scope {
    web::scope("/projects")
        .route("", web::post().to(create_project_handler))
        .route("", web::get().to(get_all_projects_handler))
        .route("/{id}", web::get().to(get_one_project_handler))
        .route("/{id}", web::patch().to(update_project_handler))
        .route("/{id}", web::delete().to(delete_project_handler))
        .route(
            "/{project_id}/coordinators",
            web::post().to(assign_coordinator),
        )
        .route(
            "/{project_id}/coordinators",
            web::get().to(list_coordinators),
        )
        .route(
            "/{project_id}/coordinators/{admin_id}",
            web::delete().to(remove_coordinator),
        )
}
