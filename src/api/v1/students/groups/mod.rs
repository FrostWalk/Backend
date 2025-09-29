use crate::api::v1::students::groups::check_name::check_name;
use crate::api::v1::students::groups::create::create_group;
use crate::api::v1::students::groups::delete::delete_group;
use crate::api::v1::students::groups::members::{add_member, remove_member};
use crate::api::v1::students::groups::read::get_groups;
use crate::api::v1::students::groups::update::update_group;
use crate::api::v1::students::groups::validate_code::validate_code;
use crate::jwt::student_auth_factory::Student;
use actix_web::{web, Scope};

pub(crate) mod check_name;
pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod members;
pub(crate) mod read;
pub(crate) mod update;
pub(crate) mod validate_code;

pub(super) fn groups_scope() -> Scope {
    web::scope("/groups")
        .route(
            "",
            web::post().to(create_group).wrap(Student::require_auth()),
        )
        .route("", web::get().to(get_groups).wrap(Student::require_auth()))
        .route(
            "/validate-code",
            web::post().to(validate_code).wrap(Student::require_auth()),
        )
        .route(
            "/check-name",
            web::get().to(check_name).wrap(Student::require_auth()),
        )
        .route(
            "/{group_id}",
            web::put().to(update_group).wrap(Student::require_auth()),
        )
        .route(
            "/{group_id}",
            web::delete().to(delete_group).wrap(Student::require_auth()),
        )
        .route(
            "/{group_id}/members",
            web::post().to(add_member).wrap(Student::require_auth()),
        )
        .route(
            "/{group_id}/members",
            web::delete()
                .to(remove_member)
                .wrap(Student::require_auth()),
        )
}
