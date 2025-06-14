use crate::api::v1::admins::users::create::create_admin_handler;
use crate::api::v1::admins::users::delete::delete_admin_handler;
use crate::api::v1::admins::users::me::admins_me_handler;
use crate::api::v1::admins::users::read::{admins_get_all_handler, admins_get_one_handler};
use crate::api::v1::admins::users::update::update_admin_handler;
use crate::database::repositories::admins_repository::{AdminRole, ALL};
use crate::jwt::admin_auth_factory::Admin;
use actix_web::{web, Scope};
use entity::admins;
use serde::Serialize;
use utoipa::ToSchema;

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod me;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn users_scope() -> Scope {
    web::scope("/users")
        .route(
            "",
            web::get()
                .to(admins_get_all_handler)
                .wrap(Admin::require_roles([
                    AdminRole::Root,
                    AdminRole::Professor,
                    AdminRole::Tutor,
                ])),
        )
        .route(
            "",
            web::post()
                .to(create_admin_handler)
                .wrap(Admin::require_roles([
                    AdminRole::Root,
                    AdminRole::Professor,
                ])),
        )
        .route(
            "",
            web::patch()
                .to(update_admin_handler)
                .wrap(Admin::require_roles([
                    AdminRole::Root,
                    AdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::get()
                .to(admins_get_one_handler)
                .wrap(Admin::require_roles([
                    AdminRole::Root,
                    AdminRole::Professor,
                    AdminRole::Tutor,
                ])),
        )
        .route(
            "/{id}",
            web::delete()
                .to(delete_admin_handler)
                .wrap(Admin::require_roles([
                    AdminRole::Root,
                    AdminRole::Professor,
                ])),
        )
        .route(
            "/me",
            web::get()
                .to(admins_me_handler)
                .wrap(Admin::require_roles(ALL)),
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
    #[schema(example = 1)]
    pub role_id: i32,
}

impl From<admins::Model> for AdminResponseScheme {
    fn from(value: admins::Model) -> Self {
        Self {
            id: value.admin_id,
            first_name: value.first_name,
            last_name: value.last_name,
            email: value.email,
            role_id: value.admin_role_id,
        }
    }
}
