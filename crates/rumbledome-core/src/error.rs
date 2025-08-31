//! Error types for RumbleDome core logic

use thiserror::Error;
use rumbledome_hal::HalError;

/// Core system errors
#[derive(Error, Debug)]
pub enum CoreError {
    /// Hardware abstraction layer error
    #[error("Hardware error: {0}")]
    Hardware(#[from] HalError),
    
    /// Invalid system state for requested operation
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Calibration system error
    #[error("Calibration error: {0}")]
    Calibration(String),
    
    /// Control loop error
    #[error("Control error: {0}")]
    Control(String),
    
    /// Safety system violation
    #[error("Safety violation: {0}")]
    Safety(String),
    
    /// Learning system error
    #[error("Learning error: {0}")]
    Learning(String),
    
    /// Input validation error
    #[error("Input validation failed: {0}")]
    InputValidation(String),
    
    /// Data persistence error
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// Communication protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    /// System initialization error
    #[error("Initialization error: {0}")]
    Initialization(String),
    
    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl CoreError {
    /// Create an invalid state error
    pub fn invalid_state(msg: impl Into<String>) -> Self {
        CoreError::InvalidState(msg.into())
    }
    
    /// Create a configuration error
    pub fn configuration(msg: impl Into<String>) -> Self {
        CoreError::Configuration(msg.into())
    }
    
    /// Create a calibration error
    pub fn calibration(msg: impl Into<String>) -> Self {
        CoreError::Calibration(msg.into())
    }
    
    /// Create a control error
    pub fn control(msg: impl Into<String>) -> Self {
        CoreError::Control(msg.into())
    }
    
    /// Create a safety violation error
    pub fn safety(msg: impl Into<String>) -> Self {
        CoreError::Safety(msg.into())
    }
    
    /// Create a learning error
    pub fn learning(msg: impl Into<String>) -> Self {
        CoreError::Learning(msg.into())
    }
    
    /// Create an input validation error
    pub fn input_validation(msg: impl Into<String>) -> Self {
        CoreError::InputValidation(msg.into())
    }
    
    /// Create a storage error
    pub fn storage(msg: impl Into<String>) -> Self {
        CoreError::Storage(msg.into())
    }
    
    /// Create a protocol error
    pub fn protocol(msg: impl Into<String>) -> Self {
        CoreError::Protocol(msg.into())
    }
    
    /// Create an initialization error
    pub fn initialization(msg: impl Into<String>) -> Self {
        CoreError::Initialization(msg.into())
    }
    
    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        CoreError::Internal(msg.into())
    }
    
    /// Check if error is recoverable (system can continue with degraded function)
    pub fn is_recoverable(&self) -> bool {
        match self {
            CoreError::Hardware(hal_err) => {
                // Check if HAL error is recoverable
                matches!(hal_err, 
                    HalError::Communication(_) |
                    HalError::Timeout(_) |
                    HalError::SensorOutOfRange { .. }
                )
            },
            CoreError::InputValidation(_) => true,
            CoreError::Protocol(_) => true,
            CoreError::Storage(_) => true,
            CoreError::Control(_) => true,
            CoreError::Learning(_) => true,
            
            // These errors require system shutdown
            CoreError::Safety(_) => false,
            CoreError::InvalidState(_) => false,
            CoreError::Configuration(_) => false,
            CoreError::Calibration(_) => false,
            CoreError::Initialization(_) => false,
            CoreError::Internal(_) => false,
        }
    }
    
    /// Check if error requires immediate safety response (0% duty)
    pub fn requires_immediate_shutdown(&self) -> bool {
        match self {
            CoreError::Safety(_) => true,
            CoreError::Hardware(hal_err) => {
                matches!(hal_err, HalError::SafetyViolation(_))
            },
            _ => false,
        }
    }
    
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            CoreError::Safety(_) => ErrorSeverity::Critical,
            CoreError::Hardware(hal_err) => {
                if matches!(hal_err, HalError::SafetyViolation(_)) {
                    ErrorSeverity::Critical
                } else {
                    ErrorSeverity::High
                }
            },
            CoreError::InvalidState(_) => ErrorSeverity::High,
            CoreError::Configuration(_) => ErrorSeverity::High,
            CoreError::Initialization(_) => ErrorSeverity::High,
            CoreError::Calibration(_) => ErrorSeverity::Medium,
            CoreError::Control(_) => ErrorSeverity::Medium,
            CoreError::Storage(_) => ErrorSeverity::Medium,
            CoreError::Learning(_) => ErrorSeverity::Low,
            CoreError::InputValidation(_) => ErrorSeverity::Low,
            CoreError::Protocol(_) => ErrorSeverity::Low,
            CoreError::Internal(_) => ErrorSeverity::High,
        }
    }
}

/// Error severity levels for logging and response prioritization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Low severity - log but continue operation
    Low,
    /// Medium severity - may affect performance but system operational
    Medium,
    /// High severity - significant impact, may require intervention
    High,
    /// Critical severity - immediate safety response required
    Critical,
}

impl ErrorSeverity {
    /// Get human-readable severity description
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Low => "LOW",
            ErrorSeverity::Medium => "MEDIUM",
            ErrorSeverity::High => "HIGH",
            ErrorSeverity::Critical => "CRITICAL",
        }
    }
}