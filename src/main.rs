use actix_web::{App, HttpServer};

use crate::config::MarketConfig;

mod config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    MarketConfig::load();

    HttpServer::new(|| {
        App::new()
    }).workers(MarketConfig::workers())
        .bind((MarketConfig::address(), MarketConfig::port()))?
        .run().await
}
