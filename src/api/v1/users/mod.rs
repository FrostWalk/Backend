use crate::api::v1::users::me::me_handler;
use crate::jwt::student_auth_factory::RequireStudent;
use actix_web::{web, Scope};

pub(crate) mod me;

pub(super) fn users_scope() -> Scope {
    web::scope("/users").route(
        "/me",
        web::get()
            .to(me_handler)
            .wrap(RequireStudent::require_auth()),
    )
}
