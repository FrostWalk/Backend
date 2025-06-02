use crate::api::v1::admins::admins_scope;
use crate::api::v1::students::students_scope;
use actix_web::{web, Scope};

pub(crate) mod admins;
pub(crate) mod students;

pub(super) fn v1_scope() -> Scope {
    web::scope("/v1")
        .service(admins_scope())
        .service(students_scope())
}
