use crate::api::v1::admins::projects::create::create_project_handler;
use crate::api::v1::admins::projects::delete::delete_project_handler;
use crate::api::v1::admins::projects::read::{get_all_projects_handler, get_one_project_handler};
use crate::api::v1::admins::projects::update::update_project_handler;
use crate::jwt::admin_auth_factory::Admin;
use crate::models::admin_role::{AvailableAdminRole, ALL};
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn projects_scope() -> Scope {
    web::scope("/projects")
        .route(
            "",
            web::post()
                .to(create_project_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "",
            web::get()
                .to(get_all_projects_handler)
                .wrap(Admin::require_roles(ALL)),
        )
        .route(
            "/{id}",
            web::get()
                .to(get_one_project_handler)
                .wrap(Admin::require_roles(ALL)),
        )
        .route(
            "/{id}",
            web::patch()
                .to(update_project_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::delete()
                .to(delete_project_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
}
