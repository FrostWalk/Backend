use actix_web::web::Json;
use serde::Serialize;
use std::fmt::{Display, Formatter};
use utoipa::ToSchema;
/// Represents an error for JSON responses.
///
/// This struct wraps an error message as a string, making it suitable for JSON serialization.
#[derive(Serialize, Debug, ToSchema)]
pub(crate) struct JsonError {
    #[schema(example = "Error message")]
    error: String,
}

impl Display for JsonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

/// Trait that provides a helper method to convert a type implementing `ToString` into a JSON error.
///
/// The default implementation returns an Actix-Web JSON response wrapping a `JsonError` that contains
/// the string representation of the error.
pub trait ToJsonError: ToString {
    /// Converts the value into a JSON error.
    ///
    /// This method wraps the error message produced by `to_string()` in a `JsonError`, then in Actix-Web's `Json` type.
    fn to_json_error(&self) -> Json<JsonError> {
        Json(JsonError {
            error: self.to_string(),
        })
    }
}
impl<T: ToString> ToJsonError for T {}
