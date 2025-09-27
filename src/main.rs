use crate::api::configure_endpoints;
use crate::app_data::AppData;
use crate::config::Config;
use crate::database::repositories::admins_repository::create_default_admin;
use crate::logging::logger::init_mongo_logger;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use log::{error, info};
use welds::connections::postgres::connect;

mod api;
mod app_data;
mod common;
mod config;
mod database;
mod jwt;
mod logging;
mod mail;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // load config from env or file
    let app_config = Config::load();

    if let Err(e) = init_mongo_logger(app_config.logs_mongo_uri(), app_config.logs_db_name()).await
    {
        eprintln!("Failed to initialize MongoDB logger: {}", e);
        std::process::exit(1);
    }
    let client = match connect(app_config.db_url()).await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to connect to DB: {}", e);
            std::process::exit(1);
        }
    };

    let app_data = AppData::new(app_config.clone(), client.clone()).await;

    info!("Migrating database schema");
    sqlx::migrate!().run(client.as_sqlx_pool()).await.expect("");

    create_default_admin(
        &client,
        app_config.default_admin_email().clone(),
        app_config.default_admin_password().clone(),
    )
    .await;

    info!("Starting server");
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_data.clone())) //add application state with repositories and config
            .wrap(Logger::default()) // add logging middleware
            .configure(configure_endpoints) // add scopes and routes
    })
    .workers(app_config.workers()) // normally 1 worker per thread
    .bind((app_config.address().clone(), app_config.port()))? // address and port on which the server is listening to
    .run()
    .await
}
