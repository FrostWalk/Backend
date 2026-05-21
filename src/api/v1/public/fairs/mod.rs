use crate::api::v1::public::fairs::leaderboard::leaderboard_handler;
use actix_web::{web, Scope};

pub(crate) mod leaderboard;

pub(super) fn public_fairs_scope() -> Scope {
    web::scope("/fairs").route("/{fair_id}/leaderboard", web::get().to(leaderboard_handler))
}
