use crate::api::v1::students::auth::auth_scope;
use crate::api::v1::students::groups::groups_scope;
use crate::api::v1::students::projects::projects_scope;
use crate::api::v1::students::security_codes::security_codes_scope;
use crate::api::v1::students::users::users_scope;
use actix_web::{web, Scope};

pub(crate) mod auth;
pub(crate) mod groups;
pub(crate) mod projects;
pub(crate) mod security_codes;
pub(crate) mod users;

pub(super) fn students_scope() -> Scope {
    web::scope("/students")
        .service(users_scope())
        .service(auth_scope())
        .service(projects_scope())
        .service(security_codes_scope())
        .service(groups_scope())
}
