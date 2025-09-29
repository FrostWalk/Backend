use crate::api::v1::students::security_codes::validate_code::validate_code;
use crate::jwt::student_auth_factory::Student;
use actix_web::{web, Scope};

pub(crate) mod validate_code;

pub(super) fn security_codes_scope() -> Scope {
    web::scope("/security-codes").route(
        "/validate",
        web::post().to(validate_code).wrap(Student::require_auth()),
    )
}
