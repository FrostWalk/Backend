use crate::api::v1::admins::student_deliverables::create::create_student_deliverable_handler;
use crate::api::v1::admins::student_deliverables::delete::delete_student_deliverable_handler;
use crate::api::v1::admins::student_deliverables::read::{
    get_all_student_deliverables_handler, get_components_for_student_deliverable_handler,
    get_student_deliverable_handler, get_student_deliverables_for_project_handler,
};
use crate::api::v1::admins::student_deliverables::update::update_student_deliverable_handler;
use crate::jwt::admin_auth_factory::Admin;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn student_deliverables_scope() -> Scope {
    web::scope("/student-deliverables")
        .route(
            "",
            web::get()
                .to(get_all_student_deliverables_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "",
            web::post()
                .to(create_student_deliverable_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/project/{project_id}",
            web::get()
                .to(get_student_deliverables_for_project_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::get()
                .to(get_student_deliverable_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}/components",
            web::get()
                .to(get_components_for_student_deliverable_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                    AvailableAdminRole::Coordinator,
                ])),
        )
        .route(
            "/{id}",
            web::patch()
                .to(update_student_deliverable_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::delete()
                .to(delete_student_deliverable_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
}
