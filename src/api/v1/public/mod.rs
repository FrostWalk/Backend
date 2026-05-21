use crate::api::v1::public::fairs::public_fairs_scope;
use actix_web::{web, Scope};

pub(crate) mod fairs;

pub(super) fn public_scope() -> Scope {
    web::scope("").service(public_fairs_scope())
}
