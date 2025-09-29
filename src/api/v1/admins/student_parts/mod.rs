use crate::api::v1::admins::student_parts::create::create_student_part_handler;
use crate::api::v1::admins::student_parts::delete::delete_student_part_handler;
use crate::api::v1::admins::student_parts::read::{
    get_all_student_parts_handler, get_components_for_student_part_handler, get_student_part_handler,
    get_student_parts_for_project_handler,
};
use crate::api::v1::admins::student_parts::update::update_student_part_handler;
use crate::jwt::admin_auth_factory::Admin;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn student_parts_scope() -> Scope {
    web::scope("/student-parts")
        .route(
            "",
            web::get()
                .to(get_all_student_parts_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "",
            web::post()
                .to(create_student_part_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/project/{project_id}",
            web::get()
                .to(get_student_parts_for_project_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::get()
                .to(get_student_part_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}/components",
            web::get()
                .to(get_components_for_student_part_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::patch()
                .to(update_student_part_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::delete()
                .to(delete_student_part_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
}
