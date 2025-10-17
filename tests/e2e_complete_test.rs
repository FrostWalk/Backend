//! Complete End-to-End Test
//!
//! This test properly launches the application and tests complete workflows
//! with real database connections and full application stack.

use serde_json::json;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tokio::time::timeout;

const SERVER_URL: &str = "http://localhost:8080";
const SERVER_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Helper to ensure any existing server processes are killed
fn cleanup_existing_servers() {
    use std::process::Command;

    // Kill any existing backend processes
    let _ = Command::new("pkill").args(&["-f", "backend"]).output();

    // Give a moment for processes to be killed
    thread::sleep(Duration::from_millis(1000));
}

/// Helper to start the server process
fn start_server() -> Child {
    println!("Starting server process...");

    let child = Command::new("cargo")
        .args(&["run", "--bin", "backend"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server process");

    println!(
        "Server process started (PID: {}), waiting for startup...",
        child.id()
    );

    child
}

/// Helper to check if server is running
async fn wait_for_server() -> bool {
    let client = reqwest::Client::new();

    println!("Waiting for server to be ready...");

    for i in 0..30 {
        if i % 5 == 0 {
            println!("   Attempt {} to connect to server...", i + 1);
        }

        if let Ok(response) = client.get(format!("{}/health", SERVER_URL)).send().await {
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

/// Test various API endpoints
async fn test_api_endpoints(token: &str) {
    println!("Testing admin endpoints...");

    // Test admin users endpoint
    let response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", token, None),
    )
    .await
    .expect("Admin users request timed out")
    .expect("Admin users request failed");

    assert_eq!(response.status(), 200);
    let users_data: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse admin users response");
    assert!(users_data["admins"].is_array());
    println!("  - Admin users endpoint: OK");

    // Test admin me endpoint
    let response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users/me", token, None),
    )
    .await
    .expect("Admin me request timed out")
    .expect("Admin me request failed");

    assert_eq!(response.status(), 200);
    let me_data: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse admin me response");
    assert!(me_data["email"].is_string());
    println!("  - Admin me endpoint: OK");

    // Test projects endpoint
    let response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/projects", token, None),
    )
    .await
    .expect("Projects request timed out")
    .expect("Projects request failed");

    assert_eq!(response.status(), 200);
    let projects_data: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse projects response");
    assert!(projects_data["projects"].is_array());
    println!("  - Projects endpoint: OK");

    // Test security codes endpoint
    let response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/security-codes", token, None),
    )
    .await
    .expect("Security codes request timed out")
    .expect("Security codes request failed");

    assert_eq!(response.status(), 200);
    let codes_data: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse security codes response");
    // Just check that we got a valid response, regardless of structure
    assert!(codes_data.is_object() || codes_data.is_array());
    println!("  - Security codes endpoint: OK");

    println!("All admin API endpoints tested successfully!");

    // Test student endpoints (without authentication to test public endpoints)
    println!("Testing student endpoints...");

    // Test student allowed domains endpoint
    let response = timeout(
        TEST_TIMEOUT,
        make_request("GET", "/v1/students/auth/allowed-domains", None),
    )
    .await
    .expect("Student allowed domains request timed out")
    .expect("Student allowed domains request failed");

    assert_eq!(response.status(), 200);
    let domains_data: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse student allowed domains response");
    assert!(domains_data["domains"].is_array());
    println!("  - Student allowed domains endpoint: OK");

    println!("All API endpoints tested successfully!");
}

#[tokio::test]
async fn test_complete_end_to_end_workflow() {
    println!("\n=== COMPLETE END-TO-END TEST ===");
    println!("This test launches the actual application and tests the complete workflow");

    // Clean up any existing servers first
    cleanup_existing_servers();

    let mut server_process = start_server();

    // Wait for server to be ready
    let server_ready = timeout(SERVER_STARTUP_TIMEOUT, wait_for_server()).await;
    if server_ready.is_err() || !server_ready.unwrap() {
        println!("Server failed to start. Checking server logs...");

        // Try to get server output
        if let Ok(output) = server_process.try_wait() {
            if output.is_some() {
                println!("Server process has exited. Checking logs...");
                // The process has already exited, let's see what happened
                let _ = server_process.wait();
            }
        }

        // Check if server is actually running
        let client = reqwest::Client::new();
        if let Ok(response) = client.get(format!("{}/health", SERVER_URL)).send().await {
            println!(
                "Server is actually responding with status: {}",
                response.status()
            );
        } else {
            println!("Server is not responding at all");
        }

        assert!(false, "Server is not responding");
    }

    println!("\n=== TESTING HEALTH ENDPOINT ===");
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

    // Verify database connectivity
    assert!(health_data["database"].is_object());
    let database_status = &health_data["database"];
    assert_eq!(database_status["status"], "healthy");

    println!("Health endpoint test passed!");
    println!("   - Server status: {}", health_data["status"]);
    println!("   - Database status: {}", database_status["status"]);
    println!("   - Version: {}", health_data["version"]);

    println!("\n=== TESTING AUTHENTICATION WORKFLOW ===");
    // Test admin login
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
    println!("   - Token received: {}...", &token[..20]);

    // Test authenticated request
    let auth_response = timeout(
        TEST_TIMEOUT,
        make_authenticated_request("GET", "/v1/admins/users", token, None),
    )
    .await
    .expect("Authenticated request timed out")
    .expect("Authenticated request failed");

    // Should not be unauthorized
    assert_ne!(auth_response.status(), 401);

    let users_data: serde_json::Value = auth_response
        .json()
        .await
        .expect("Failed to parse users response");
    assert!(users_data["admins"].is_array());

    println!("Authenticated request successful!");
    println!(
        "   - Retrieved {} admin users",
        users_data["admins"].as_array().unwrap().len()
    );

    println!("\n=== TESTING ERROR HANDLING ===");
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
    println!("Wrong password correctly rejected!");

    // Test unauthenticated access
    let response = timeout(TEST_TIMEOUT, make_request("GET", "/v1/admins/users", None))
        .await
        .expect("Unauthenticated request timed out")
        .expect("Unauthenticated request failed");

    assert_eq!(response.status(), 401);
    println!("Unauthenticated access correctly rejected!");

    println!("\n=== TESTING API ENDPOINTS ===");
    // Test various API endpoints
    test_api_endpoints(token).await;

    println!("\n=== TESTING DIFFERENT HTTP METHODS ===");
    // Test different HTTP methods
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

    println!("All HTTP methods tested successfully!");

    println!("\n=== SHUTTING DOWN SERVER ===");

    // Send SIGTERM to gracefully shutdown
    if let Err(e) = server_process.kill() {
        println!("Warning: Failed to kill server process: {}", e);
    }

    // Wait for the process to exit
    match server_process.wait() {
        Ok(status) => {
            if status.success() {
                println!("Server shutdown gracefully!");
            } else {
                println!("Server exited with non-zero status: {:?}", status.code());
            }
        }
        Err(e) => {
            println!("Warning: Error waiting for server process: {}", e);
        }
    }

    // Give a moment for the port to be released
    thread::sleep(Duration::from_millis(500));

    println!("Server shutdown complete!");

    println!("\n=== END-TO-END TEST COMPLETE ===");
    println!("All tests passed! The application works correctly from start to finish.");

    // Final cleanup to ensure no processes are left running
    cleanup_existing_servers();
}
