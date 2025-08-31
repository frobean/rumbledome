//! Hardware abstraction layer error types

use thiserror::Error;

/// Hardware abstraction layer errors
#[derive(Error, Debug)]
pub enum HalError {
    /// Generic hardware communication error
    #[error("Hardware communication failed: {0}")]
    Communication(String),
    
    /// Invalid parameter provided to HAL function
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    /// Hardware not initialized or not available
    #[error("Hardware not available: {0}")]
    NotAvailable(String),
    
    /// Timeout waiting for hardware operation
    #[error("Hardware operation timeout: {0}")]
    Timeout(String),
    
    /// ADC conversion error
    #[error("ADC conversion failed: {0}")]
    AdcError(String),
    
    /// PWM controller error
    #[error("PWM controller error: {0}")]
    PwmError(String),
    
    /// CAN bus error
    #[error("CAN bus error: {0}")]
    CanError(String),
    
    /// Display controller error
    #[error("Display error: {0}")]
    DisplayError(String),
    
    /// Storage system error
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// GPIO operation error
    #[error("GPIO error: {0}")]
    GpioError(String),
    
    /// Watchdog timer error
    #[error("Watchdog error: {0}")]
    WatchdogError(String),
    
    /// Sensor reading out of expected range
    #[error("Sensor reading out of range: {sensor} = {value} (expected {min}-{max})")]
    SensorOutOfRange {
        sensor: String,
        value: f32,
        min: f32,
        max: f32,
    },
    
    /// Critical safety constraint violation
    #[error("Safety violation: {0}")]
    SafetyViolation(String),
}

impl HalError {
    /// Create a communication error
    pub fn communication(msg: impl Into<String>) -> Self {
        HalError::Communication(msg.into())
    }
    
    /// Create an invalid parameter error
    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        HalError::InvalidParameter(msg.into())
    }
    
    /// Create a not available error
    pub fn not_available(msg: impl Into<String>) -> Self {
        HalError::NotAvailable(msg.into())
    }
    
    /// Create a timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        HalError::Timeout(msg.into())
    }
    
    /// Create a safety violation error
    pub fn safety_violation(msg: impl Into<String>) -> Self {
        HalError::SafetyViolation(msg.into())
    }
    
    /// Create a sensor out of range error
    pub fn sensor_out_of_range(sensor: impl Into<String>, value: f32, min: f32, max: f32) -> Self {
        HalError::SensorOutOfRange {
            sensor: sensor.into(),
            value,
            min,
            max,
        }
    }
}