use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub target: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub request_context: Option<RequestContext>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct RequestContext {
    pub request_id: Uuid,
    pub method: Option<String>,
    pub path: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub user_id: Option<String>,
    pub user_type: Option<String>,               // "admin" or "student"
    pub payload: Option<serde_json::Value>,      // Request body/payload data
    pub query_params: Option<serde_json::Value>, // Query parameters
    pub response_status: Option<u16>,            // HTTP status code of the response
}
