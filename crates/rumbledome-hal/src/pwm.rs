//! PWM Control Interface
//! 
//! 🔗 T4-HAL-006: PWM Control Implementation
//! Derived From: T2-HAL-004 (4-Port MAC Solenoid Drive Requirements) + T2-PWM-001 (30 Hz PWM)
//! AI Traceability: Controls 4-port MAC solenoid for pneumatic boost control

#[cfg(not(feature = "std"))]
use alloc::{string::{String, ToString}, format};

#[cfg(feature = "std")]
use std::{string::{String, ToString}, format};

use crate::{HalResult, HalError, time::PwmTimingInfo};

/// PWM control interface for solenoid drive
/// 
/// Controls 4-port MAC solenoid with failsafe 0% duty = wastegate open
pub trait PwmControl {
    /// Set PWM output frequency in Hz
    /// 
    /// 🔗 T4-HAL-007: 30Hz PWM Frequency Implementation
    /// Derived From: T2-PWM-001 (30 Hz PWM Frequency)
    /// Default: 30 Hz for MAC solenoid compatibility
    /// Range: 20-50 Hz acceptable for most MAC solenoids
    fn set_frequency(&mut self, freq_hz: u32) -> HalResult<()>;
    
    /// Set PWM duty cycle as percentage (0.0-100.0)
    /// 
    /// 🔗 T4-HAL-008: Failsafe Duty Cycle Control
    /// Derived From: T1-SAFETY-001 (Overboost as Fault Condition)
    /// 0% = wastegate fully open (failsafe state)
    /// 100% = wastegate fully closed (maximum boost authority)
    fn set_duty_cycle(&mut self, duty_percent: f32) -> HalResult<()>;
    
    /// Get current duty cycle setting
    fn get_current_duty(&self) -> f32;
    
    /// Enable PWM output
    fn enable(&mut self) -> HalResult<()>;
    
    /// Disable PWM output (failsafe - forces 0% duty)
    fn disable(&mut self) -> HalResult<()>;
    
    /// Get PWM timing information for synchronized control
    /// 
    /// 🔗 T4-HAL-009: PWM Timing Coordination
    /// Derived From: T2-CONTROL-002 (PWM-Synchronized Control Architecture)
    /// Enables beat frequency prevention and jitter reduction
    fn get_timing_info(&self) -> HalResult<PwmTimingInfo>;
    
    /// Apply duty cycle with timing synchronization
    /// 
    /// Waits for optimal timing window before applying duty cycle change
    /// Used by control loop for coordinated updates
    fn set_duty_cycle_synchronized(&mut self, duty_percent: f32, current_time_us: u64) -> HalResult<()>;
    
    /// Force immediate duty cycle change (emergency override)
    /// 
    /// Bypasses timing synchronization for safety-critical situations
    /// Used for overboost protection and fault responses
    fn set_duty_cycle_immediate(&mut self, duty_percent: f32) -> HalResult<()>;
}

/// PWM-specific error types
#[derive(Debug, Clone, PartialEq)]
pub enum PwmError {
    /// Frequency out of acceptable range
    FrequencyOutOfRange { requested: u32, min: u32, max: u32 },
    /// Duty cycle out of range (must be 0.0-100.0)
    DutyCycleOutOfRange { requested: f32 },
    /// Hardware fault in PWM generation
    HardwareFault(String),
    /// PWM not initialized
    NotInitialized,
    /// Timing synchronization failed
    TimingSyncFailed,
}

impl From<PwmError> for HalError {
    fn from(error: PwmError) -> Self {
        match error {
            PwmError::FrequencyOutOfRange { requested, min, max } => {
                HalError::InvalidParameter(format!("PWM frequency {} Hz out of range {}-{} Hz", requested, min, max))
            },
            PwmError::DutyCycleOutOfRange { requested } => {
                HalError::InvalidParameter(format!("PWM duty cycle {} out of range 0.0-100.0", requested))
            },
            PwmError::HardwareFault(msg) => HalError::HardwareFault(format!("PWM: {}", msg)),
            PwmError::NotInitialized => HalError::InitializationFailed("PWM not initialized".to_string()),
            PwmError::TimingSyncFailed => HalError::CommunicationError("PWM timing synchronization failed".to_string()),
        }
    }
}

/// PWM configuration constants
/// 
/// 🔗 T4-HAL-010: PWM Configuration Constants
/// Derived From: T2-SOLENOID-001 + TechnicalSpecs.md PWM specifications
pub mod constants {
    /// Default PWM frequency for MAC solenoid (Hz)
    pub const PWM_FREQUENCY_HZ: u32 = 30;
    
    /// Minimum acceptable PWM frequency (Hz)
    pub const MIN_FREQUENCY_HZ: u32 = 20;
    
    /// Maximum acceptable PWM frequency (Hz)  
    pub const MAX_FREQUENCY_HZ: u32 = 50;
    
    /// Minimum duty cycle resolution (%)
    pub const MIN_DUTY_RESOLUTION: f32 = 0.1;
    
    /// Failsafe duty cycle (%)
    pub const FAILSAFE_DUTY: f32 = 0.0;
    
    /// Maximum response time for duty cycle changes (microseconds)
    pub const MAX_RESPONSE_TIME_US: u32 = 10_000; // 10ms
}