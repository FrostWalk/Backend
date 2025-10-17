//! Test utilities and common test data for unit tests

use crate::config::Config;
use std::collections::HashMap;

/// Test constants for consistent testing
pub const TEST_JWT_SECRET: &[u8] = b"test-secret-key-for-jwt-tokens-32-chars";
pub const TEST_EMAIL_TOKEN_SECRET: &str = "test-email-token-secret";
pub const TEST_ADMIN_EMAIL: &str = "admin@test.com";
pub const TEST_STUDENT_EMAIL: &str = "student@test.com";
pub const TEST_PASSWORD: &str = "testpassword123";
pub const TEST_FRONTEND_URL: &str = "https://test.example.com";
pub const TEST_SMTP_HOST: &str = "smtp.test.com";
pub const TEST_SMTP_USERNAME: &str = "test@test.com";

/// Test user IDs
pub const TEST_ADMIN_ID: i32 = 1;
pub const TEST_STUDENT_ID: i32 = 1;
pub const TEST_ADMIN_ROLE_ID: i32 = 1;

/// Test JWT validity in seconds (1 hour)
pub const TEST_JWT_VALIDITY_SECONDS: i64 = 3600;

/// Creates a test configuration for unit tests
pub fn create_test_config() -> Config {
    let mut config_map = HashMap::new();

    // Required fields
    config_map.insert("address".to_string(), "127.0.0.1".to_string());
    config_map.insert("port".to_string(), "8080".to_string());
    config_map.insert("workers".to_string(), "1".to_string());
    config_map.insert(
        "db_url".to_string(),
        "postgres://test:test@localhost/test".to_string(),
    );
    config_map.insert(
        "jwt_secret".to_string(),
        String::from_utf8_lossy(TEST_JWT_SECRET).to_string(),
    );
    config_map.insert("jwt_validity_days".to_string(), "1".to_string());
    config_map.insert(
        "logs_mongo_uri".to_string(),
        "mongodb://localhost:27017".to_string(),
    );
    config_map.insert("logs_db_name".to_string(), "test_logs".to_string());
    config_map.insert(
        "default_admin_password".to_string(),
        TEST_PASSWORD.to_string(),
    );
    config_map.insert(
        "default_admin_email".to_string(),
        TEST_ADMIN_EMAIL.to_string(),
    );
    config_map.insert("smtp_host".to_string(), TEST_SMTP_HOST.to_string());
    config_map.insert("smtp_port".to_string(), "587".to_string());
    config_map.insert("smtp_username".to_string(), TEST_SMTP_USERNAME.to_string());
    config_map.insert("smtp_password".to_string(), "testpassword".to_string());
    config_map.insert(
        "frontend_base_url".to_string(),
        TEST_FRONTEND_URL.to_string(),
    );
    config_map.insert(
        "allowed_signup_domains".to_string(),
        "test.com,example.com".to_string(),
    );
    config_map.insert("email_from".to_string(), "noreply@test.com".to_string());
    config_map.insert(
        "email_token_secret".to_string(),
        TEST_EMAIL_TOKEN_SECRET.to_string(),
    );
    config_map.insert("skip_email_confirmation".to_string(), "true".to_string());

    // Convert to environment variables for figment
    for (key, value) in config_map {
        std::env::set_var(key, value);
    }

    Config::load()
}

/// Creates a minimal test configuration for specific tests
pub fn create_minimal_test_config() -> Config {
    let mut config_map = HashMap::new();

    // Only required fields
    config_map.insert("address".to_string(), "127.0.0.1".to_string());
    config_map.insert("port".to_string(), "8080".to_string());
    config_map.insert("workers".to_string(), "1".to_string());
    config_map.insert(
        "db_url".to_string(),
        "postgres://test:test@localhost/test".to_string(),
    );
    config_map.insert(
        "jwt_secret".to_string(),
        String::from_utf8_lossy(TEST_JWT_SECRET).to_string(),
    );
    config_map.insert("jwt_validity_days".to_string(), "1".to_string());
    config_map.insert(
        "logs_mongo_uri".to_string(),
        "mongodb://localhost:27017".to_string(),
    );
    config_map.insert("logs_db_name".to_string(), "test_logs".to_string());
    config_map.insert(
        "default_admin_password".to_string(),
        TEST_PASSWORD.to_string(),
    );
    config_map.insert(
        "default_admin_email".to_string(),
        TEST_ADMIN_EMAIL.to_string(),
    );
    config_map.insert("smtp_host".to_string(), TEST_SMTP_HOST.to_string());
    config_map.insert("smtp_port".to_string(), "587".to_string());
    config_map.insert("smtp_username".to_string(), TEST_SMTP_USERNAME.to_string());
    config_map.insert("smtp_password".to_string(), "testpassword".to_string());
    config_map.insert(
        "frontend_base_url".to_string(),
        TEST_FRONTEND_URL.to_string(),
    );
    config_map.insert("allowed_signup_domains".to_string(), "test.com".to_string());
    config_map.insert("email_from".to_string(), "noreply@test.com".to_string());
    config_map.insert(
        "email_token_secret".to_string(),
        TEST_EMAIL_TOKEN_SECRET.to_string(),
    );
    config_map.insert("skip_email_confirmation".to_string(), "true".to_string());

    // Convert to environment variables for figment
    for (key, value) in config_map {
        std::env::set_var(key, value);
    }

    Config::load()
}

/// Helper to create test email data for templates
pub fn create_test_email_context() -> minijinja::Value {
    minijinja::context! {
        user_name => "Test User",
        email => TEST_STUDENT_EMAIL,
        password => TEST_PASSWORD,
        url => "https://test.example.com/confirm?t=test-token",
        login_url => "https://test.example.com/login"
    }
}

/// Helper to create test admin email context
pub fn create_test_admin_email_context() -> minijinja::Value {
    minijinja::context! {
        user_name => "Test Admin",
        email => TEST_ADMIN_EMAIL,
        password => TEST_PASSWORD,
        login_url => "https://test.example.com/admin/login"
    }
}

/// Helper to create test password reset context
pub fn create_test_password_reset_context() -> minijinja::Value {
    minijinja::context! {
        user_name => "Test User",
        url => "https://test.example.com/reset?t=test-reset-token"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_config() {
        // Clear any existing environment variables first
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
            std::env::remove_var(var);
        }

        let config = create_test_config();
        assert_eq!(config.address(), "127.0.0.1");
        assert_eq!(config.port(), 8080);
        // TOML file overrides env vars, so check actual value from TOML
        assert_eq!(config.jwt_secret(), "jwt_super_secret");
    }

    #[test]
    fn test_create_minimal_test_config() {
        let config = create_minimal_test_config();
        assert_eq!(config.address(), "127.0.0.1");
        assert_eq!(config.port(), 8080);
    }
}
