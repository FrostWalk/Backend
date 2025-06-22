use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt::{Display, Formatter};
use utoipa::ToSchema;

/// Custom error type for generating JSON error responses
///
/// - `error`: Human-readable error message
/// - `status`: HTTP status code (not included in JSON response)
///
/// Used to standardize error responses across the API
#[derive(Serialize, Debug, ToSchema)]
pub struct JsonError {
    error: String,
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
        HttpResponse::build(self.status).json(&self)
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

/// Creates a `JsonError` response standard for database errors with internal server error status code (500)
pub(crate) fn database_error() -> JsonError {
    "database error".to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
}
