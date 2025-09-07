//! Simplified Mock HAL Implementation
//! 
//! Minimal working version to get the build system functional

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::vec::Vec;

use crate::{
    HalTrait, HalResult, HalError, TestStatus, SelfTestResult,
    TimeProvider, PwmControl, PlatformInfo, PlatformCapabilities,
};

/// Simplified mock HAL for basic functionality
#[derive(Debug, Default)]
pub struct SimpleMockHal {
    duty_cycle: f32,
    initialized: bool,
}

impl SimpleMockHal {
    pub fn new() -> Self {
        Self::default()
    }
}

impl HalTrait for SimpleMockHal {
    fn init(&mut self) -> HalResult<()> {
        self.initialized = true;
        Ok(())
    }

    fn self_test(&mut self) -> HalResult<SelfTestResult> {
        Ok(SelfTestResult {
            overall_status: TestStatus::Pass,
            pwm_test: TestStatus::Pass,
            analog_test: TestStatus::Pass,
            storage_test: TestStatus::Pass,
            can_test: TestStatus::Pass,
            display_test: TestStatus::Pass,
            bluetooth_test: TestStatus::Pass,
            failures: Vec::new(),
        })
    }

    fn get_platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            platform_name: "SimpleMockHal",
            version: "0.1.0",
            capabilities: PlatformCapabilities {
                has_pwm: true,
                analog_channels: 8,
                storage_size: 1024,
                can_controllers: 2,
                display_resolution: (128, 160),
                has_bluetooth: true,
            },
        }
    }

    fn emergency_shutdown(&mut self) -> HalResult<()> {
        self.duty_cycle = 0.0;
        Ok(())
    }
}

impl TimeProvider for SimpleMockHal {
    fn now_us(&self) -> u64 {
        // Simple mock time - just return a fixed value for now
        1000000
    }

    fn now_ms(&self) -> u32 {
        1000
    }

    fn delay_us(&mut self, _microseconds: u32) -> HalResult<()> {
        Ok(())
    }

    fn delay_ms(&mut self, _milliseconds: u32) -> HalResult<()> {
        Ok(())
    }

    fn schedule_callback(&mut self, _delay_ms: u32, _callback: fn()) -> HalResult<crate::CallbackHandle> {
        Ok(crate::CallbackHandle(0))
    }

    fn cancel_callback(&mut self, _handle: crate::CallbackHandle) -> HalResult<()> {
        Ok(())
    }

    fn system_uptime_ms(&self) -> u32 {
        2000
    }
}

impl PwmControl for SimpleMockHal {
    fn set_duty_cycle(&mut self, duty_percent: f32) -> HalResult<()> {
        if duty_percent < 0.0 || duty_percent > 100.0 {
            return Err(HalError::InvalidParameter("Duty cycle out of range".into()));
        }
        self.duty_cycle = duty_percent;
        Ok(())
    }

    fn get_current_duty(&self) -> f32 {
        self.duty_cycle
    }

    fn enable(&mut self) -> HalResult<()> {
        Ok(())
    }

    fn disable(&mut self) -> HalResult<()> {
        self.duty_cycle = 0.0;
        Ok(())
    }

    fn get_timing_info(&self) -> HalResult<crate::PwmTimingInfo> {
        Ok(crate::PwmTimingInfo {
            cycle_position: 0.0,
            time_to_next_cycle_us: 1000,
            time_to_optimal_window_us: 500,
            in_optimal_window: true,
        })
    }

    fn set_duty_cycle_synchronized(&mut self, duty_percent: f32, _current_time_us: u64) -> HalResult<()> {
        self.set_duty_cycle(duty_percent)
    }

    fn set_duty_cycle_immediate(&mut self, duty_percent: f32) -> HalResult<()> {
        self.set_duty_cycle(duty_percent)
    }

    fn set_frequency(&mut self, _freq_hz: u32) -> HalResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_hal_basic_functionality() {
        let mut hal = SimpleMockHal::new();
        
        // Test initialization
        assert!(hal.init().is_ok());
        
        // Test self-test
        let test_result = hal.self_test().unwrap();
        assert_eq!(test_result.overall_status, TestStatus::Pass);
        
        // Test PWM control
        assert!(hal.set_duty_cycle(50.0).is_ok());
        assert_eq!(hal.get_current_duty(), 50.0);
        
        // Test invalid duty cycle
        assert!(hal.set_duty_cycle(-10.0).is_err());
        assert!(hal.set_duty_cycle(110.0).is_err());
        
        // Test emergency shutdown
        hal.set_duty_cycle(75.0).unwrap();
        assert!(hal.emergency_shutdown().is_ok());
        assert_eq!(hal.get_current_duty(), 0.0);
    }

    #[test]
    fn test_time_provider() {
        let hal = SimpleMockHal::new();
        
        // Test time functions (basic smoke test)
        let time_us = hal.now_us();
        let time_ms = hal.now_ms();
        
        assert!(time_us > 0);
        assert!(time_ms > 0);
    }

    #[test]
    fn test_platform_info() {
        let hal = SimpleMockHal::new();
        let info = hal.get_platform_info();
        
        assert_eq!(info.platform_name, "SimpleMockHal");
        assert_eq!(info.version, "0.1.0");
        assert!(info.capabilities.has_pwm);
    }
}