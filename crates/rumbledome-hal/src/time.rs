//! Time Management Interface
//! 
//! 🔗 T4-HAL-003: Time Management Implementation
//! Derived From: Hardware.md time management requirements + 100Hz control loop timing
//! AI Traceability: Enables precise control loop timing, PWM synchronization

use crate::{HalResult, HalError};

/// Time management provider trait
/// 
/// Provides monotonic timestamps and timing services for control loops
pub trait TimeProvider {
    /// Get current timestamp in milliseconds since system start
    /// Must be monotonic and immune to clock adjustments
    fn now_ms(&self) -> u32;
    
    /// Get current timestamp in microseconds for high-precision timing
    /// Used for PWM synchronization and precise control timing
    fn now_us(&self) -> u64;
    
    /// Non-blocking delay for specified duration in milliseconds
    /// Returns immediately if duration is 0
    fn delay_ms(&mut self, duration_ms: u32) -> HalResult<()>;
    
    /// Non-blocking delay for specified duration in microseconds
    /// Used for precise PWM timing coordination
    fn delay_us(&mut self, duration_us: u32) -> HalResult<()>;
    
    /// Schedule a callback to execute after specified delay
    /// Used for calibration sequences and timed operations
    fn schedule_callback(&mut self, delay_ms: u32, callback: fn()) -> HalResult<CallbackHandle>;
    
    /// Cancel a previously scheduled callback
    fn cancel_callback(&mut self, handle: CallbackHandle) -> HalResult<()>;
    
    /// Check if system has been running for specified minimum time
    /// Used for system stabilization checks
    fn system_uptime_ms(&self) -> u32;
}

/// Handle for scheduled callbacks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallbackHandle(pub u32);

/// PWM timing information for synchronized control updates
/// 
/// 🔗 T4-HAL-004: PWM Synchronization Support
/// Derived From: T2-CONTROL-002 (PWM-Synchronized Control Architecture)
/// AI Traceability: Enables beat frequency prevention, jitter reduction
#[derive(Debug, Clone)]
pub struct PwmTimingInfo {
    /// Current PWM cycle position (0.0 = cycle start, 1.0 = cycle end)
    pub cycle_position: f32,
    /// Time until next PWM cycle start in microseconds
    pub time_to_next_cycle_us: u32,
    /// Time until optimal update window in microseconds
    pub time_to_optimal_window_us: u32,
    /// Whether we're currently in an optimal update window
    pub in_optimal_window: bool,
}

impl PwmTimingInfo {
    /// Check if current time is optimal for control updates
    pub fn is_optimal_update_time(&self, current_time_us: u64) -> bool {
        self.in_optimal_window
    }
    
    /// Get time until next optimal update window
    pub fn time_to_next_update_window_us(&self, current_time_us: u64) -> u32 {
        if self.in_optimal_window {
            0
        } else {
            self.time_to_optimal_window_us
        }
    }
}

/// Control update strategy for PWM synchronization
/// 
/// 🔗 T4-HAL-005: Control Update Strategy Implementation
/// Derived From: Architecture.md PWM synchronization requirements
#[derive(Debug, Clone, PartialEq)]
pub enum ControlUpdateStrategy {
    /// Standard asynchronous timing - no PWM coordination
    Asynchronous,
    /// Update at PWM cycle start
    CycleStart,
    /// Update at cycle midpoint (optimal for most applications)
    CycleMidpoint,
    /// Multiple updates per cycle with specified count
    SubCycle { updates_per_cycle: u8 },
}

/// Maximum acceptable delay for PWM synchronization (microseconds)
pub const MAX_ACCEPTABLE_DELAY_US: u32 = 1000; // 1ms max delay for 100Hz control loop