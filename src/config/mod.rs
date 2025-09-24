use derive_getters::Getters;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;

const CONFIG_FILE: &str = "config.toml";

/// Application configs
#[derive(Deserialize, Getters, Clone)]
pub(crate) struct Config {
    /// Interface address on which the app is listening to `127.0.0.1`, `0.0.0.0`
    address: String,
    /// Local port on which the app is listening to `8080`
    port: u16,
    /// Number of workers for the server, normally one per thread
    workers: usize,
    /// Connection string for Postgres in standard format  
    db_url: String,
    /// Key used to sign and crypt jwt tokens, should be random and long
    jwt_secret: String,
    /// Seconds after which the token is considered expired, and the cookie is deleted
    jwt_validity_days: i64,
    /// Mongo's connection string for logs storage
    logs_mongo_uri: String,
    /// Mongo's database name for logs storage
    logs_db_name: String,
    /// Application default admin account password
    default_admin_password: String,
    /// Application default admin account email
    default_admin_email: String,
    /// Host of smtp server
    smtp_host: String,
    /// Port of smtp server
    smtp_port: u16,
    /// Username of the smtp server
    smtp_username: String,
    /// Password of the smtp server
    smtp_password: String,
    /// Application base url
    app_base_url: String,
}
impl Config {
    /// Loads and validates the application configuration from multiple sources.
    ///
    /// This function aggregates configuration values from environment variables and
    /// a TOML file, with the following precedence rules:
    /// 1. TOML file values are override environment variables
    /// 2. Environment variables override default values (if any exist in `Config` struct)
    ///
    /// # Configuration Sources
    /// - **Environment Variables**: Read from environment variables
    /// - **TOML File**: Looks for `config.toml` in the current working directory
    ///
    /// # Panics
    /// This function will panic if:
    /// - No valid configuration sources are found
    /// - Configuration values fail validation
    /// - There are type mismatches in configuration values
    /// - The TOML file contains syntax errors
    pub(crate) fn load() -> Self {
        let res: figment::Result<Config> = Figment::new()
            .merge(Env::raw())
            .merge(Toml::file(CONFIG_FILE)) // config files overwrite env vars
            .extract();

        // in case it fails, panic with a message and specific error
        res.unwrap_or_else(|e| panic!("unable to load config: {:?}", e))
    }
}
