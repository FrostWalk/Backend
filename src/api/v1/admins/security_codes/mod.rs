use crate::api::v1::admins::security_codes::create::create_code_handler;
use crate::database::repositories::admins_repository::AdminRole;
use crate::jwt::admin_auth_factory::Admin;
use actix_web::{web, Scope};

pub(crate) mod create;

pub(super) fn security_codes_scope() -> Scope {
    web::scope("/security-codes").route(
        "",
        web::post()
            .to(create_code_handler)
            .wrap(Admin::require_roles([
                AdminRole::Root,
                AdminRole::Professor,
            ])),
    )
}
