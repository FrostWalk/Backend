/// Helper macro to capture request payload for logging context
/// This should be called at the beginning of handlers that process request bodies
#[macro_export]
macro_rules! capture_request_payload {
    ($payload:expr) => {
        $crate::logging::context::update_request_payload_from(&$payload);
    };
}

/// Helper function to capture response status for logging context
/// This should be called when returning successful responses
pub fn capture_response_status(status: u16) {
    crate::logging::context::update_response_status(status);
}
