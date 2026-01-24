//! Error message quality tests
//!
//! Tests that verify error messages are helpful and distinguishable.

use truenas_exporter::error::ExporterError;

#[test]
fn test_auth_error_message_clarity() {
    // Given: An authentication error
    let error = ExporterError::Auth("Invalid API key".to_string());

    // When: Converting to string
    let message = format!("{}", error);

    // Then: Message should clearly indicate authentication issue
    assert!(message.contains("Authentication failed"));
    assert!(message.contains("Invalid API key"));
}

#[test]
fn test_api_error_message_clarity() {
    // Given: A TrueNAS API error
    let error = ExporterError::TrueNasApi("Endpoint not found".to_string());

    // When: Converting to string
    let message = format!("{}", error);

    // Then: Message should clearly indicate API issue
    assert!(message.contains("TrueNAS API error"));
    assert!(message.contains("Endpoint not found"));
}

#[test]
fn test_websocket_error_message_clarity() {
    // Given: A WebSocket error (needs a tungstenite::Error)
    use tungstenite::error::Error as WsError;
    use tungstenite::error::ProtocolError;

    let ws_err = WsError::Protocol(ProtocolError::ResetWithoutClosingHandshake);
    let error = ExporterError::WebSocket(ws_err);

    // When: Converting to string
    let message = format!("{}", error);

    // Then: Message should clearly indicate WebSocket issue
    assert!(message.contains("WebSocket error"));
}

#[test]
fn test_config_error_message_clarity() {
    // Given: A configuration error
    let error = ExporterError::Config("Missing required field: host".to_string());

    // When: Converting to string
    let message = format!("{}", error);

    // Then: Message should clearly indicate configuration issue
    assert!(message.contains("Configuration error"));
    assert!(message.contains("Missing required field"));
}

#[test]
fn test_json_error_message_clarity() {
    // Given: A JSON parsing error
    use serde_json;
    let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let error = ExporterError::Json(json_err);

    // When: Converting to string
    let message = format!("{}", error);

    // Then: Message should clearly indicate JSON issue
    assert!(message.contains("JSON error"));
}

#[test]
fn test_server_error_message_clarity() {
    // Given: A server error
    let error = ExporterError::Server("Failed to bind to port".to_string());

    // When: Converting to string
    let message = format!("{}", error);

    // Then: Message should clearly indicate server issue
    assert!(message.contains("HTTP server error"));
    assert!(message.contains("Failed to bind"));
}

#[test]
fn test_error_messages_are_distinguishable() {
    // Given: Different error types
    let auth_err = format!("{}", ExporterError::Auth("test".to_string()));
    let api_err = format!("{}", ExporterError::TrueNasApi("test".to_string()));
    let config_err = format!("{}", ExporterError::Config("test".to_string()));
    let server_err = format!("{}", ExporterError::Server("test".to_string()));

    // When: Comparing error messages
    // Then: Each should have a unique prefix
    assert!(auth_err.starts_with("Authentication failed"));
    assert!(api_err.starts_with("TrueNAS API error"));
    assert!(config_err.starts_with("Configuration error"));
    assert!(server_err.starts_with("HTTP server error"));

    // All should be different
    assert_ne!(auth_err, api_err);
    assert_ne!(api_err, config_err);
    assert_ne!(config_err, server_err);
}

#[test]
fn test_error_context_is_preserved() {
    // Given: Errors with specific context
    let detailed_error = ExporterError::TrueNasApi(
        "Failed to query pool.dataset.query: Connection timeout after 30s".to_string(),
    );

    // When: Converting to string
    let message = format!("{}", detailed_error);

    // Then: Context should be preserved in message
    assert!(message.contains("pool.dataset.query"));
    assert!(message.contains("Connection timeout"));
    assert!(message.contains("30s"));
}

#[test]
fn test_empty_error_message_handling() {
    // Given: An error with empty context
    let error = ExporterError::Auth(String::new());

    // When: Converting to string
    let message = format!("{}", error);

    // Then: Should still have error type prefix
    assert!(message.contains("Authentication failed"));
}

#[test]
fn test_error_debug_format() {
    // Given: An error instance
    let error = ExporterError::Auth("Invalid credentials".to_string());

    // When: Using debug format
    let debug_message = format!("{:?}", error);

    // Then: Should include variant name and details
    assert!(debug_message.contains("Auth"));
    assert!(debug_message.contains("Invalid credentials"));
}
