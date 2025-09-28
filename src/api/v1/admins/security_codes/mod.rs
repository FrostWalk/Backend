use crate::api::v1::admins::security_codes::create::create_code_handler;
use crate::api::v1::admins::security_codes::read::get_all_codes_handler;
use crate::jwt::admin_auth_factory::Admin;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod read;

pub(super) fn security_codes_scope() -> Scope {
    web::scope("/security-codes")
        .route(
            "",
            web::post()
                .to(create_code_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "",
            web::get()
                .to(get_all_codes_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
}
