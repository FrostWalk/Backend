use crate::api::v1::admins::student_deliverable_selections::read::get_student_deliverable_selections;
use crate::jwt::admin_auth_factory::Admin;
use crate::models::admin_role::ALL;
use actix_web::{web, Scope};

pub(crate) mod read;

pub(super) fn student_deliverable_selections_scope() -> Scope {
    web::scope("/student-deliverable-selections").route(
        "/projects/{project_id}",
        web::get()
            .to(get_student_deliverable_selections)
            .wrap(Admin::require_roles(ALL)),
    )
}
