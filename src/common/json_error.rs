use serde_json::json;
use serde_json::value::Value;

const ERROR_KEY: &str = "error";

pub trait JsonError: ToString {
    fn to_json_error(&self) -> Value {
        json!({
            ERROR_KEY: self.to_string(),
        })
    }
}
impl<T: ToString> JsonError for T {}
