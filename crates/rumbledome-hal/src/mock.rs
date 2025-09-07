//! Mock Hardware Abstraction Layer Implementation
//! 
//! 🔗 T4-HAL-003: Mock HAL for Desktop Testing
//! Derived From: T2-HAL-001 (Platform-Independent Hardware Abstraction Design)
//! Decision Type: 🔗 Direct Derivation - Desktop simulation implementation
//! AI Traceability: Enables algorithm development, unit testing, desktop simulation

#[cfg(feature = "std")]
use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
    collections::HashMap,
    string::{String, ToString},
    vec::Vec,
    format,
    vec,
};

#[cfg(not(feature = "std"))]
use alloc::{
    sync::Arc,
    collections::HashMap,
    string::{String, ToString},
    vec::Vec,
    format,
    vec,
};

#[cfg(not(feature = "std"))]
use spin::Mutex;

#[cfg(feature = "mock")]
use log::{debug, info, warn};

use crate::{
    HalTrait, HalResult, HalError, TestStatus, SelfTestResult,
    TimeProvider, PwmControl, PwmError,
};

/// Mock hardware implementation for desktop testing
/// 
/// Simulates all RumbleDome hardware interfaces with realistic behavior
/// but no actual hardware dependencies.
#[derive(Debug, Clone)]
pub struct MockHal {
    /// Simulated system state
    state: Arc<Mutex<MockHalState>>,
}

#[derive(Debug)]
struct MockHalState {
    /// System start time for relative timing
    start_time: SystemTime,
    /// Current PWM duty cycle (0.0-100.0)
    current_duty_cycle: f32,
    /// PWM frequency in Hz
    pwm_frequency: f32,
    /// Simulated sensor readings
    sensor_readings: HashMap<String, f32>,
    /// Self-test results
    self_test_passed: bool,
    /// Hardware initialization state
    initialized: bool,
}

impl MockHal {
    /// Create new mock HAL instance
    pub fn new() -> Self {
        let state = MockHalState {
            start_time: SystemTime::now(),
            current_duty_cycle: 0.0,
            pwm_frequency: 30.0, // 30Hz default for MAC solenoid
            sensor_readings: HashMap::new(),
            self_test_passed: true,
            initialized: false,
        };

        let mock_hal = Self {
            state: Arc::new(Mutex::new(state)),
        };

        // Initialize default sensor readings
        mock_hal.initialize_sensors();
        
        mock_hal
    }

    /// Initialize default sensor readings for simulation
    fn initialize_sensors(&self) {
        let mut state = self.state.lock().unwrap();
        
        // Atmospheric pressure sensors (14.7 PSI at sea level)
        state.sensor_readings.insert("manifold_pressure".to_string(), 14.7);
        state.sensor_readings.insert("dome_input_pressure".to_string(), 80.0); // Typical feed pressure
        state.sensor_readings.insert("upper_dome_pressure".to_string(), 14.7); // Atmospheric initially
        state.sensor_readings.insert("lower_dome_pressure".to_string(), 14.7); // Atmospheric initially
        
        #[cfg(feature = "mock")]
        info!("MockHal: Initialized sensor readings");
    }

    /// Set simulated sensor reading for testing
    pub fn set_sensor_reading(&self, sensor: &str, value: f32) {
        let mut state = self.state.lock().unwrap();
        state.sensor_readings.insert(sensor.to_string(), value);
        
        #[cfg(feature = "mock")]
        debug!("MockHal: Set {} = {:.2}", sensor, value);
    }

    /// Get current sensor reading
    pub fn get_sensor_reading(&self, sensor: &str) -> Option<f32> {
        let state = self.state.lock().unwrap();
        state.sensor_readings.get(sensor).copied()
    }
}

impl Default for MockHal {
    fn default() -> Self {
        Self::new()
    }
}

impl HalTrait for MockHal {
    fn init(&mut self) -> HalResult<()> {
        let mut state = self.state.lock().unwrap();
        
        if state.initialized {
            return Err(HalError::InvalidParameter(
                "MockHal already initialized".to_string()
            ));
        }

        // Simulate hardware initialization
        state.initialized = true;
        
        #[cfg(feature = "mock")]
        info!("MockHal: Hardware initialized successfully");
        
        Ok(())
    }

    fn self_test(&mut self) -> HalResult<SelfTestResult> {
        let state = self.state.lock().unwrap();
        
        if !state.initialized {
            return Err(HalError::InvalidParameter(
                "Cannot run self-test before initialization".to_string()
            ));
        }

        // Simulate comprehensive self-test
        let overall_status = if state.self_test_passed {
            TestStatus::Pass
        } else {
            TestStatus::Fail
        };

        let result = SelfTestResult {
            overall_status,
            pwm_test: TestStatus::Pass,
            analog_test: TestStatus::Pass,
            storage_test: TestStatus::Pass,
            can_test: TestStatus::Pass,
            display_test: TestStatus::Pass,
            bluetooth_test: TestStatus::Pass,
            failures: if state.self_test_passed {
                Vec::new()
            } else {
                vec!["Simulated self-test failure".to_string()]
            },
        };

        #[cfg(feature = "mock")]
        info!("MockHal: Self-test completed with status: {:?}", overall_status);

        Ok(result)
    }

    fn get_platform_info(&self) -> crate::PlatformInfo {
        crate::PlatformInfo {
            name: "MockHal Desktop Simulator".to_string(),
            version: "0.1.0".to_string(),
            hardware_id: "MOCK-DESKTOP".to_string(),
            capabilities: vec![
                "PWM Control".to_string(),
                "Time Provider".to_string(),
                "Sensor Simulation".to_string(),
            ],
        }
    }

    fn emergency_shutdown(&mut self) -> HalResult<()> {
        // Set PWM to failsafe 0% duty cycle
        self.set_duty_cycle(0.0)?;
        
        #[cfg(feature = "mock")]
        warn!("MockHal: Emergency shutdown executed - all systems safe");
        
        Ok(())
    }
}

impl TimeProvider for MockHal {
    fn now_us(&self) -> u64 {
        let state = self.state.lock().unwrap();
        
        SystemTime::now()
            .duration_since(state.start_time)
            .unwrap_or_default()
            .as_micros() as u64
    }

    fn now_ms(&self) -> u32 {
        (self.now_us() / 1000) as u32
    }

    fn delay_us(&self, microseconds: u32) {
        #[cfg(feature = "std")]
        std::thread::sleep(std::time::Duration::from_micros(microseconds as u64));
    }

    fn delay_ms(&mut self, milliseconds: u32) {
        self.delay_us(milliseconds * 1000);
    }
}

impl PwmControl for MockHal {
    fn set_duty_cycle(&mut self, duty_percent: f32) -> HalResult<()> {
        if duty_percent < 0.0 || duty_percent > 100.0 {
            return Err(HalError::InvalidParameter(
                format!("PWM duty cycle {:.1}% out of range 0.0-100.0", duty_percent)
            ));
        }

        {
            let mut state = self.state.lock().unwrap();
            state.current_duty_cycle = duty_percent;
        }

        // Simulate solenoid response - update dome pressures based on duty cycle
        self.simulate_pneumatic_response(duty_percent);

        #[cfg(feature = "mock")]
        debug!("MockHal: PWM duty cycle set to {:.1}%", duty_percent);

        Ok(())
    }

    fn get_current_duty(&self) -> f32 {
        let state = self.state.lock().unwrap();
        state.current_duty_cycle
    }

    fn enable(&mut self) -> HalResult<()> {
        #[cfg(feature = "mock")]
        debug!("MockHal: PWM enabled");
        Ok(())
    }

    fn disable(&mut self) -> HalResult<()> {
        // Set to 0% duty cycle for failsafe
        self.set_duty_cycle(0.0)?;
        
        #[cfg(feature = "mock")]
        debug!("MockHal: PWM disabled (failsafe 0% duty)");
        Ok(())
    }

    fn get_timing_info(&self) -> HalResult<crate::PwmTimingInfo> {
        let state = self.state.lock().unwrap();
        let period_us = (1_000_000.0 / state.pwm_frequency) as u32;
        
        Ok(crate::PwmTimingInfo {
            period_us,
            next_cycle_start_us: self.now_us() + period_us as u64,
            duty_update_window_us: period_us / 10, // 10% of period
        })
    }

    fn set_duty_cycle_synchronized(&mut self, duty_percent: f32, _current_time_us: u64) -> HalResult<()> {
        // For mock implementation, synchronized is same as normal
        self.set_duty_cycle(duty_percent)
    }

    fn set_duty_cycle_immediate(&mut self, duty_percent: f32) -> HalResult<()> {
        // For mock implementation, immediate is same as normal
        self.set_duty_cycle(duty_percent)
    }
}

impl MockHal {
    /// Simulate pneumatic system response to PWM duty cycle changes
    fn simulate_pneumatic_response(&self, duty_cycle: f32) {
        let feed_pressure = self.get_sensor_reading("dome_input_pressure").unwrap_or(80.0);
        
        // Simplified pneumatic simulation:
        // 0% duty = Lower dome gets feed pressure, upper dome vented (wastegate open)
        // 100% duty = Upper dome gets feed pressure, lower dome vented (wastegate closed)
        
        let upper_dome_pressure = 14.7 + (feed_pressure - 14.7) * (duty_cycle / 100.0);
        let lower_dome_pressure = 14.7 + (feed_pressure - 14.7) * (1.0 - duty_cycle / 100.0);
        
        self.set_sensor_reading("upper_dome_pressure", upper_dome_pressure);
        self.set_sensor_reading("lower_dome_pressure", lower_dome_pressure);
        
        #[cfg(feature = "mock")]
        debug!(
            "MockHal: Pneumatic simulation - Upper: {:.1} PSI, Lower: {:.1} PSI", 
            upper_dome_pressure, lower_dome_pressure
        );
    }
}