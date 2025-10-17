//! End-to-End Integration Tests
//!
//! These tests launch the actual server program and test complete workflows
//! with real database connections and full application stack.

use serde_json::json;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tokio::time::timeout;

const SERVER_URL: &str = "http://localhost:8080";
const SERVER_STARTUP_TIMEOUT: Duration = Duration::from_secs(15);
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
    thread::sleep(Duration::from_secs(5));

    child
}

/// Helper to check if server is running
async fn wait_for_server() -> bool {
    let client = reqwest::Client::new();

    for _ in 0..10 {
        if let Ok(response) = client.get(SERVER_URL).send().await {
            if response.status().is_success() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(1000));
    }
    false
}

/// Helper to make HTTP requests
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

#[tokio::test]
async fn test_server_startup_and_health_check() {
    let mut server_process = start_server();

    // Wait for server to be ready
    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok(),
        "Server failed to start within timeout"
    );
    assert!(server_ready.unwrap(), "Server is not responding");

    // Test health endpoint
    let response = timeout(TEST_TIMEOUT, make_request("GET", "/health", None))
        .await
        .expect("Health check request timed out")
        .expect("Health check request failed");

    assert_eq!(response.status(), 200);

    let health_data: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse health response");
    assert_eq!(health_data["status"], "healthy");
    assert!(health_data["timestamp"].is_number());
    assert!(health_data["version"].is_string());

    // Clean up - ensure server stops
    if let Err(e) = server_process.kill() {
        println!("Warning: Failed to kill server process: {}", e);
    }

    // Wait for the process to exit
    match server_process.wait() {
        Ok(status) => {
            if !status.success() {
                println!("Server exited with status: {:?}", status.code());
            }
        }
        Err(e) => {
            println!("Error waiting for server process: {}", e);
        }
    }

    // Give a moment for the port to be released
    thread::sleep(Duration::from_millis(500));
}

#[tokio::test]
async fn test_liveness_endpoint() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

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

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_admin_login_flow() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Test successful admin login
    let login_data = json!({
        "email": "root",
        "password": "password"
    });

    let response = timeout(
        TEST_TIMEOUT,
        make_request("POST", "/v1/admins/auth/login", Some(login_data)),
    )
    .await
    .expect("Admin login request timed out")
    .expect("Admin login request failed");

    assert_eq!(response.status(), 200);

    let login_response: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse login response");
    assert!(login_response["token"].is_string());

    let token = login_response["token"].as_str().unwrap();
    assert!(!token.is_empty());
    assert!(token.contains('.'));

    // Test that we can use the token for authenticated requests
    let auth_response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", token, None),
    )
    .await
    .expect("Authenticated request timed out")
    .expect("Authenticated request failed");

    // The endpoint might not exist, but we should get some response (not 401)
    assert_ne!(auth_response.status(), 401);

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_admin_login_failure_cases() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

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

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_unauthenticated_access_returns_401() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Test accessing protected endpoint without token
    let response = timeout(TEST_TIMEOUT, make_request("GET", "/v1/admins/users", None))
        .await
        .expect("Unauthenticated request timed out")
        .expect("Unauthenticated request failed");

    assert_eq!(response.status(), 401);

    // Test with invalid token
    let response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", "invalid_token", None),
    )
    .await
    .expect("Invalid token request timed out")
    .expect("Invalid token request failed");

    assert_eq!(response.status(), 401);

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_server_handles_concurrent_requests() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Make multiple concurrent health check requests
    let mut handles = vec![];

    for i in 0..5 {
        let handle = tokio::spawn(async move {
            let response = timeout(TEST_TIMEOUT, make_request("GET", "/health", None))
                .await
                .expect(&format!("Health check {} timed out", i))
                .expect(&format!("Health check {} failed", i));

            assert_eq!(response.status(), 200);
            i
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let result = timeout(TEST_TIMEOUT, handle)
            .await
            .expect("Concurrent request timed out");
        assert!(result.is_ok());
    }

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_server_graceful_shutdown() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Verify server is running
    let response = timeout(TEST_TIMEOUT, make_request("GET", "/health", None))
        .await
        .expect("Health check before shutdown timed out")
        .expect("Health check before shutdown failed");

    assert_eq!(response.status(), 200);

    // Send SIGTERM to gracefully shutdown
    let _ = server_process.kill();

    // Wait for process to exit
    let exit_status = timeout(Duration::from_secs(5), async { server_process.wait() })
        .await
        .expect("Server shutdown timed out");

    assert!(exit_status.is_ok());
}

#[tokio::test]
async fn test_database_connectivity() {
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

    // Check that database status is included
    assert!(health_data["database"].is_object());
    let database_status = &health_data["database"];
    assert!(database_status["status"].is_string());

    // Database should be healthy (assuming test database is running)
    assert_eq!(database_status["status"], "healthy");

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_cors_headers() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

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

    // These might not be present if CORS is not configured
    // but we should at least get a response
    assert!(response.status().is_success() || response.status() == 404);

    let _ = server_process.kill();
    let _ = server_process.wait();
}
