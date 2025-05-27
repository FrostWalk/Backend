use crate::api::v1::users::me::me_handler;
use crate::jwt::auth_factory::RequireAuth;
use actix_web::{web, Scope};

pub(crate) mod me;

pub(super) fn users_scope() -> Scope {
    web::scope("/users").route(
        "/me",
        web::get()
            .to(me_handler)
            .wrap(RequireAuth::allowed_roles(ALL)),
    )
}
