//! End-to-End Database Workflow Tests
//!
//! These tests verify complete database operations through the API
//! including data persistence, retrieval, and consistency.

use serde_json::json;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tokio::time::timeout;

const SERVER_URL: &str = "http://localhost:8080";
const SERVER_STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Helper to start the server process
fn start_server() -> Child {
    let child = Command::new("cargo")
        .args(&["run", "--bin", "backend"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server process");

    // Give the server time to start up
    thread::sleep(Duration::from_secs(3));

    child
}

/// Helper to check if server is running
async fn wait_for_server() -> bool {
    let client = reqwest::Client::new();

    for _ in 0..20 {
        if let Ok(response) = client.get(SERVER_URL).send().await {
            if response.status().is_success() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
    false
}

/// Helper to make authenticated requests
async fn make_authenticated_request(
    method: &str, path: &str, token: &str, body: Option<serde_json::Value>,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", SERVER_URL, path);

    let mut request = match method {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => panic!("Unsupported HTTP method: {}", method),
    };

    request = request.header("X-Admin-Token", token);

    if let Some(json_body) = body {
        request = request.json(&json_body);
    }

    request.send().await
}

/// Helper to make regular requests
async fn make_request(
    method: &str, path: &str, body: Option<serde_json::Value>,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", SERVER_URL, path);

    let mut request = match method {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => panic!("Unsupported HTTP method: {}", method),
    };

    if let Some(json_body) = body {
        request = request.json(&json_body);
    }

    request.send().await
}

/// Helper to get admin authentication token
async fn get_admin_token() -> String {
    let login_data = json!({
        "email": "root",
        "password": "password"
    });

    let login_response = timeout(
        TEST_TIMEOUT,
        make_request("POST", "/v1/admins/auth/login", Some(login_data)),
    )
    .await
    .expect("Admin login request timed out")
    .expect("Admin login request failed");

    assert_eq!(login_response.status(), 200);

    let login_data: serde_json::Value = login_response
        .json()
        .await
        .expect("Failed to parse login response");
    login_data["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_database_health_check() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Test health endpoint which should include database status
    let response = timeout(TEST_TIMEOUT, make_request("GET", "/health", None))
        .await
        .expect("Health check request timed out")
        .expect("Health check request failed");

    assert_eq!(response.status(), 200);

    let health_data: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse health response");

    // Verify database connectivity
    assert!(health_data["database"].is_object());
    let database_status = &health_data["database"];
    assert!(database_status["status"].is_string());

    // Database should be healthy
    assert_eq!(database_status["status"], "healthy");

    // If there's an error field, it should be null for healthy database
    if database_status.get("error").is_some() {
        assert!(database_status["error"].is_null());
    }

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_admin_authentication_with_database() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Test admin login (this should query the database)
    let login_data = json!({
        "email": "root",
        "password": "password"
    });

    let login_response = timeout(
        TEST_TIMEOUT,
        make_request("POST", "/v1/admins/auth/login", Some(login_data)),
    )
    .await
    .expect("Admin login request timed out")
    .expect("Admin login request failed");

    assert_eq!(login_response.status(), 200);

    let login_response_data: serde_json::Value = login_response
        .json()
        .await
        .expect("Failed to parse login response");
    let token = login_response_data["token"].as_str().unwrap();

    // Verify the token by making an authenticated request
    // This should also query the database to validate the admin
    let auth_response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", token, None),
    )
    .await
    .expect("Authenticated request timed out")
    .expect("Authenticated request failed");

    // Should not be unauthorized (database lookup should work)
    assert_ne!(auth_response.status(), 401);

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_database_transaction_consistency() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Test that multiple operations maintain database consistency
    let token = get_admin_token().await;

    // Make multiple authenticated requests to test database connection pooling
    let mut handles = vec![];

    for i in 0..3 {
        let token_clone = token.clone();
        let handle = tokio::spawn(async move {
            let response = timeout(
                TEST_TIMEOUT,
                make_authenticated_request("GET", "/v1/admins/users", &token_clone, None),
            )
            .await
            .expect(&format!("Database request {} timed out", i))
            .expect(&format!("Database request {} failed", i));

            // All requests should succeed (or at least not fail due to database issues)
            assert_ne!(response.status(), 500); // Internal server error
            i
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let result = timeout(TEST_TIMEOUT, handle)
            .await
            .expect("Database consistency test timed out");
        assert!(result.is_ok());
    }

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_database_error_handling() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Test that the server handles database errors gracefully
    // This is harder to test without actually breaking the database connection
    // but we can test that the server responds appropriately to various scenarios

    // Test with invalid admin credentials (should query database and return 401)
    let invalid_login_data = json!({
        "email": "nonexistent@test.com",
        "password": "password"
    });

    let response = timeout(
        TEST_TIMEOUT,
        make_request("POST", "/v1/admins/auth/login", Some(invalid_login_data)),
    )
    .await
    .expect("Invalid login request timed out")
    .expect("Invalid login request failed");

    assert_eq!(response.status(), 401);

    // Test with wrong password (should query database and return 401)
    let wrong_password_data = json!({
        "email": "root",
        "password": "wrongpassword"
    });

    let response = timeout(
        TEST_TIMEOUT,
        make_request("POST", "/v1/admins/auth/login", Some(wrong_password_data)),
    )
    .await
    .expect("Wrong password request timed out")
    .expect("Wrong password request failed");

    assert_eq!(response.status(), 401);

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_database_connection_pooling() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Test concurrent database operations to verify connection pooling works
    let mut handles = vec![];

    for i in 0..10 {
        let handle = tokio::spawn(async move {
            // Each task will try to login (database operation)
            let login_data = json!({
                "email": "root",
                "password": "password"
            });

            let response = timeout(
                TEST_TIMEOUT,
                make_request("POST", "/v1/admins/auth/login", Some(login_data)),
            )
            .await
            .expect(&format!("Concurrent login {} timed out", i))
            .expect(&format!("Concurrent login {} failed", i));

            assert_eq!(response.status(), 200);

            let login_response: serde_json::Value = response
                .json()
                .await
                .expect("Failed to parse login response");
            assert!(login_response["token"].is_string());

            i
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let result = timeout(TEST_TIMEOUT, handle)
            .await
            .expect("Connection pooling test timed out");
        assert!(result.is_ok());
    }

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_database_migration_compatibility() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Test that the server can handle the current database schema
    // This is verified by successful authentication and basic operations

    let token = get_admin_token().await;

    // Test that we can make authenticated requests (verifies database schema compatibility)
    let response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", &token, None),
    )
    .await
    .expect("Schema compatibility request timed out")
    .expect("Schema compatibility request failed");

    // Should not get database-related errors (500)
    assert_ne!(response.status(), 500);

    // Test health endpoint to verify database schema is accessible
    let health_response = timeout(TEST_TIMEOUT, make_request("GET", "/health", None))
        .await
        .expect("Health check request timed out")
        .expect("Health check request failed");

    assert_eq!(health_response.status(), 200);

    let health_data: serde_json::Value = health_response
        .json()
        .await
        .expect("Failed to parse health response");
    assert_eq!(health_data["database"]["status"], "healthy");

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_database_performance_under_load() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Test database performance under moderate load
    let start_time = std::time::Instant::now();

    let mut handles = vec![];

    for i in 0..20 {
        let handle = tokio::spawn(async move {
            let login_data = json!({
                "email": "root",
                "password": "password"
            });

            let response = timeout(
                TEST_TIMEOUT,
                make_request("POST", "/v1/admins/auth/login", Some(login_data)),
            )
            .await
            .expect(&format!("Performance test request {} timed out", i))
            .expect(&format!("Performance test request {} failed", i));

            assert_eq!(response.status(), 200);
            i
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let result = timeout(TEST_TIMEOUT, handle)
            .await
            .expect("Performance test timed out");
        assert!(result.is_ok());
    }

    let duration = start_time.elapsed();

    // All 20 requests should complete within reasonable time
    assert!(
        duration.as_secs() < 10,
        "Database performance test took too long: {:?}",
        duration
    );

    let _ = server_process.kill();
    let _ = server_process.wait();
}
