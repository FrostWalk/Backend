use crate::api::v1::admins::users::create::create_admin_handler;
use crate::api::v1::admins::users::delete::delete_admin_handler;
use crate::api::v1::admins::users::me::admins_me_handler;
use crate::api::v1::admins::users::read::{get_all_admins_handler, get_one_admin_handler};
use crate::api::v1::admins::users::update::update_admin_handler;
use crate::api::v1::admins::users::update_me::update_me_admin_handler;
use crate::jwt::admin_auth_factory::Admin;
use crate::models::admin;
use crate::models::admin_role::{AvailableAdminRole, ALL};
use actix_web::{web, Scope};
use serde::Serialize;
use utoipa::ToSchema;

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod me;
pub(crate) mod read;
pub(crate) mod update;
pub(crate) mod update_me;

pub(super) fn users_scope() -> Scope {
    web::scope("/users")
        .route(
            "/me",
            web::get()
                .to(admins_me_handler)
                .wrap(Admin::require_roles(ALL)),
        )
        .route(
            "/me",
            web::patch()
                .to(update_me_admin_handler)
                .wrap(Admin::require_roles(ALL)),
        )
        .route(
            "",
            web::get()
                .to(get_all_admins_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "",
            web::post()
                .to(create_admin_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::patch()
                .to(update_admin_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::get()
                .to(get_one_admin_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::delete()
                .to(delete_admin_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct AdminResponseScheme {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "Jane")]
    pub first_name: String,
    #[schema(example = "Doe")]
    pub last_name: String,
    #[schema(format = "email", example = "jane.doe@admin.com")]
    pub email: String,
    #[schema(example = 2)]
    pub role_id: i32,
}

impl From<admin::Admin> for AdminResponseScheme {
    fn from(value: admin::Admin) -> Self {
        Self {
            id: value.admin_id,
            first_name: value.first_name,
            last_name: value.last_name,
            email: value.email,
            role_id: value.admin_role_id,
        }
    }
}
