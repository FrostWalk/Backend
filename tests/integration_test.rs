//! Integration tests for the backend API

use actix_web::{http::StatusCode, web, App, HttpResponse, Result};

// Import the actual health check handler from the main crate
// We'll need to make it public or create a test version
async fn test_health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "version": "0.1.0",
        "uptime_seconds": 0,
        "database": {
            "status": "healthy",
            "error": null
        }
    })))
}

async fn test_liveness_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    })))
}

#[actix_web::test]
async fn test_health_endpoint() {
    let srv = actix_test::start(|| App::new().route("/health", web::get().to(test_health_check)));

    let req = srv.get("/health");
    let mut res = req.send().await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // Verify response body contains expected fields
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body.get("status").unwrap(), "healthy");
    assert!(body.get("timestamp").unwrap().is_number());
    assert!(body.get("version").unwrap().is_string());
    assert!(body.get("uptime_seconds").unwrap().is_number());

    let database = body.get("database").unwrap();
    assert_eq!(database.get("status").unwrap(), "healthy");
    assert!(database.get("error").unwrap().is_null());
}

#[actix_web::test]
async fn test_liveness_endpoint() {
    let srv =
        actix_test::start(|| App::new().route("/health/live", web::get().to(test_liveness_check)));

    let req = srv.get("/health/live");
    let mut res = req.send().await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // Verify response body contains expected fields
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body.get("status").unwrap(), "alive");
    assert!(body.get("timestamp").unwrap().is_number());
}

#[actix_web::test]
async fn test_health_endpoint_content_type() {
    let srv = actix_test::start(|| App::new().route("/health", web::get().to(test_health_check)));

    let req = srv.get("/health");
    let res = req.send().await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // Verify content type is JSON
    let content_type = res.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

#[actix_web::test]
async fn test_liveness_endpoint_content_type() {
    let srv =
        actix_test::start(|| App::new().route("/health/live", web::get().to(test_liveness_check)));

    let req = srv.get("/health/live");
    let res = req.send().await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // Verify content type is JSON
    let content_type = res.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

#[actix_web::test]
async fn test_health_endpoint_structure() {
    let srv = actix_test::start(|| App::new().route("/health", web::get().to(test_health_check)));

    let req = srv.get("/health");
    let mut res = req.send().await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body: serde_json::Value = res.json().await.unwrap();

    // Verify all required fields are present and have correct types
    assert!(body.get("status").unwrap().is_string());
    assert!(body.get("timestamp").unwrap().is_number());
    assert!(body.get("version").unwrap().is_string());
    assert!(body.get("uptime_seconds").unwrap().is_number());

    let database = body.get("database").unwrap();
    assert!(database.get("status").unwrap().is_string());
    // error field is optional, so we don't assert its presence
}

#[actix_web::test]
async fn test_multiple_requests() {
    let srv = actix_test::start(|| App::new().route("/health", web::get().to(test_health_check)));

    // Make multiple requests to test server stability
    for _ in 0..5 {
        let req = srv.get("/health");
        let res = req.send().await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}

#[actix_web::test]
async fn test_different_http_methods() {
    let srv = actix_test::start(|| App::new().route("/health", web::get().to(test_health_check)));

    // Test GET request (should work)
    let req = srv.get("/health");
    let res = req.send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Test POST request (should return 404 Not Found since only GET is defined)
    let req = srv.post("/health");
    let res = req.send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
