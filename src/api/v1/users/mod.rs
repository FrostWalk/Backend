use crate::api::v1::users::get_me::get_me;
use crate::jwt::auth_factory::RequireAuth;
use crate::jwt::role::ALL;
use actix_web::{web, Scope};

mod get_me;

pub(super) fn users_scope() -> Scope {
    web::scope("/users").route(
        "/me",
        web::get().to(get_me).wrap(RequireAuth::allowed_roles(ALL)),
    )
}
