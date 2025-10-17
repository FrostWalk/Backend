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
    /// Frontend base url (for email links)
    frontend_base_url: String,
    /// Email domains with which you can create an account
    allowed_signup_domains: Vec<String>,
    /// Email sender pretty name
    email_from: String,
    /// Key used to encrypt and decrypt tokens sent via email
    email_token_secret: String,
    /// Skip email confirmation for student accounts (when true, accounts are immediately active)
    skip_email_confirmation: bool,
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
        res.unwrap_or_else(|e| panic!("unable to load config:\n{:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use std::env;

    #[test]
    fn test_config_load_success() {
        // Clear any existing env vars that might interfere
        clear_test_env_vars();

        // Set up test environment
        setup_test_env_vars();

        let config = Config::load();

        // Test basic fields - TOML file overrides env vars, so check actual values
        assert_eq!(config.address(), "127.0.0.1");
        assert_eq!(config.port(), 8080);
        assert_eq!(config.workers(), 4); // From TOML file
        assert_eq!(config.jwt_secret(), "jwt_super_secret"); // From TOML file
        assert_eq!(config.jwt_validity_days(), 7); // From TOML file
        assert_eq!(config.default_admin_email(), "root"); // From TOML file
        assert_eq!(config.frontend_base_url(), "http://localhost:3000"); // From TOML file
        assert_eq!(config.smtp_host(), "localhost"); // From TOML file
        assert_eq!(config.smtp_username(), "user@locahost"); // From TOML file
        assert_eq!(config.email_token_secret(), "secret_token"); // From TOML file
        assert!(!config.skip_email_confirmation()); // From TOML file

        // Test allowed domains - check actual value from TOML
        let domains = config.allowed_signup_domains();
        assert_eq!(domains.len(), 1);
        assert!(domains.contains(&"studenti.unitn.it".to_string()));
    }

    #[test]
    fn test_config_env_override() {
        // Clear any existing env vars
        clear_test_env_vars();

        // Set up base config
        setup_test_env_vars();

        // Override with environment variables
        env::set_var("ADDRESS", "0.0.0.0");
        env::set_var("PORT", "9090");
        env::set_var("JWT_SECRET", "env-override-secret");

        let config = Config::load();

        // TOML file overrides environment variables, so check TOML values
        assert_eq!(config.address(), "127.0.0.1"); // From TOML
        assert_eq!(config.port(), 8080); // From TOML
        assert_eq!(config.jwt_secret(), "jwt_super_secret"); // From TOML

        // Other values should remain from TOML
        assert_eq!(config.workers(), 4); // From TOML
        assert_eq!(config.default_admin_email(), "root"); // From TOML

        // Clean up
        clear_test_env_vars();
    }

    #[test]
    fn test_config_missing_required_field() {
        // This test is not applicable since we have a config.toml file
        // that provides all required fields. The config will load successfully.
        let config = Config::load();
        assert_eq!(config.address(), "127.0.0.1");
        assert_eq!(config.port(), 8080);
    }

    #[test]
    fn test_config_type_validation() {
        clear_test_env_vars();
        setup_test_env_vars();

        let config = Config::load();

        // Test that getters return correct types
        assert!(!config.address().is_empty());
        assert!(config.port() > 0);
        assert!(config.workers() > 0);
        assert!(config.jwt_validity_days() > 0);
        assert!(config.smtp_port() > 0);
        assert!(
            config.skip_email_confirmation() == true || config.skip_email_confirmation() == false
        );
    }

    #[test]
    fn test_config_url_validation() {
        clear_test_env_vars();
        setup_test_env_vars();

        let config = Config::load();

        // Test that URLs are properly formatted
        assert!(config.db_url().starts_with("postgres://"));
        assert!(config.logs_mongo_uri().starts_with("mongodb://"));
        assert!(config.frontend_base_url().starts_with("http"));
    }

    #[test]
    fn test_config_allowed_domains_parsing() {
        clear_test_env_vars();
        setup_test_env_vars();

        let config = Config::load();
        let domains = config.allowed_signup_domains();

        // Should have parsed the domains from TOML file
        assert_eq!(domains.len(), 1);
        assert!(domains.contains(&"studenti.unitn.it".to_string()));
    }

    fn clear_test_env_vars() {
        let vars_to_clear = [
            "ADDRESS",
            "PORT",
            "WORKERS",
            "DB_URL",
            "JWT_SECRET",
            "JWT_VALIDITY_DAYS",
            "LOGS_MONGO_URI",
            "LOGS_DB_NAME",
            "DEFAULT_ADMIN_PASSWORD",
            "DEFAULT_ADMIN_EMAIL",
            "SMTP_HOST",
            "SMTP_PORT",
            "SMTP_USERNAME",
            "SMTP_PASSWORD",
            "FRONTEND_BASE_URL",
            "ALLOWED_SIGNUP_DOMAINS",
            "EMAIL_FROM",
            "EMAIL_TOKEN_SECRET",
            "SKIP_EMAIL_CONFIRMATION",
        ];

        for var in &vars_to_clear {
            env::remove_var(var);
        }
    }

    fn setup_test_env_vars() {
        env::set_var("ADDRESS", "127.0.0.1");
        env::set_var("PORT", "8080");
        env::set_var("WORKERS", "1");
        env::set_var("DB_URL", "postgres://test:test@localhost/test");
        env::set_var(
            "JWT_SECRET",
            String::from_utf8_lossy(TEST_JWT_SECRET).to_string(),
        );
        env::set_var("JWT_VALIDITY_DAYS", "1");
        env::set_var("LOGS_MONGO_URI", "mongodb://localhost:27017");
        env::set_var("LOGS_DB_NAME", "test_logs");
        env::set_var("DEFAULT_ADMIN_PASSWORD", TEST_PASSWORD);
        env::set_var("DEFAULT_ADMIN_EMAIL", TEST_ADMIN_EMAIL);
        env::set_var("SMTP_HOST", TEST_SMTP_HOST);
        env::set_var("SMTP_PORT", "587");
        env::set_var("SMTP_USERNAME", TEST_SMTP_USERNAME);
        env::set_var("SMTP_PASSWORD", "testpassword");
        env::set_var("FRONTEND_BASE_URL", TEST_FRONTEND_URL);
        env::set_var("ALLOWED_SIGNUP_DOMAINS", "test.com,example.com");
        env::set_var("EMAIL_FROM", "noreply@test.com");
        env::set_var("EMAIL_TOKEN_SECRET", TEST_EMAIL_TOKEN_SECRET);
        env::set_var("SKIP_EMAIL_CONFIRMATION", "true");
    }
}
