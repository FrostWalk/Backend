use crate::api::v1::users::me::me_handler;
use crate::jwt::admin_auth_factory::RequireAdmin;
use actix_web::{web, Scope};

pub(crate) mod me;

pub(super) fn users_scope() -> Scope {
    web::scope("/users").route(
        "/me",
        web::get()
            .to(me_handler)
            .wrap(RequireAdmin::allowed_roles(ALL)),
    )
}
