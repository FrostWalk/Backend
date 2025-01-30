use std::sync::{Arc, RwLock};

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct MarketConfig {
    address: String,
    port: u16,
    workers: usize,
    db_url: String,
    jwt_secret: String,
    jwt_expires_in: String,
    jwt_max_age: i32,
}
lazy_static! {
    static ref CONFIG: Arc<RwLock<MarketConfig>> = Arc::new(RwLock::new(MarketConfig::load()));
}
const LOCK_ERROR: &str = "Unable to lock CONFIG";
impl MarketConfig {
    pub(crate) fn load() -> Self {
        Figment::new()
            .merge(Env::prefixed("MARKET_"))
            .merge(Toml::file("config.toml"))
            .extract()
            .expect("Failed to load configuration")
    }

    pub(crate) fn address() -> String {
        CONFIG.read().expect(LOCK_ERROR).address.clone()
    }
    pub(crate) fn port() -> u16 {
        CONFIG.read().expect(LOCK_ERROR).port
    }
    pub(crate) fn workers() -> usize {
        CONFIG.read().expect(LOCK_ERROR).workers
    }
    pub(crate) fn db_url() -> String {
        CONFIG.read().expect(LOCK_ERROR).db_url.clone()
    }
    pub(crate) fn jwt_secret() -> String {
        CONFIG.read().expect(LOCK_ERROR).jwt_secret.clone()
    }
    pub(crate) fn jwt_expires_in() -> String {
        CONFIG.read().expect(LOCK_ERROR).jwt_expires_in.clone()
    }
    pub(crate) fn jwt_max_age() -> i32 {
        CONFIG.read().expect(LOCK_ERROR).jwt_max_age
    }
}
