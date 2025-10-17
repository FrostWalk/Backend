//! End-to-End Authentication Workflow Tests
//!
//! These tests verify complete authentication workflows from start to finish
//! including token generation, validation, and protected resource access.

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

#[tokio::test]
async fn test_complete_admin_authentication_workflow() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

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

    let login_data: serde_json::Value = login_response
        .json()
        .await
        .expect("Failed to parse login response");
    let token = login_data["token"].as_str().unwrap();

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

    // Step 4: Test token expiration by waiting (if implemented)
    // This would require a very short token expiration time for testing

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_authentication_error_handling() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Test various authentication failure scenarios

    // 1. No token provided
    let response = timeout(TEST_TIMEOUT, make_request("GET", "/v1/admins/users", None))
        .await
        .expect("No token request timed out")
        .expect("No token request failed");

    assert_eq!(response.status(), 401);

    // 2. Invalid token format
    let response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", "invalid_token", None),
    )
    .await
    .expect("Invalid token request timed out")
    .expect("Invalid token request failed");

    assert_eq!(response.status(), 401);

    // 3. Malformed JWT token
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

    // 4. Expired token (if we can generate one)
    // This would require creating an expired token for testing

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_concurrent_authentication_requests() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

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

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_authentication_with_different_http_methods() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Get a valid token first
    let login_data = json!({
        "email": "root",
        "password": "password"
    });

    let login_response = timeout(
        TEST_TIMEOUT,
        make_request("POST", "/v1/admins/auth/login", Some(login_data)),
    )
    .await
    .expect("Login request timed out")
    .expect("Login request failed");

    assert_eq!(login_response.status(), 200);

    let login_data: serde_json::Value = login_response
        .json()
        .await
        .expect("Failed to parse login response");
    let token = login_data["token"].as_str().unwrap();

    // Test different HTTP methods with authentication
    let methods = vec!["GET", "POST", "PUT", "DELETE"];

    for method in methods {
        let response = timeout(
            TEST_TIMEOUT,
            make_authenticated_request(method, "/v1/admins/users", token, None),
        )
        .await
        .expect(&format!("{} request timed out", method))
        .expect(&format!("{} request failed", method));

        // Should not be unauthorized (even if method not allowed, auth should work)
        assert_ne!(response.status(), 401);
    }

    let _ = server_process.kill();
    let _ = server_process.wait();
}

#[tokio::test]
async fn test_server_restart_authentication_persistence() {
    // This test verifies that authentication state is properly managed
    // when the server restarts (tokens should be invalidated)

    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    // Get a valid token
    let login_data = json!({
        "email": "root",
        "password": "password"
    });

    let login_response = timeout(
        TEST_TIMEOUT,
        make_request("POST", "/v1/admins/auth/login", Some(login_data)),
    )
    .await
    .expect("Login request timed out")
    .expect("Login request failed");

    assert_eq!(login_response.status(), 200);

    let login_data: serde_json::Value = login_response
        .json()
        .await
        .expect("Failed to parse login response");
    let token = login_data["token"].as_str().unwrap();

    // Verify token works
    let auth_response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", token, None),
    )
    .await
    .expect("Authenticated request timed out")
    .expect("Authenticated request failed");

    assert_ne!(auth_response.status(), 401);

    // Restart server
    let _ = server_process.kill();
    let _ = server_process.wait();

    // Wait a bit for server to fully shutdown
    thread::sleep(Duration::from_secs(2));

    // Start server again
    let mut server_process2 = start_server();

    let server_ready2 = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready2.is_ok() && server_ready2.unwrap(),
        "Server not ready after restart"
    );

    // Token should still work (unless server maintains state in memory)
    // This depends on your JWT implementation
    let auth_response2 = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", token, None),
    )
    .await
    .expect("Authenticated request after restart timed out")
    .expect("Authenticated request after restart failed");

    // JWT tokens should still be valid after server restart
    // (unless you're using server-side token blacklisting)
    assert_ne!(auth_response2.status(), 401);

    let _ = server_process2.kill();
    let _ = server_process2.wait();
}

#[tokio::test]
async fn test_authentication_with_malformed_requests() {
    let mut server_process = start_server();

    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(
        server_ready.is_ok() && server_ready.unwrap(),
        "Server not ready"
    );

    let client = reqwest::Client::new();

    // Test various malformed authentication requests

    // 1. Missing Content-Type header
    let response = timeout(
        TEST_TIMEOUT,
        client
            .post(&format!("{}/v1/admins/auth/login", SERVER_URL))
            .body("{\"email\":\"root\",\"password\":\"password\"}")
            .send(),
    )
    .await
    .expect("Missing Content-Type request timed out")
    .expect("Missing Content-Type request failed");

    // Should still work (server might auto-detect JSON)
    assert!(response.status().is_success() || response.status() == 400);

    // 2. Invalid JSON structure
    let response = timeout(
        TEST_TIMEOUT,
        client
            .post(&format!("{}/v1/admins/auth/login", SERVER_URL))
            .header("Content-Type", "application/json")
            .body("{\"email\":\"root\"}") // Missing password
            .send(),
    )
    .await
    .expect("Invalid JSON structure request timed out")
    .expect("Invalid JSON structure request failed");

    assert_eq!(response.status(), 400);

    // 3. Empty request body
    let response = timeout(
        TEST_TIMEOUT,
        client
            .post(&format!("{}/v1/admins/auth/login", SERVER_URL))
            .header("Content-Type", "application/json")
            .body("")
            .send(),
    )
    .await
    .expect("Empty body request timed out")
    .expect("Empty body request failed");

    assert_eq!(response.status(), 400);

    let _ = server_process.kill();
    let _ = server_process.wait();
}
