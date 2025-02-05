extern crate core;

use crate::api::configure_endpoints;
use crate::app_state::AppState;
use crate::config::Config;
use crate::database::migrate_database;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use env_logger::Env;

mod api;
mod app_state;
mod common;
mod config;
mod database;
mod jwt;
mod test;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // print debug logs in console
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    // load config from env or file
    let app_config = Config::load();
    let app_state = AppState::new(app_config.clone()).await;

    // migrate database to latest changes
    migrate_database(app_config.db_url()).await;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_state.clone())) //add application state with repositories and config
            .wrap(Logger::default()) // add logging middleware
            .configure(configure_endpoints) // add scopes and routes
    })
    .workers(app_config.workers()) // normally 1 worker per thread
    .bind((app_config.address().clone(), app_config.port()))? // address and port on which the server is listening
    .run()
    .await
}
