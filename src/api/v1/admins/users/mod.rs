use crate::api::v1::admins::users::create::create_admin_handler;
use crate::api::v1::admins::users::me::admins_me_handler;
use crate::database::repositories::admins_repository::{AdminRole, ALL};
use crate::jwt::admin_auth_factory::Admin;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod me;

pub(super) fn users_scope() -> Scope {
    web::scope("/users")
        .route(
            "/me",
            web::get()
                .to(admins_me_handler)
                .wrap(Admin::require_roles(ALL)),
        )
        .route(
            "/create",
            web::post()
                .to(create_admin_handler)
                .wrap(Admin::require_roles([
                    AdminRole::Root,
                    AdminRole::Professor,
                ])),
        )
}
