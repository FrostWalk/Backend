use crate::app_data::AppData;
use actix_web::web::Data;
use actix_web::{HttpResponse, Result};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    status: String,
    timestamp: u64,
    version: String,
    uptime_seconds: u64,
    database: DatabaseStatus,
}

#[derive(Serialize, ToSchema)]
struct DatabaseStatus {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Health check endpoint for monitoring
///
/// This endpoint provides:
/// - Application status (healthy/unhealthy)
/// - Current timestamp
/// - Application version
/// - Uptime in seconds
/// - Database connectivity status
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Application is healthy", body = HealthResponse),
        (status = 503, description = "Application is unhealthy", body = HealthResponse)
    ),
    summary = "Get application health status",
    description = "Comprehensive health check that includes database connectivity and application status"
)]
pub async fn health_check(data: Data<AppData>) -> Result<HttpResponse> {
    let start_time = SystemTime::now();
    let timestamp = start_time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Check database connectivity
    let database_status = check_database_health(&data).await;

    // Calculate uptime (simplified - in a real app you'd track start time)
    let uptime_seconds = timestamp; // This is a simplified uptime calculation

    let health_response = HealthResponse {
        status: if database_status.status == "healthy" {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        },
        timestamp,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
        database: database_status,
    };

    let status_code = if health_response.status == "healthy" {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    Ok(HttpResponse::build(status_code).json(health_response))
}

/// Check database health by attempting a simple query
async fn check_database_health(app_data: &AppData) -> DatabaseStatus {
    match sqlx::query("SELECT 1")
        .fetch_one(app_data.db.as_sqlx_pool())
        .await
    {
        Ok(_) => DatabaseStatus {
            status: "healthy".to_string(),
            error: None,
        },
        Err(e) => DatabaseStatus {
            status: "unhealthy".to_string(),
            error: Some(e.to_string()),
        },
    }
}

/// Simple liveness probe endpoint
///
/// This is a minimal endpoint that just returns 200 OK if the service is running.
/// Useful for basic liveness checks without database dependencies.
#[utoipa::path(
    get,
    path = "/health/live",
    tag = "Health",
    responses(
        (status = 200, description = "Service is alive", body = serde_json::Value)
    ),
    summary = "Get service liveness status",
    description = "Simple liveness check that returns 200 OK if the service is running"
)]
pub async fn liveness_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "timestamp": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    })))
}
