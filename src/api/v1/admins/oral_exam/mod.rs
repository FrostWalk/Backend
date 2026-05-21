use crate::api::v1::admins::oral_exam::completions::{
    bulk_set_group_completions, set_student_completion,
};
use crate::api::v1::admins::oral_exam::group_details::get_oral_exam_group_details;
use crate::api::v1::admins::oral_exam::list_groups::list_oral_exam_groups;
use crate::api::v1::admins::oral_exam::notes::{delete_note, upsert_note};
use crate::api::v1::admins::oral_exam::toggle::toggle_oral_exam;
use actix_web::{web, Scope};

pub(crate) mod completions;
pub(crate) mod group_details;
pub(crate) mod list_groups;
pub(crate) mod notes;
pub(crate) mod toggle;

pub(super) fn oral_exam_scope() -> Scope {
    web::scope("/oral-exam")
        // Enable / disable oral exam mode on a project
        .route("/projects/{project_id}", web::patch().to(toggle_oral_exam))
        // List groups alphabetically (with optional ?search=)
        .route(
            "/projects/{project_id}/groups",
            web::get().to(list_oral_exam_groups),
        )
        // Full group details for oral exam
        .route(
            "/projects/{project_id}/groups/{group_id}",
            web::get().to(get_oral_exam_group_details),
        )
        // Upsert / delete a note for a student in a project
        .route(
            "/projects/{project_id}/students/{student_id}/note",
            web::put().to(upsert_note),
        )
        .route(
            "/projects/{project_id}/students/{student_id}/note",
            web::delete().to(delete_note),
        )
        // Mark individual student completed / incomplete
        .route(
            "/projects/{project_id}/students/{student_id}/completion",
            web::put().to(set_student_completion),
        )
        // Bulk-mark students in a group (for "mark all present" shortcut)
        .route(
            "/projects/{project_id}/groups/{group_id}/completions",
            web::post().to(bulk_set_group_completions),
        )
}
