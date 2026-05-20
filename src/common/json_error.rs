use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt::{Display, Formatter};
use utoipa::ToSchema;
use uuid::Uuid;

/// Custom error type for generating JSON error responses
///
/// - `error`: Human-readable error message
/// - `log_id`: Unique identifier included in console logs for frontend tracking
/// - `status`: HTTP status code (not included in JSON response)
///
/// Used to standardize error responses across the API
#[derive(Serialize, Debug, ToSchema)]
pub struct JsonError {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    log_id: Option<String>,
    #[serde(skip)]
    status: StatusCode,
}

impl JsonError {
    /// Creates a new error instance with message and status code
    ///
    /// # Arguments
    /// * `msg` - Error message that can be converted to String
    /// * `status` - HTTP status code to associate with the error
    pub fn new(msg: impl Into<String>, status: StatusCode) -> Self {
        JsonError {
            error: msg.into(),
            log_id: None,
            status,
        }
    }

    /// Creates a new error instance with message, status code, and log ID
    ///
    /// # Arguments
    /// * `msg` - Error message that can be converted to String
    /// * `status` - HTTP status code to associate with the error
    /// * `log_id` - Unique identifier included in the console log line
    fn new_with_log_id(msg: impl Into<String>, status: StatusCode, log_id: Uuid) -> Self {
        JsonError {
            error: msg.into(),
            log_id: Some(log_id.to_string()),
            status,
        }
    }
}

impl Display for JsonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.error, self.status)
    }
}

impl ResponseError for JsonError {
    /// Returns the HTTP status code associated with this error
    fn status_code(&self) -> StatusCode {
        self.status
    }

    /// Converts error into Actix Web HTTP response
    ///
    /// Builds a JSON response containing the error message
    /// with the appropriate status code
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status).json(self)
    }
}

/// Convenience trait for converting Display types to JsonError
pub(crate) trait ToJsonError {
    /// Converts self into a JsonError with specified status code
    ///
    /// # Arguments
    /// * `status` - HTTP status code to associate with the error
    fn to_json_error(self, status: StatusCode) -> JsonError;
}

impl<T: Display> ToJsonError for T {
    fn to_json_error(self, status: StatusCode) -> JsonError {
        JsonError::new(self.to_string(), status)
    }
}

/// Creates a `JsonError` with a log ID for frontend tracking.
/// This function logs detailed error information to the console and returns a user-friendly error message.
pub(crate) fn error_with_log_id(
    detailed_msg: impl Into<String>, user_msg: impl Into<String>, status: StatusCode,
    log_level: log::Level,
) -> JsonError {
    let detailed_message = detailed_msg.into();
    let user_message = user_msg.into();
    let log_id = Uuid::new_v4();
    let log_message = format!("log_id={} {}", log_id, detailed_message);

    // Log the detailed error with the specific level
    match log_level {
        log::Level::Error => log::error!("{}", log_message),
        log::Level::Warn => log::warn!("{}", log_message),
        log::Level::Info => log::info!("{}", log_message),
        log::Level::Debug => log::debug!("{}", log_message),
        log::Level::Trace => log::trace!("{}", log_message),
    }

    JsonError::new_with_log_id(user_message, status, log_id)
}

/// Creates a `JsonError` with a log ID while keeping existing payload-aware call sites.
/// Payloads are intentionally not logged to the console.
pub(crate) fn error_with_log_id_and_payload<T>(
    detailed_msg: impl Into<String>, user_msg: impl Into<String>, status: StatusCode,
    log_level: log::Level, _payload: &T,
) -> JsonError {
    error_with_log_id(detailed_msg, user_msg, status, log_level)
}
