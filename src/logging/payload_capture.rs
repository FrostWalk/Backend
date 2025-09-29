/// Helper function to capture response status for logging context
/// This should be called when returning successful responses
pub fn capture_response_status(status: u16) {
    crate::logging::context::update_response_status(status);
}
