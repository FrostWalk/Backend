//! Simple health check test

use actix_web::{http::StatusCode, web, App, HttpResponse, Result};

// Simple health check handler for testing
async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    })))
}

#[actix_web::test]
async fn test_health_check_returns_200() {
    let srv = actix_test::start(|| App::new().route("/health", web::get().to(health_check)));

    let req = srv.get("/health");
    let mut res = req.send().await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // Verify response body contains expected fields
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body.get("status").unwrap(), "healthy");
    assert!(body.get("timestamp").unwrap().is_number());
}

#[actix_web::test]
async fn test_health_check_content_type() {
    let srv = actix_test::start(|| App::new().route("/health", web::get().to(health_check)));

    let req = srv.get("/health");
    let res = req.send().await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // Verify content type is JSON
    let content_type = res.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}
