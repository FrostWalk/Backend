//! End-to-End Launch Tests
//!
//! These tests properly launch the application and test complete workflows
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
    println!("Starting server process...");

    let child = Command::new("cargo")
        .args(&["run", "--bin", "backend"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server process");

    println!("Server process started, waiting for startup...");

    child
}

/// Helper to check if server is running
async fn wait_for_server() -> bool {
    let client = reqwest::Client::new();

    println!("Waiting for server to be ready...");

    for i in 0..10 {
        println!("Attempt {} to connect to server...", i + 1);

        if let Ok(response) = client.get(SERVER_URL).send().await {
            if response.status().is_success() {
                println!("Server is ready!");
                return true;
            }
        }

        thread::sleep(Duration::from_millis(1000));
    }

    println!("Server failed to start within timeout");
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
async fn test_launch_server_and_health_check() {
    println!("=== Starting E2E Test: Launch Server and Health Check ===");

    let mut server_process = start_server();

    // Wait for server to be ready
    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(server_ready.is_ok(), "Server startup timed out");
    assert!(server_ready.unwrap(), "Server is not responding");

    // Test health endpoint
    println!("Testing health endpoint...");
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

    println!("Health check passed!");

    // Clean up - ensure server stops
    println!("Shutting down server...");
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

    println!("=== E2E Test Complete ===");
}

#[tokio::test]
async fn test_launch_server_and_admin_login() {
    println!("=== Starting E2E Test: Launch Server and Admin Login ===");

    let mut server_process = start_server();

    // Wait for server to be ready
    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(server_ready.is_ok(), "Server startup timed out");
    assert!(server_ready.unwrap(), "Server is not responding");

    // Test admin login
    println!("Testing admin login...");
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

    println!("Admin login successful!");

    // Test authenticated request
    println!("Testing authenticated request...");
    let auth_response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", token, None),
    )
    .await
    .expect("Authenticated request timed out")
    .expect("Authenticated request failed");

    // Should not be unauthorized
    assert_ne!(auth_response.status(), 401);

    println!("Authenticated request successful!");

    // Clean up - ensure server stops
    println!("Shutting down server...");
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

    println!("=== E2E Test Complete ===");
}

#[tokio::test]
async fn test_launch_server_and_database_operations() {
    println!("=== Starting E2E Test: Launch Server and Database Operations ===");

    let mut server_process = start_server();

    // Wait for server to be ready
    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(server_ready.is_ok(), "Server startup timed out");
    assert!(server_ready.unwrap(), "Server is not responding");

    // Test health endpoint (includes database status)
    println!("Testing database connectivity...");
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
    assert_eq!(database_status["status"], "healthy");

    println!("Database connectivity verified!");

    // Test admin login (requires database)
    println!("Testing admin login with database...");
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

    // Test database operation through API
    println!("Testing database operation through API...");
    let auth_response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", token, None),
    )
    .await
    .expect("Database operation request timed out")
    .expect("Database operation request failed");

    // Should not get database-related errors (500)
    assert_ne!(auth_response.status(), 500);

    println!("Database operations successful!");

    // Clean up - ensure server stops
    println!("Shutting down server...");
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

    println!("=== E2E Test Complete ===");
}

#[tokio::test]
async fn test_launch_server_and_error_handling() {
    println!("=== Starting E2E Test: Launch Server and Error Handling ===");

    let mut server_process = start_server();

    // Wait for server to be ready
    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    assert!(server_ready.is_ok(), "Server startup timed out");
    assert!(server_ready.unwrap(), "Server is not responding");

    // Test wrong password
    println!("Testing wrong password...");
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
    println!("Wrong password correctly rejected!");

    // Test non-existent user
    println!("Testing non-existent user...");
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
    println!("Non-existent user correctly rejected!");

    // Test unauthenticated access
    println!("Testing unauthenticated access...");
    let response = timeout(TEST_TIMEOUT, make_request("GET", "/v1/admins/users", None))
        .await
        .expect("Unauthenticated request timed out")
        .expect("Unauthenticated request failed");

    assert_eq!(response.status(), 401);
    println!("Unauthenticated access correctly rejected!");

    // Clean up - ensure server stops
    println!("Shutting down server...");
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

    println!("=== E2E Test Complete ===");
}
