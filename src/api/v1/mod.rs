use crate::api::v1::auth::auth_scope;
use crate::api::v1::users::users_scope;
use actix_web::{web, Scope};

mod auth;
pub(crate) mod doc;
mod users;

pub(super) fn v1_scope() -> Scope {
    web::scope("/v1")
        .service(auth_scope())
        .service(users_scope())
}
