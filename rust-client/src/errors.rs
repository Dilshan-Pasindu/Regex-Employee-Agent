// =============================================================================
// errors.rs — Unified error types for the MCP client
// =============================================================================

use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("MCP protocol error: {0}")]
    Protocol(String),

    #[error("MCP server process failed to start: {0}")]
    ProcessStart(String),

    #[error("MCP server process exited unexpectedly")]
    ProcessDied,

    #[error("Request timed out after {secs} seconds")]
    Timeout { secs: u64 },

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error returned by MCP server: {0}")]
    Database(String),
}

pub type Result<T> = std::result::Result<T, McpError>;
