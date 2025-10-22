use crate::jwt::get_user::LoggedUser;
use crate::logging::model::RequestContext;
use actix_web::{HttpMessage, HttpRequest};
use std::cell::RefCell;
use uuid::Uuid;

thread_local! {
    static REQUEST_CONTEXT: RefCell<Option<RequestContext>> = const { RefCell::new(None) };
}

/// Initialize the request context storage
pub fn init_request_context_storage() {
    // Thread-local storage is automatically initialized when first accessed
}

/// Extract request context from an HTTP request
pub fn extract_request_context(req: &HttpRequest) -> RequestContext {
    let request_id = Uuid::new_v4();
    let method = req.method().to_string();
    let path = req.path().to_string();
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .map(String::from);
    let ip_address = req.connection_info().realip_remote_addr().map(String::from);

    // Try to extract user information from request extensions
    let (user_id, user_type) = extract_user_info(req);

    // Extract query parameters
    let query_params = extract_query_params(req);

    RequestContext {
        request_id,
        method: Some(method),
        path: Some(path),
        user_agent,
        ip_address,
        user_id,
        user_type,
        payload: None, // Will be set later by handlers
        query_params,
        response_status: None, // Will be set later by handlers
    }
}

/// Extract user information from request extensions
fn extract_user_info(req: &HttpRequest) -> (Option<String>, Option<String>) {
    // Try to get admin user first
    if let Ok(admin) = req.extensions().get_admin() {
        return (Some(admin.admin_id.to_string()), Some("admin".to_string()));
    }

    // Try to get student user
    if let Ok(student) = req.extensions().get_student() {
        return (
            Some(student.student_id.to_string()),
            Some("student".to_string()),
        );
    }

    (None, None)
}

/// Extract query parameters from request
fn extract_query_params(req: &HttpRequest) -> Option<serde_json::Value> {
    let query_string = req.query_string();
    if query_string.is_empty() {
        return None;
    }

    // Parse query string into a JSON object
    let mut query_map = serde_json::Map::new();
    for param in req.query_string().split('&') {
        if let Some((k, v)) = param.split_once('=') {
            query_map.insert(k.to_string(), serde_json::Value::String(v.to_string()));
        }
    }

    if query_map.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(query_map))
    }
}

/// Set request context for the current thread
pub fn set_request_context(context: RequestContext) {
    REQUEST_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(context);
    });
}

/// Get request context for the current thread
pub fn get_request_context() -> Option<RequestContext> {
    REQUEST_CONTEXT.with(|ctx| ctx.borrow().clone())
}

/// Clear request context for the current thread
pub fn clear_request_context() {
    REQUEST_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = None;
    });
}

/// Update the current request context with payload data
pub fn update_request_payload(payload: serde_json::Value) {
    REQUEST_CONTEXT.with(|ctx| {
        if let Some(ref mut context) = *ctx.borrow_mut() {
            context.payload = Some(payload);
        }
    });
}

/// Update the current request context with response status code
pub fn update_response_status(status: u16) {
    REQUEST_CONTEXT.with(|ctx| {
        if let Some(ref mut context) = *ctx.borrow_mut() {
            context.response_status = Some(status);
        }
    });
}

/// Capture the current request payload into the request context
/// This should be called when an error occurs to capture the request data
pub fn capture_current_request_payload<T: serde::Serialize>(payload: &T) {
    if let Ok(payload_value) = serde_json::to_value(payload) {
        update_request_payload(payload_value);
    }
}
