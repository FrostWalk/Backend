use crate::app_state::AppState;
use crate::config::MarketConfig;
use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};

mod api;
mod app_state;
mod config;
mod database;
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");

    MarketConfig::load();

    let db_conn = sea_orm::Database::connect(MarketConfig::db_url()).await.unwrap();

    HttpServer::new(move || App::new().app_data(Data::new(AppState::new(db_conn.clone()))).wrap(middleware::Logger::default()))
        .workers(MarketConfig::workers())
        .bind((MarketConfig::address(), MarketConfig::port()))?
        .run()
        .await
}
