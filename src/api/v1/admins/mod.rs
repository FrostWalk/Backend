use crate::api::v1::admins::auth::auth_scope;
use crate::api::v1::admins::projects::projects_scope;
use crate::api::v1::admins::security_codes::security_codes_scope;
use crate::api::v1::admins::users::users_scope;
use actix_web::{web, Scope};

pub(crate) mod auth;
pub(crate) mod projects;
pub(crate) mod security_codes;
pub(crate) mod users;

pub(super) fn admins_scope() -> Scope {
    web::scope("/admins")
        .service(auth_scope())
        .service(users_scope())
        .service(projects_scope())
        .service(security_codes_scope())
}
