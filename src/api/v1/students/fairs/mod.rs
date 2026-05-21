use crate::api::v1::students::fairs::list::list_transactions_handler;
use crate::api::v1::students::fairs::purchase::purchase_handler;
use actix_web::{web, Scope};

pub(crate) mod list;
pub(crate) mod purchase;

pub(super) fn student_fairs_scope() -> Scope {
    web::scope("/fairs")
        .route("/{fair_id}/transactions", web::post().to(purchase_handler))
        .route(
            "/{fair_id}/transactions",
            web::get().to(list_transactions_handler),
        )
}
