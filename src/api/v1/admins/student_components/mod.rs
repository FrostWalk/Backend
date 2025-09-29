use crate::api::v1::admins::student_components::create::create_student_component_handler;
use crate::api::v1::admins::student_components::delete::delete_student_component_handler;
use crate::api::v1::admins::student_components::read::{
    get_all_student_components_handler, get_student_component_handler,
    get_student_components_for_project_handler,
};
use crate::api::v1::admins::student_components::update::update_student_component_handler;
use crate::jwt::admin_auth_factory::Admin;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn student_components_scope() -> Scope {
    web::scope("/student-components")
        .route(
            "",
            web::get()
                .to(get_all_student_components_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "",
            web::post()
                .to(create_student_component_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/project/{project_id}",
            web::get()
                .to(get_student_components_for_project_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::get()
                .to(get_student_component_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::patch()
                .to(update_student_component_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::delete()
                .to(delete_student_component_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
}
