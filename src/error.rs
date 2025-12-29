use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)] // MVP: Some variants will be used in future iterations
pub enum ExporterError {
    #[error("TrueNAS API error: {0}")]
    TrueNasApi(String),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tungstenite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("HTTP server error: {0}")]
    Server(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ExporterError>;
