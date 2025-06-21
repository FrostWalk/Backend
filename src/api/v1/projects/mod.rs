use crate::api::v1::projects::create::create_project_handler;
use crate::api::v1::projects::delete::delete_project_handler;
use crate::database::repositories::admins_repository::AdminRole;
use crate::jwt::admin_auth_factory::Admin;
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
                    AdminRole::Root,
                    AdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::delete()
                .to(delete_project_handler)
                .wrap(Admin::require_roles([
                    AdminRole::Root,
                    AdminRole::Professor,
                ])),
        )
}
