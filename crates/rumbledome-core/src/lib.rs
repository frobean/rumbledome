//! RumbleDome Core Control Logic
//! 
//! Hardware-independent implementation of the torque-based boost controller.
//! This crate contains all business logic, safety systems, and control algorithms
//! without any direct hardware dependencies.

pub mod config;
pub mod state;
pub mod control;
pub mod safety;
pub mod calibration;
pub mod learning;
pub mod error;

pub use config::*;
pub use state::*;
pub use control::*;
pub use safety::*;
pub use calibration::*;
pub use learning::*;
pub use error::*;

use rumbledome_hal::{HalTrait, SystemInputs};
use crate::calibration::{CalibrationAction};
use crate::state::{FaultCode};
use std::time::Instant;

/// Main RumbleDome controller
/// 
/// Coordinates all subsystems and implements the primary control loop.
/// Generic over HAL implementation to support multiple platforms.
pub struct RumbleDomeCore<H: HalTrait> {
    /// Current system state
    state: SystemState,
    
    /// User configuration
    config: SystemConfig,
    
    /// Learned calibration data
    learned_data: LearnedData,
    
    /// Control loop implementation
    control_loop: ControlLoop,
    
    /// Safety monitoring system
    safety_monitor: SafetyMonitor,
    
    /// Auto-calibration system
    calibration: AutoCalibration,
    
    /// Hardware abstraction layer
    hal: H,
    
    /// Last control loop execution time
    last_execution: Option<Instant>,
}

impl<H: HalTrait> RumbleDomeCore<H> {
    /// Create new RumbleDome controller instance
    pub fn new(hal: H, config: SystemConfig) -> Result<Self, CoreError> {
        Ok(Self {
            state: SystemState::Initializing,
            config,
            learned_data: LearnedData::default(),
            control_loop: ControlLoop::new()?,
            safety_monitor: SafetyMonitor::new(),
            calibration: AutoCalibration::new(),
            hal,
            last_execution: None,
        })
    }
    
    /// Initialize the system
    pub fn initialize(&mut self) -> Result<(), CoreError> {
        log::info!("Initializing RumbleDome system");
        
        // TODO: Load configuration and learned data from storage
        // TODO: Verify hardware functionality
        // TODO: Perform system self-tests
        
        self.state = SystemState::Idle;
        log::info!("RumbleDome initialization complete");
        Ok(())
    }
    
    /// Execute one iteration of the main control loop
    /// 
    /// Should be called at 100Hz (every 10ms) for proper operation.
    pub fn execute_control_cycle(&mut self) -> Result<(), CoreError> {
        let start_time = Instant::now();
        
        // Validate timing constraints
        if let Some(last) = self.last_execution {
            let elapsed = start_time.duration_since(last);
            if elapsed.as_millis() > 15 {
                log::warn!("Control loop timing violation: {}ms", elapsed.as_millis());
            }
        }
        
        // Read all system inputs
        let inputs = self.read_inputs()?;
        self.validate_inputs(&inputs)?;
        
        // Execute control logic based on current state
        match self.state {
            SystemState::Initializing => {
                // Initialization should be complete by now
                return Err(CoreError::InvalidState("Still initializing during control loop".into()));
            },
            
            SystemState::Idle => {
                // Monitor for arming conditions
                if self.check_arming_conditions(&inputs)? {
                    self.state = SystemState::Armed;
                    log::info!("System armed for boost control");
                }
                
                // Maintain 0% duty cycle in idle
                self.set_solenoid_output(0.0)?;
            },
            
            SystemState::Armed => {
                // Execute main control logic
                self.execute_armed_control(&inputs)?;
            },
            
            SystemState::Calibrating(_) => {
                // Execute calibration sequence
                self.execute_calibration_control(&inputs)?;
            },
            
            SystemState::OverboostCut => {
                // Safety cut - maintain 0% duty until recovery
                self.set_solenoid_output(0.0)?;
                
                if self.check_overboost_recovery(&inputs)? {
                    self.state = SystemState::Armed;
                    log::info!("Recovered from overboost condition");
                }
            },
            
            SystemState::Fault(_) => {
                // Maintain safe state until fault cleared
                self.set_solenoid_output(0.0)?;
                
                if self.check_fault_recovery(&inputs)? {
                    self.state = SystemState::Idle;
                    log::info!("Fault condition cleared");
                }
            },
        }
        
        // Update learning data if appropriate
        self.update_learning(&inputs)?;
        
        // Record execution timing
        self.last_execution = Some(start_time);
        
        Ok(())
    }
    
    /// Get current system state
    pub fn get_state(&self) -> &SystemState {
        &self.state
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> &SystemConfig {
        &self.config
    }
    
    /// Update system configuration
    pub fn update_config(&mut self, config: SystemConfig) -> Result<(), CoreError> {
        // TODO: Validate configuration
        self.config = config;
        log::info!("Configuration updated");
        Ok(())
    }
    
    /// Start calibration sequence
    pub fn start_calibration(&mut self, target_rpm: u16, target_boost: f32) -> Result<(), CoreError> {
        if !matches!(self.state, SystemState::Armed | SystemState::Idle) {
            return Err(CoreError::InvalidState("Cannot start calibration in current state".into()));
        }
        
        let calibration_state = CalibrationState::Conservative {
            target_rpm,
            target_boost,
            runs_completed: 0,
        };
        
        self.state = SystemState::Calibrating(calibration_state);
        self.calibration.start_session(target_rpm, target_boost)?;
        
        log::info!("Started calibration: {} RPM, {} PSI", target_rpm, target_boost);
        Ok(())
    }
    
    // Private implementation methods...
    
    fn read_inputs(&mut self) -> Result<SystemInputs, CoreError> {
        use rumbledome_hal::{AnalogChannel, AnalogReader, CanBus, TimeProvider};
        
        let timestamp_ms = self.hal.now_ms();
        
        // Read pressure sensors
        let dome_input_pressure = self.hal.read_pressure_psi(AnalogChannel::DomeInputPressure)
            .map_err(|e| CoreError::hardware(e))?;
        let upper_dome_pressure = self.hal.read_pressure_psi(AnalogChannel::UpperDomePressure)
            .map_err(|e| CoreError::hardware(e))?;
        let manifold_pressure_gauge = self.hal.read_pressure_psi(AnalogChannel::ManifoldPressure)
            .map_err(|e| CoreError::hardware(e))?;
        
        let sensors = rumbledome_hal::SensorReadings {
            dome_input_pressure,
            upper_dome_pressure,
            manifold_pressure_gauge,
            timestamp_ms,
        };
        
        // Read CAN data - TODO: Parse actual CAN messages
        // For now, return default values - needs CAN protocol implementation
        let can = rumbledome_hal::CanData::default();
        
        Ok(rumbledome_hal::SystemInputs {
            sensors,
            can,
            timestamp_ms,
        })
    }
    
    fn validate_inputs(&self, inputs: &SystemInputs) -> Result<(), CoreError> {
        let sensors = &inputs.sensors;
        let can = &inputs.can;
        
        // Validate sensor ranges
        let ranges = &self.config.safety.sensor_ranges;
        
        // Check dome input pressure
        let (min_dome, max_dome) = ranges.dome_input_pressure;
        if sensors.dome_input_pressure < min_dome || sensors.dome_input_pressure > max_dome {
            return Err(CoreError::input_validation(format!(
                "Dome input pressure out of range: {:.1} PSI", sensors.dome_input_pressure
            )));
        }
        
        // Check manifold pressure
        let (min_map, max_map) = ranges.manifold_pressure;
        if sensors.manifold_pressure_gauge < min_map || sensors.manifold_pressure_gauge > max_map {
            return Err(CoreError::input_validation(format!(
                "Manifold pressure out of range: {:.1} PSI", sensors.manifold_pressure_gauge
            )));
        }
        
        // Validate CAN data freshness
        let can_age = inputs.timestamp_ms.saturating_sub(can.timestamp_ms);
        if can_age > self.config.safety.can_timeout_ms {
            return Err(CoreError::input_validation(format!(
                "CAN data timeout: {}ms old", can_age
            )));
        }
        
        // Validate RPM is reasonable
        if can.rpm > self.config.safety.max_rpm {
            return Err(CoreError::input_validation(format!(
                "RPM too high: {} > {}", can.rpm, self.config.safety.max_rpm
            )));
        }
        
        Ok(())
    }
    
    fn check_arming_conditions(&self, inputs: &SystemInputs) -> Result<bool, CoreError> {
        let can = &inputs.can;
        
        // Engine must be running
        if can.rpm < self.config.safety.min_rpm_for_arming {
            return Ok(false);
        }
        
        // Must have valid torque data
        if can.desired_torque <= 0.0 || can.actual_torque <= 0.0 {
            return Ok(false);
        }
        
        // No active faults
        if matches!(self.state, SystemState::Fault(_)) {
            return Ok(false);
        }
        
        // All sensors must be reading reasonable values
        let sensors = &inputs.sensors;
        
        // Manifold pressure should not be in deep vacuum when engine running
        if sensors.manifold_pressure_gauge < -10.0 {
            return Ok(false);
        }
        
        // Dome system should have adequate pressure
        if sensors.dome_input_pressure < 5.0 {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    fn execute_armed_control(&mut self, inputs: &SystemInputs) -> Result<(), CoreError> {
        // Get active profile
        let active_profile = self.config.get_active_profile()
            .ok_or_else(|| CoreError::configuration("No active profile configured"))?;
        
        // Execute 3-level control hierarchy
        let control_result = self.control_loop.execute_cycle(
            inputs,
            &self.config,
            &self.learned_data,
            active_profile,
        )?;
        
        // Apply safety overrides
        let safety_action = self.safety_monitor.check_safety(
            inputs,
            &self.config,
            active_profile,
            control_result.duty_cycle,
        )?;
        
        let final_duty = match safety_action {
            crate::safety::SafetyAction::Continue => control_result.duty_cycle,
            crate::safety::SafetyAction::ImmediateCut { reason } => {
                log::warn!("Safety cut: {}", reason);
                self.state = SystemState::OverboostCut;
                0.0
            },
            crate::safety::SafetyAction::MaintainCut { reason } => {
                log::debug!("Maintaining safety cut: {}", reason);
                0.0
            },
            crate::safety::SafetyAction::AllowRecovery { message } => {
                log::info!("Safety recovery: {}", message);
                control_result.duty_cycle
            },
            crate::safety::SafetyAction::LimitDuty { max_duty, reason } => {
                log::warn!("Duty limited: {}", reason);
                control_result.duty_cycle.min(max_duty)
            },
        };
        
        // Update solenoid output
        self.set_solenoid_output(final_duty)?;
        
        // Update learning system with control result
        if control_result.boost_error.abs() < 0.5 {
            // Only learn from accurate control
            self.learned_data.update_calibration_point(
                inputs.can.rpm,
                control_result.target_boost,
                inputs.sensors.manifold_pressure_gauge,
                final_duty,
                inputs.timestamp_ms,
            )?;
        }
        
        // Log control diagnostics
        for diagnostic in &control_result.diagnostics {
            log::debug!("Control: {}", diagnostic);
        }
        
        Ok(())
    }
    
    fn execute_calibration_control(&mut self, inputs: &SystemInputs) -> Result<(), CoreError> {
        // Get current calibration duty cycle
        let calibration_duty = self.calibration.get_current_duty_cycle();
        
        // Process calibration cycle
        let calibration_result = self.calibration.process_calibration_cycle(
            inputs,
            &self.config,
            &mut self.learned_data,
        )?;
        
        // Update system state based on calibration result
        match calibration_result.next_action {
            CalibrationAction::Complete => {
                self.state = SystemState::Armed;
                log::info!("Calibration completed successfully");
            },
            CalibrationAction::Abort { reason } => {
                self.state = SystemState::Fault(FaultCode::SafetyFault {
                    check: "calibration".to_string(),
                    details: reason,
                });
                log::error!("Calibration aborted");
            },
            _ => {
                // Continue calibration - maintain current state
                if let SystemState::Calibrating(ref mut cal_state) = self.state {
                    if let Some(new_cal_state) = self.calibration.get_calibration_state() {
                        *cal_state = new_cal_state;
                    }
                }
            }
        }
        
        // Set solenoid output for calibration
        self.set_solenoid_output(calibration_duty)?;
        
        Ok(())
    }
    
    fn check_overboost_recovery(&self, inputs: &SystemInputs) -> Result<bool, CoreError> {
        // Get current active profile for recovery thresholds
        let active_profile = self.config.get_active_profile()
            .ok_or_else(|| CoreError::configuration("No active profile for overboost recovery"))?;
        
        let current_pressure = inputs.sensors.manifold_pressure_gauge;
        let recovery_threshold = active_profile.overboost_limit - active_profile.overboost_hysteresis;
        
        // Recovery allowed when pressure drops below threshold
        Ok(current_pressure < recovery_threshold)
    }
    
    fn check_fault_recovery(&self, inputs: &SystemInputs) -> Result<bool, CoreError> {
        // Check if fault conditions have cleared
        match &self.state {
            SystemState::Fault(fault_code) => {
                match fault_code {
                    FaultCode::SensorFault { .. } => {
                        // Try to validate inputs - if they pass, fault is cleared
                        self.validate_inputs(inputs).is_ok()
                    },
                    FaultCode::CanTimeout { .. } => {
                        // Check if CAN data is fresh
                        let can_age = inputs.timestamp_ms.saturating_sub(inputs.can.timestamp_ms);
                        can_age < self.config.safety.can_timeout_ms
                    },
                    FaultCode::HardwareFault { .. } |
                    FaultCode::SafetyFault { .. } |
                    FaultCode::WatchdogTimeout => {
                        // These require manual intervention
                        false
                    },
                    _ => {
                        // Other faults may be recoverable
                        true
                    }
                }
            },
            _ => true, // Not in fault state
        }
    }
    
    fn update_learning(&mut self, inputs: &SystemInputs) -> Result<(), CoreError> {
        // Update environmental factors based on current conditions
        let current_supply = inputs.sensors.dome_input_pressure;
        let baseline = self.learned_data.environmental_factors.supply_pressure_baseline;
        
        // Exponential moving average for supply pressure baseline
        let alpha = 0.01; // Slow adaptation
        let new_baseline = baseline * (1.0 - alpha) + current_supply * alpha;
        self.learned_data.environmental_factors.supply_pressure_baseline = new_baseline;
        
        // Update confidence metrics
        self.learned_data.calculate_confidence();
        
        Ok(())
    }
    
    fn set_solenoid_output(&mut self, duty_cycle: f32) -> Result<(), CoreError> {
        use rumbledome_hal::{PwmChannel, PwmController};
        
        // Clamp duty cycle to safe range
        let safe_duty = duty_cycle.clamp(0.0, 100.0);
        
        // Set PWM output
        self.hal.set_duty_cycle(PwmChannel::BoostSolenoid, safe_duty)
            .map_err(|e| CoreError::hardware(e))?;
        
        log::trace!("Solenoid output: {:.1}% duty cycle", safe_duty);
        
        Ok(())
    }
}