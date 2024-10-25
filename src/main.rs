use crate::app_state::AppState;
use crate::config::MarketConfig;
use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use std::env::set_var;
use sea_orm::Database;

mod api;
mod app_state;
mod config;
mod database;
mod test;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    set_var("RUST_LOG", "debug");

    MarketConfig::load();

    let db_conn = Database::connect(MarketConfig::db_url()).await.unwrap();

    HttpServer::new(move || App::new().app_data(Data::new(AppState::new(db_conn.clone()))).wrap(middleware::Logger::default()))
        .workers(MarketConfig::workers())
        .bind((MarketConfig::address(), MarketConfig::port()))?
        .run()
        .await
}
