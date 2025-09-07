//! RumbleDome Hardware Abstraction Layer
//! 
//! 🔗 T4-HAL-001: HAL Trait Definitions
//! Derived From: T2-HAL-001 (Platform-Independent Hardware Abstraction Design)
//! Decision Type: 🔗 Direct Derivation - Implementation of HAL abstraction interfaces
//! AI Traceability: Enables desktop simulation, multi-platform support, comprehensive testing

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{vec::Vec, string::String, format};

#[cfg(feature = "std")]
use std::{vec::Vec, string::String, format};

pub mod time;
pub mod pwm;

// Mock implementation for desktop testing
#[cfg(feature = "mock")]
pub mod simple_mock;

// TODO: Create remaining HAL modules as needed
// pub mod analog;
// pub mod storage; 
// pub mod can;
// pub mod display;
// pub mod gpio;
// pub mod bluetooth;

pub use time::*;
pub use pwm::*;

#[cfg(feature = "mock")]
pub use simple_mock::SimpleMockHal as MockHal;

/// Core error type for all HAL operations
#[derive(Debug, Clone, PartialEq)]
pub enum HalError {
    /// Hardware initialization failed
    InitializationFailed(String),
    /// Operation timeout
    Timeout,
    /// Invalid parameter provided
    InvalidParameter(String),
    /// Hardware fault detected
    HardwareFault(String),
    /// Operation not supported on this platform
    NotSupported,
    /// Communication error
    CommunicationError(String),
}

pub type HalResult<T> = Result<T, HalError>;

/// Main HAL trait that aggregates all hardware interfaces
/// 
/// 🔗 T4-HAL-002: Unified Hardware Interface
/// Derived From: T2-HAL-001 + T3-BUILD-002 (Crate Dependency Structure)
/// AI Traceability: Single point of hardware abstraction for core control logic
pub trait HalTrait: 
    TimeProvider + 
    PwmControl 
    // TODO: Add remaining HAL interfaces as modules are implemented
    // + AnalogInput + 
    // + NonVolatileStorage + 
    // + CanInterface + 
    // + DisplayInterface + 
    // + GpioControl + 
    // + BluetoothSerial 
{
    /// Initialize all hardware subsystems
    fn init(&mut self) -> HalResult<()>;
    
    /// Perform hardware self-test
    fn self_test(&mut self) -> HalResult<SelfTestResult>;
    
    /// Get hardware platform information
    fn get_platform_info(&self) -> PlatformInfo;
    
    /// Emergency shutdown - put all hardware in safe state
    fn emergency_shutdown(&mut self) -> HalResult<()>;
}

/// Hardware self-test results
#[derive(Debug, Clone)]
pub struct SelfTestResult {
    pub overall_status: TestStatus,
    pub pwm_test: TestStatus,
    pub analog_test: TestStatus,
    pub storage_test: TestStatus,
    pub can_test: TestStatus,
    pub display_test: TestStatus,
    pub bluetooth_test: TestStatus,
    pub failures: Vec<String>,
}

/// Platform information
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub platform_name: &'static str,
    pub version: &'static str,
    pub capabilities: PlatformCapabilities,
}

#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    pub has_pwm: bool,
    pub analog_channels: u8,
    pub storage_size: usize,
    pub can_controllers: u8,
    pub display_resolution: (u16, u16),
    pub has_bluetooth: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Pass,
    Fail,
    Warning,
    NotTested,
}