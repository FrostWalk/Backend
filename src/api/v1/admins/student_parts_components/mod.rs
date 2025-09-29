use crate::api::v1::admins::student_parts_components::create::create_student_part_component_handler;
use crate::api::v1::admins::student_parts_components::delete::delete_student_part_component_handler;
use crate::api::v1::admins::student_parts_components::read::{
    get_components_for_part_handler, get_parts_for_component_handler,
};
use crate::api::v1::admins::student_parts_components::update::update_student_part_component_handler;
use crate::jwt::admin_auth_factory::Admin;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn student_parts_components_scope() -> Scope {
    web::scope("/student-parts-components")
        .route(
            "",
            web::post()
                .to(create_student_part_component_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/components/{part_id}",
            web::get()
                .to(get_components_for_part_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/parts/{component_id}",
            web::get()
                .to(get_parts_for_component_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::patch()
                .to(update_student_part_component_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
        .route(
            "/{id}",
            web::delete()
                .to(delete_student_part_component_handler)
                .wrap(Admin::require_roles([
                    AvailableAdminRole::Root,
                    AvailableAdminRole::Professor,
                ])),
        )
}
