use crate::api::v1::admins::auth::auth_scope;
use crate::api::v1::admins::group_deliverable_components::group_deliverable_components_scope;
use crate::api::v1::admins::group_deliverables::group_deliverables_scope;
use crate::api::v1::admins::group_deliverables_and_components::group_deliverables_components_scope;
use crate::api::v1::admins::groups::groups_scope;
use crate::api::v1::admins::projects::projects_scope;
use crate::api::v1::admins::security_codes::security_codes_scope;
use crate::api::v1::admins::student_deliverable_components::student_deliverable_components_scope;
use crate::api::v1::admins::student_deliverables::student_deliverables_scope;
use crate::api::v1::admins::student_deliverables_and_components::student_deliverables_components_scope;
use crate::api::v1::admins::users::users_scope;
use actix_web::{web, Scope};

pub(crate) mod auth;
pub(crate) mod group_deliverable_components;
pub(crate) mod group_deliverables;
pub(crate) mod group_deliverables_and_components;
pub(crate) mod groups;
pub(crate) mod projects;
pub(crate) mod security_codes;
pub(crate) mod student_deliverable_components;
pub(crate) mod student_deliverables;
pub(crate) mod student_deliverables_and_components;
pub(crate) mod users;

pub(super) fn admins_scope() -> Scope {
    web::scope("/admins")
        .service(auth_scope())
        .service(users_scope())
        .service(projects_scope())
        .service(security_codes_scope())
        .service(groups_scope())
        .service(group_deliverable_components_scope())
        .service(group_deliverables_scope())
        .service(group_deliverables_components_scope())
        .service(student_deliverable_components_scope())
        .service(student_deliverables_scope())
        .service(student_deliverables_components_scope())
}
