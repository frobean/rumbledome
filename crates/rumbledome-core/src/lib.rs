//! RumbleDome Core Control Logic
//! 
//! 🔗 T4-CORE-001: Core Control Architecture
//! Derived From: T3-BUILD-003 (Core Control State Machine) + T2-CONTROL-003 (3-Level Control Hierarchy)
//! Decision Type: 🔗 Direct Derivation - Implementation of hardware-independent control logic
//! AI Traceability: Enables desktop testing, safety-critical algorithm validation

#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::{String, ToString};

pub mod config;
pub mod state;
// TODO: Implement remaining core modules
// pub mod control;
// pub mod learning;
// pub mod safety;
// pub mod torque_following;

pub use config::*;
pub use state::*;

use rumbledome_hal::{HalTrait, HalResult, HalError};

/// Core system error types
/// 
/// 🔗 T4-CORE-002: Error Classification System
/// Derived From: Safety.md fault response hierarchy
#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    /// Configuration validation failed
    ConfigurationError(String),
    /// Hardware abstraction layer error
    HalError(HalError),
    /// Safety limit violation detected
    SafetyViolation(String),
    /// Learning system error
    LearningError(String),
    /// CAN communication error
    CanError(String),
    /// System not in valid state for operation
    InvalidState(String),
    /// Calibration process error
    CalibrationError(String),
    /// Sensor validation failed
    SensorError(String),
}

impl From<HalError> for CoreError {
    fn from(error: HalError) -> Self {
        CoreError::HalError(error)
    }
}

/// Main RumbleDome control system
/// 
/// 🔗 T4-CORE-003: RumbleDome Core Implementation
/// Derived From: T3-BUILD-003 + T2-CONTROL-001 (Priority Hierarchy Implementation)
/// AI Traceability: Central coordination of all system functions
pub struct RumbleDomeCore<H: HalTrait> {
    /// Current system state
    pub state: SystemState,
    /// User configuration (5 parameters)
    pub config: SystemConfig,
    /// Hardware abstraction layer
    pub hal: H,
    /// Control loop statistics  
    pub stats: ControlLoopStats,
    // TODO: Add these back when modules are implemented
    // /// Learned calibration data
    // pub learned_data: LearnedData,
    // /// Torque-following control logic
    // pub torque_following: TorqueFollowing,
    // /// Safety monitoring system
    // pub safety_monitor: SafetyMonitor,
    // /// Auto-calibration system
    // pub calibration: AutoCalibration,
}

/// System inputs from sensors and CAN
/// 
/// 🔗 T4-CORE-004: System Input Structure
/// Derived From: T2-HAL-005 (Ford S550 CAN Signal Integration) + sensor specifications
#[derive(Debug, Clone)]
pub struct SystemInputs {
    /// Engine RPM from CAN
    pub rpm: u16,
    /// Desired torque from ECU (Nm)
    pub desired_torque: f32,
    /// Actual torque from ECU (Nm) 
    pub actual_torque: f32,
    /// Manifold pressure (PSI gauge)
    pub manifold_pressure: f32,
    /// Dome input pressure (PSI gauge)
    pub dome_input_pressure: f32,
    /// Upper dome pressure (PSI gauge) 
    pub upper_dome_pressure: f32,
    /// Lower dome pressure (PSI gauge)
    pub lower_dome_pressure: f32,
    /// Current aggression setting (0.0-1.0)
    pub aggression: f32,
    /// Scramble button state
    pub scramble_active: bool,
    /// System timestamp (milliseconds)
    pub timestamp_ms: u32,
}

/// Control loop performance statistics
/// 
/// 🔗 T4-CORE-005: Performance Monitoring
/// Derived From: Performance requirements + diagnostic needs
#[derive(Debug, Clone, Default)]
pub struct ControlLoopStats {
    /// Total control cycles executed
    pub cycles_executed: u64,
    /// Average cycle time (microseconds)
    pub avg_cycle_time_us: u32,
    /// Maximum cycle time (microseconds)
    pub max_cycle_time_us: u32,
    /// Control cycles that exceeded target timing
    pub timing_violations: u32,
    /// Safety interventions triggered
    pub safety_interventions: u32,
    /// Learning updates applied
    pub learning_updates: u32,
    /// Last update timestamp
    pub last_update_ms: u32,
}

impl<H: HalTrait> RumbleDomeCore<H> {
    /// Create new RumbleDome core instance
    /// 
    /// 🔗 T4-CORE-006: System Initialization
    /// Derived From: T3-BUILD-003 (Core Control State Machine)
    pub fn new(hal: H, config: SystemConfig) -> Self {
        Self {
            state: SystemState::Initializing,
            config,
            hal,
            stats: ControlLoopStats::default(),
        }
    }
    
    /// Initialize system and perform self-test
    /// 
    /// 🔗 T4-CORE-007: System Initialization Process
    /// Derived From: T1-SAFETY-002 (Defense in Depth) + startup requirements
    pub fn initialize(&mut self) -> Result<(), CoreError> {
        self.state = SystemState::Initializing;
        
        // Initialize hardware
        self.hal.init()?;
        
        // Perform self-test
        let self_test = self.hal.self_test()?;
        if self_test.overall_status != rumbledome_hal::TestStatus::Pass {
            self.state = SystemState::Fault(FaultCode::SelfTestFailed);
            return Err(CoreError::SafetyViolation("Hardware self-test failed".to_string()));
        }
        
        // TODO: Load learned data when learning module is implemented
        // self.learned_data = LearnedData::load_from_storage(&mut self.hal)?;
        
        // TODO: Initialize safety monitor when safety module is implemented  
        // self.safety_monitor.initialize(&self.config)?;
        
        // Transition to idle state
        self.state = SystemState::Idle;
        
        Ok(())
    }
    
    /// Execute one control cycle
    /// 
    /// 🔗 T4-CORE-008: Main Control Loop Implementation
    /// Derived From: T3-BUILD-005 (3-Level Control Hierarchy Implementation)
    /// Must be called at 100 Hz for proper system operation
    pub fn execute_control_cycle(&mut self) -> Result<(), CoreError> {
        let cycle_start = self.hal.now_us();
        self.stats.cycles_executed += 1;
        
        // Read system inputs
        let inputs = self.read_system_inputs()?;
        
        // Validate inputs and check safety conditions
        self.safety_monitor.validate_inputs(&inputs)?;
        
        // Execute control based on current state
        match self.state {
            SystemState::Idle => {
                // System idle - minimal boost operation
                self.hal.set_duty_cycle(0.0)?;
            },
            
            SystemState::Armed => {
                // Normal operation - execute 3-level control hierarchy
                let duty_cycle = self.execute_control_hierarchy(&inputs)?;
                self.update_output(duty_cycle, &inputs)?;
                
                // Update learning system
                self.learned_data.update_from_operation(&inputs, duty_cycle)?;
            },
            
            SystemState::Calibrating(_) => {
                // Auto-calibration in progress
                let duty_cycle = self.calibration.execute_step(&inputs, &mut self.learned_data)?;
                self.update_output(duty_cycle, &inputs)?;
            },
            
            SystemState::OverboostCut => {
                // Overboost protection active - force 0% duty
                self.hal.set_duty_cycle_immediate(0.0)?;
                
                // Check if we can return to normal operation
                if inputs.manifold_pressure < (self.config.overboost_limit - 0.5) {
                    self.state = SystemState::Armed;
                }
            },
            
            SystemState::Fault(_) => {
                // System fault - maintain failsafe state
                self.hal.set_duty_cycle_immediate(0.0)?;
            },
            
            SystemState::Initializing => {
                // Still initializing - maintain safe state
                self.hal.set_duty_cycle(0.0)?;
            },
        }
        
        // Update performance statistics
        let cycle_time = (self.hal.now_us() - cycle_start) as u32;
        self.update_performance_stats(cycle_time);
        
        Ok(())
    }
    
    /// Read all system inputs from sensors and CAN
    fn read_system_inputs(&mut self) -> Result<SystemInputs, CoreError> {
        // Implementation would read from HAL interfaces
        // This is a placeholder structure
        Ok(SystemInputs {
            rpm: 0,
            desired_torque: 0.0,
            actual_torque: 0.0,
            manifold_pressure: 0.0,
            dome_input_pressure: 0.0,
            upper_dome_pressure: 0.0,
            lower_dome_pressure: 0.0,
            aggression: self.config.aggression,
            scramble_active: false,
            timestamp_ms: self.hal.now_ms(),
        })
    }
    
    /// Execute 3-level control hierarchy
    fn execute_control_hierarchy(&mut self, inputs: &SystemInputs) -> Result<f32, CoreError> {
        // LEVEL 1: Torque-Based Boost Target Adjustment
        let torque_gap = inputs.desired_torque - inputs.actual_torque;
        let assistance_needed = self.torque_following.analyze_assistance_need(torque_gap, inputs)?;
        
        // LEVEL 2: Precise Boost Delivery (PID + Learned Calibration)
        let target_boost = if assistance_needed {
            self.torque_following.calculate_boost_assistance(torque_gap, inputs)?
        } else {
            self.torque_following.get_baseline_boost(inputs)?
        };
        
        // LEVEL 3: Safety and Output
        let target_duty = self.learned_data.boost_to_duty_conversion(target_boost, inputs)?;
        let safe_duty = self.safety_monitor.validate_and_limit(target_duty, inputs)?;
        
        Ok(safe_duty)
    }
    
    /// Update PWM output with safety validation
    fn update_output(&mut self, duty_cycle: f32, inputs: &SystemInputs) -> Result<(), CoreError> {
        // Apply aggression scaling
        let final_duty = self.apply_aggression_scaling(duty_cycle)?;
        
        // Update PWM with timing synchronization
        self.hal.set_duty_cycle_synchronized(final_duty, self.hal.now_us())?;
        
        Ok(())
    }
    
    /// Apply aggression-based scaling to duty cycle
    fn apply_aggression_scaling(&self, base_duty: f32) -> Result<f32, CoreError> {
        // Aggression scales response characteristics
        let response_profile = self.config.get_response_characteristics();
        let scaled_duty = base_duty * response_profile.torque_following_gain;
        
        Ok(scaled_duty.clamp(0.0, 100.0))
    }
    
    /// Update control loop performance statistics
    fn update_performance_stats(&mut self, cycle_time_us: u32) {
        self.stats.avg_cycle_time_us = 
            (self.stats.avg_cycle_time_us * 7 + cycle_time_us) / 8; // Rolling average
        
        if cycle_time_us > self.stats.max_cycle_time_us {
            self.stats.max_cycle_time_us = cycle_time_us;
        }
        
        // Check for timing violations (>10ms for 100Hz loop = timing issue)
        if cycle_time_us > 10_000 {
            self.stats.timing_violations += 1;
        }
        
        self.stats.last_update_ms = self.hal.now_ms();
    }
    
    /// Get current system status for diagnostics
    pub fn get_system_status(&self) -> SystemStatus {
        SystemStatus {
            state: self.state.clone(),
            config: self.config.clone(),
            stats: self.stats.clone(),
            uptime_ms: self.hal.now_ms(),
        }
    }
}

/// System status for diagnostics and monitoring
/// 
/// 🔗 T4-CORE-009: System Status Reporting
/// Derived From: Diagnostic and monitoring requirements
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub state: SystemState,
    pub config: SystemConfig,
    pub stats: ControlLoopStats,
    pub uptime_ms: u32,
}