/// Stub for session-related functionality.

/// Create a dummy JWT.
pub fn create_jwt() -> String {
    "dummy_jwt".to_string()
}

/// Always verifies the dummy JWT successfully.
pub fn verify_jwt(_token: &str) -> bool {
    true
}