use crate::api::v1::admins::security_codes::create::create_code_handler;
use crate::api::v1::admins::security_codes::delete::delete_code_handler;
use crate::api::v1::admins::security_codes::read::get_all_codes_handler;
use crate::api::v1::admins::security_codes::update::update_code_handler;
use crate::jwt::admin_auth_factory::Admin;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn security_codes_scope() -> Scope {
    web::scope("/security-codes")
        .route(
            "",
            web::post()
                .to(create_code_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                    AvailableAdminRole::Coordinator,
                ])),
        )
        .route(
            "",
            web::get()
                .to(get_all_codes_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                    AvailableAdminRole::Coordinator,
                ])),
        )
        .route(
            "/{security_code_id}",
            web::patch()
                .to(update_code_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                    AvailableAdminRole::Coordinator,
                ])),
        )
        .route(
            "/{security_code_id}",
            web::delete()
                .to(delete_code_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                    AvailableAdminRole::Coordinator,
                ])),
        )
}
