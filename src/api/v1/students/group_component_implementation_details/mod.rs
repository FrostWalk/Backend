use crate::api::v1::students::group_component_implementation_details::create::create_component_implementation_detail;
use crate::api::v1::students::group_component_implementation_details::delete::delete_component_implementation_detail;
use crate::api::v1::students::group_component_implementation_details::read::get_component_implementation_details;
use crate::api::v1::students::group_component_implementation_details::update::update_component_implementation_detail;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn group_component_implementation_details_scope() -> Scope {
    web::scope("/group-component-implementation-details")
        .route(
            "/{group_id}",
            web::post().to(create_component_implementation_detail),
        )
        .route(
            "/{group_id}",
            web::get().to(get_component_implementation_details),
        )
        .route(
            "/{group_id}",
            web::patch().to(update_component_implementation_detail),
        )
        .route(
            "/{group_id}",
            web::delete().to(delete_component_implementation_detail),
        )
}
