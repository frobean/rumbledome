//! Protocol error types

use thiserror::Error;

/// Protocol communication errors
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    
    /// Unsupported protocol version
    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(String),
    
    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    /// Invalid parameter value
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

impl ProtocolError {
    /// Create an invalid message error
    pub fn invalid_message(msg: impl Into<String>) -> Self {
        ProtocolError::InvalidMessage(msg.into())
    }
    
    /// Create a missing field error
    pub fn missing_field(field: impl Into<String>) -> Self {
        ProtocolError::MissingField(field.into())
    }
    
    /// Create an invalid parameter error
    pub fn invalid_parameter(param: impl Into<String>) -> Self {
        ProtocolError::InvalidParameter(param.into())
    }
}