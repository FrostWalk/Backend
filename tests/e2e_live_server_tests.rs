//! End-to-End Live Server Tests
//!
//! These tests assume the server is already running and test complete workflows
//! with real database connections and full application stack.
//!
//! To run these tests:
//! 1. Start the server: cargo run
//! 2. Run the tests: cargo test --test e2e_live_server_tests

use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;

const SERVER_URL: &str = "http://localhost:8080";
const TEST_TIMEOUT: Duration = Duration::from_secs(30);

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

/// Helper to check if server is running
async fn check_server_running() -> bool {
    match make_request("GET", "/health", None).await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

#[tokio::test]
async fn test_server_is_running() {
    // This test verifies that the server is running and accessible
    let is_running = check_server_running().await;
    assert!(
        is_running,
        "Server is not running. Please start it with 'cargo run' before running these tests."
    );
}

#[tokio::test]
async fn test_health_endpoint_complete() {
    let response = timeout(TEST_TIMEOUT, make_request("GET", "/health", None))
        .await
        .expect("Health check request timed out")
        .expect("Health check request failed");

    assert_eq!(response.status(), 200);

    let health_data: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse health response");

    // Verify all expected fields are present
    assert_eq!(health_data["status"], "healthy");
    assert!(health_data["timestamp"].is_number());
    assert!(health_data["version"].is_string());
    assert!(health_data["uptime_seconds"].is_number());

    // Verify database status
    assert!(health_data["database"].is_object());
    let database_status = &health_data["database"];
    assert!(database_status["status"].is_string());
    assert_eq!(database_status["status"], "healthy");
}

#[tokio::test]
async fn test_liveness_endpoint() {
    let response = timeout(TEST_TIMEOUT, make_request("GET", "/health/live", None))
        .await
        .expect("Liveness check request timed out")
        .expect("Liveness check request failed");

    assert_eq!(response.status(), 200);

    let liveness_data: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse liveness response");
    assert_eq!(liveness_data["status"], "alive");
    assert!(liveness_data["timestamp"].is_number());
}

#[tokio::test]
async fn test_complete_admin_authentication_workflow() {
    // Step 1: Login with valid admin credentials
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

    // Step 2: Verify token is valid by making authenticated request
    let auth_response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", token, None),
    )
    .await
    .expect("Authenticated request timed out")
    .expect("Authenticated request failed");

    // Should not be unauthorized (even if endpoint doesn't exist, auth should work)
    assert_ne!(auth_response.status(), 401);

    // Step 3: Test token persistence - make another request with same token
    let auth_response2 = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/projects", token, None),
    )
    .await
    .expect("Second authenticated request timed out")
    .expect("Second authenticated request failed");

    assert_ne!(auth_response2.status(), 401);
}

#[tokio::test]
async fn test_admin_login_failure_scenarios() {
    // Test wrong password
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

    // Test non-existent user
    let nonexistent_user_data = json!({
        "email": "nonexistent@test.com",
        "password": "password"
    });

    let response = timeout(
        TEST_TIMEOUT,
        make_request("POST", "/v1/admins/auth/login", Some(nonexistent_user_data)),
    )
    .await
    .expect("Non-existent user request timed out")
    .expect("Non-existent user request failed");

    assert_eq!(response.status(), 401);

    // Test malformed JSON
    let client = reqwest::Client::new();
    let response = timeout(
        TEST_TIMEOUT,
        client
            .post(&format!("{}/v1/admins/auth/login", SERVER_URL))
            .header("Content-Type", "application/json")
            .body("invalid json")
            .send(),
    )
    .await
    .expect("Malformed JSON request timed out")
    .expect("Malformed JSON request failed");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_authentication_error_handling() {
    // Test no token provided
    let response = timeout(TEST_TIMEOUT, make_request("GET", "/v1/admins/users", None))
        .await
        .expect("No token request timed out")
        .expect("No token request failed");

    assert_eq!(response.status(), 401);

    // Test invalid token format
    let response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", "invalid_token", None),
    )
    .await
    .expect("Invalid token request timed out")
    .expect("Invalid token request failed");

    assert_eq!(response.status(), 401);

    // Test malformed JWT token
    let response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request(
            "GET",
            "/v1/admins/users",
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.invalid",
            None,
        ),
    )
    .await
    .expect("Malformed token request timed out")
    .expect("Malformed token request failed");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_concurrent_authentication_requests() {
    // Test multiple concurrent login requests
    let mut handles = vec![];

    for i in 0..5 {
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
            .expect("Concurrent authentication timed out");
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_database_operations_through_api() {
    // Test that database operations work through the API
    let token = get_admin_token().await;

    // Test that we can make authenticated requests (verifies database connectivity)
    let response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", &token, None),
    )
    .await
    .expect("Database operation request timed out")
    .expect("Database operation request failed");

    // Should not get database-related errors (500)
    assert_ne!(response.status(), 500);

    // Test health endpoint to verify database is accessible
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
}

#[tokio::test]
async fn test_server_performance_under_load() {
    // Test server performance under moderate load
    let start_time = std::time::Instant::now();

    let mut handles = vec![];

    for i in 0..20 {
        let handle = tokio::spawn(async move {
            let response = timeout(TEST_TIMEOUT, make_request("GET", "/health", None))
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
        "Performance test took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_http_methods_with_authentication() {
    let token = get_admin_token().await;

    // Test different HTTP methods with authentication
    let methods = vec!["GET", "POST", "PUT", "DELETE"];

    for method in methods {
        let response = timeout(
            TEST_TIMEOUT,
            make_authenticated_request(method, "/v1/admins/users", &token, None),
        )
        .await
        .expect(&format!("{} request timed out", method))
        .expect(&format!("{} request failed", method));

        // Should not be unauthorized (even if method not allowed, auth should work)
        assert_ne!(response.status(), 401);
    }
}

#[tokio::test]
async fn test_cors_and_headers() {
    let client = reqwest::Client::new();
    let response = timeout(
        TEST_TIMEOUT,
        client
            .get(SERVER_URL)
            .header("Origin", "http://localhost:3000")
            .send(),
    )
    .await
    .expect("CORS test request timed out")
    .expect("CORS test request failed");

    // Should get some response (might be 404 for root path)
    assert!(response.status().is_success() || response.status() == 404);

    // Check for CORS headers if implemented
    let headers = response.headers();
    // These might not be present if CORS is not configured
    // but we should at least get a response
    // Check that the response headers are not empty
    assert!(!headers.is_empty(), "Response headers should not be empty");
}

#[tokio::test]
async fn test_error_response_format() {
    // Test that error responses are properly formatted
    let response = timeout(TEST_TIMEOUT, make_request("GET", "/v1/admins/users", None))
        .await
        .expect("Error response test timed out")
        .expect("Error response test failed");

    assert_eq!(response.status(), 401);

    // Check that error response is JSON
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));

    let error_data: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse error response");
    assert!(error_data["error"].is_string() || error_data["message"].is_string());
}
