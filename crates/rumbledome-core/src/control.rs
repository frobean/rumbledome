//! Control system implementation for RumbleDome
//! 
//! Implements the 3-level control hierarchy:
//! Level 1: Torque-Based Boost Target Adjustment
//! Level 2: Precise Boost Delivery (PID + Learned)
//! Level 3: Safety and Output

use crate::config::{SystemConfig, BoostProfile, PidConfig};
use crate::learning::LearnedData;
use crate::error::CoreError;
use rumbledome_hal::{SystemInputs, SensorReadings, CanData};

/// Main control loop coordinator
pub struct ControlLoop {
    /// PID controller for precise boost delivery
    pid_controller: PidController,
    
    /// Torque-based boost target modulator
    torque_modulator: TorqueModulator,
    
    /// Output slew rate limiter for safety
    slew_limiter: SlewLimiter,
    
    /// Current control state
    state: ControlState,
}

/// Current control system state
#[derive(Debug, Clone)]
struct ControlState {
    /// Last executed boost target (PSI)
    last_boost_target: f32,
    
    /// Last executed duty cycle (%)
    last_duty_cycle: f32,
    
    /// Last control loop execution timestamp
    last_execution_ms: u64,
    
    /// Current PID controller state
    pid_state: PidState,
}

/// PID controller implementation
pub struct PidController {
    /// Current PID state
    state: PidState,
    
    /// PID gains (updated per profile)
    gains: PidConfig,
}

/// PID controller internal state
#[derive(Debug, Clone, Default)]
struct PidState {
    /// Previous error term
    previous_error: f32,
    
    /// Integral accumulator
    integral: f32,
    
    /// Last execution timestamp for derivative calculation
    last_time_ms: u64,
}

/// Torque-based boost target modulator
pub struct TorqueModulator {
    /// Target percentage of ECU desired torque to achieve
    torque_target_percentage: f32,
    
    /// Minimum torque gap to trigger boost increase
    torque_gap_threshold: f32,
    
    /// Maximum boost increase per adjustment
    max_boost_increase: f32,
    
    /// Boost reduction factor when approaching torque ceiling
    ceiling_reduction_factor: f32,
}

/// Slew rate limiter for safe duty cycle changes
pub struct SlewLimiter {
    /// Maximum duty cycle change per second (%)
    max_rate_per_second: f32,
    
    /// Last output value
    last_output: f32,
    
    /// Last update timestamp
    last_update_ms: u64,
}

/// Control loop execution result
#[derive(Debug, Clone)]
pub struct ControlResult {
    /// Final duty cycle output (0-100%)
    pub duty_cycle: f32,
    
    /// Target boost pressure used (PSI)
    pub target_boost: f32,
    
    /// Boost error (target - actual)
    pub boost_error: f32,
    
    /// PID controller contribution
    pub pid_output: f32,
    
    /// Learned duty cycle baseline
    pub learned_baseline: f32,
    
    /// Control mode description
    pub control_mode: String,
    
    /// Any warnings or diagnostics
    pub diagnostics: Vec<String>,
}

impl ControlLoop {
    /// Create new control loop instance
    pub fn new() -> Result<Self, CoreError> {
        Ok(Self {
            pid_controller: PidController::new(),
            torque_modulator: TorqueModulator::new(),
            slew_limiter: SlewLimiter::new(5.0), // 5%/second default
            state: ControlState {
                last_boost_target: 0.0,
                last_duty_cycle: 0.0,
                last_execution_ms: 0,
                pid_state: PidState::default(),
            },
        })
    }
    
    /// Execute one control cycle with 3-level hierarchy
    pub fn execute_cycle(
        &mut self,
        inputs: &SystemInputs,
        config: &SystemConfig,
        learned_data: &LearnedData,
        active_profile: &BoostProfile,
    ) -> Result<ControlResult, CoreError> {
        
        let current_time_ms = inputs.timestamp_ms;
        let mut diagnostics = Vec::new();
        
        // Validate input data freshness
        self.validate_input_freshness(inputs, &mut diagnostics)?;
        
        // LEVEL 1: Torque-Based Boost Target Adjustment
        let base_boost_target = active_profile.get_boost_target(inputs.can.rpm);
        let adjusted_boost_target = self.torque_modulator.modulate_boost_for_torque_gap(
            base_boost_target,
            &inputs.can,
            config.torque_target_percentage,
            active_profile.max_boost,
            &mut diagnostics,
        )?;
        
        // LEVEL 2: Precise Boost Delivery (PID + Learned)
        
        // Get learned baseline duty cycle for this boost target
        let learned_baseline_duty = learned_data.lookup_duty_cycle(
            inputs.can.rpm,
            adjusted_boost_target,
            &inputs.sensors,
        )?;
        
        // Update PID gains from profile
        self.pid_controller.update_gains(&active_profile.pid_tuning);
        
        // Calculate boost error (target - actual)
        let boost_error = adjusted_boost_target - inputs.sensors.manifold_pressure_gauge;
        
        // Apply PID correction
        let pid_output = self.pid_controller.update(
            boost_error,
            current_time_ms,
        )?;
        
        // Combine learned baseline with PID correction
        let target_duty = learned_baseline_duty + pid_output;
        
        // LEVEL 3: Safety and Output
        
        // Apply safety constraints (duty cycle limits, profile maximums)
        let constrained_duty = self.apply_safety_constraints(
            target_duty,
            adjusted_boost_target,
            inputs,
            active_profile,
            &mut diagnostics,
        )?;
        
        // Apply slew rate limiting
        let final_duty = self.slew_limiter.limit(
            constrained_duty,
            current_time_ms,
        );
        
        // Update control state
        self.state.last_boost_target = adjusted_boost_target;
        self.state.last_duty_cycle = final_duty;
        self.state.last_execution_ms = current_time_ms;
        self.state.pid_state = self.pid_controller.get_state();
        
        // Determine control mode description
        let control_mode = self.determine_control_mode(
            &inputs.can,
            boost_error,
            config.torque_target_percentage,
        );
        
        Ok(ControlResult {
            duty_cycle: final_duty,
            target_boost: adjusted_boost_target,
            boost_error,
            pid_output,
            learned_baseline: learned_baseline_duty,
            control_mode,
            diagnostics,
        })
    }
    
    /// Validate that input data is fresh and valid
    fn validate_input_freshness(
        &self,
        inputs: &SystemInputs,
        diagnostics: &mut Vec<String>,
    ) -> Result<(), CoreError> {
        
        let current_time = inputs.timestamp_ms;
        
        // Check sensor data freshness (should be < 100ms old)
        let sensor_age = current_time.saturating_sub(inputs.sensors.timestamp_ms);
        if sensor_age > 100 {
            diagnostics.push(format!("Sensor data age: {}ms", sensor_age));
            if sensor_age > 500 {
                return Err(CoreError::input_validation(
                    format!("Sensor data too old: {}ms", sensor_age)
                ));
            }
        }
        
        // Check CAN data freshness (should be < 200ms old)
        let can_age = current_time.saturating_sub(inputs.can.timestamp_ms);
        if can_age > 200 {
            diagnostics.push(format!("CAN data age: {}ms", can_age));
            if can_age > 1000 {
                return Err(CoreError::input_validation(
                    format!("CAN data too old: {}ms", can_age)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Apply safety constraints to duty cycle output
    fn apply_safety_constraints(
        &self,
        target_duty: f32,
        target_boost: f32,
        inputs: &SystemInputs,
        profile: &BoostProfile,
        diagnostics: &mut Vec<String>,
    ) -> Result<f32, CoreError> {
        
        let mut constrained_duty = target_duty;
        
        // Hard limit duty cycle to 0-100% range
        constrained_duty = constrained_duty.clamp(0.0, 100.0);
        
        // Check for overboost condition
        if inputs.sensors.manifold_pressure_gauge > profile.overboost_limit {
            diagnostics.push("Overboost detected - cutting to 0% duty".to_string());
            return Ok(0.0);
        }
        
        // Prevent excessive duty cycle jumps
        let duty_change = (constrained_duty - self.state.last_duty_cycle).abs();
        if duty_change > 10.0 {
            diagnostics.push(format!("Large duty change limited: {:.1}%", duty_change));
            if constrained_duty > self.state.last_duty_cycle {
                constrained_duty = self.state.last_duty_cycle + 10.0;
            } else {
                constrained_duty = self.state.last_duty_cycle - 10.0;
            }
        }
        
        // Ensure we don't exceed profile boost limits with excessive duty
        if target_boost > profile.max_boost && constrained_duty > self.state.last_duty_cycle {
            diagnostics.push("Profile boost limit - preventing duty increase".to_string());
            constrained_duty = self.state.last_duty_cycle;
        }
        
        Ok(constrained_duty.clamp(0.0, 100.0))
    }
    
    /// Determine current control mode for diagnostics
    fn determine_control_mode(
        &self,
        can_data: &CanData,
        boost_error: f32,
        torque_target_percentage: f32,
    ) -> String {
        let torque_error = can_data.desired_torque - can_data.actual_torque;
        let torque_target = can_data.desired_torque * torque_target_percentage / 100.0;
        
        if torque_error > 10.0 {
            "Helping ECU (large torque gap)".to_string()
        } else if can_data.actual_torque > torque_target {
            "Backing off (approaching ceiling)".to_string()
        } else if boost_error.abs() > 0.5 {
            "PID correction".to_string()
        } else {
            "Maintaining target".to_string()
        }
    }
}

impl PidController {
    fn new() -> Self {
        Self {
            state: PidState::default(),
            gains: PidConfig {
                kp: 0.5,
                ki: 0.2,
                kd: 0.0,
                integral_limit: 10.0,
            },
        }
    }
    
    /// Update PID gains from profile configuration
    fn update_gains(&mut self, gains: &PidConfig) {
        self.gains = gains.clone();
    }
    
    /// Execute PID update with boost error
    fn update(&mut self, error: f32, timestamp_ms: u64) -> Result<f32, CoreError> {
        // Calculate time delta
        let dt = if self.state.last_time_ms == 0 {
            0.01 // First iteration - assume 10ms
        } else {
            let dt_ms = timestamp_ms.saturating_sub(self.state.last_time_ms);
            (dt_ms as f32) / 1000.0 // Convert to seconds
        };
        
        if dt <= 0.0 || dt > 0.1 {
            // Invalid time delta - use default
            let dt = 0.01;
        }
        
        // Proportional term
        let proportional = self.gains.kp * error;
        
        // Integral term with windup protection
        self.state.integral += error * dt;
        self.state.integral = self.state.integral.clamp(
            -self.gains.integral_limit,
            self.gains.integral_limit
        );
        let integral = self.gains.ki * self.state.integral;
        
        // Derivative term
        let derivative = if dt > 0.0 {
            self.gains.kd * (error - self.state.previous_error) / dt
        } else {
            0.0
        };
        
        // Update state
        self.state.previous_error = error;
        self.state.last_time_ms = timestamp_ms;
        
        // Combine PID terms
        let output = proportional + integral + derivative;
        
        // Limit PID output to reasonable range (±20% duty cycle)
        Ok(output.clamp(-20.0, 20.0))
    }
    
    /// Get current PID state for diagnostics
    fn get_state(&self) -> PidState {
        self.state.clone()
    }
    
    /// Reset PID state (clear integral, derivative history)
    pub fn reset(&mut self) {
        self.state = PidState::default();
    }
}

impl TorqueModulator {
    fn new() -> Self {
        Self {
            torque_target_percentage: 95.0,
            torque_gap_threshold: 10.0, // Nm
            max_boost_increase: 1.0,    // PSI
            ceiling_reduction_factor: 0.95,
        }
    }
    
    /// Modulate boost target based on ECU torque performance
    fn modulate_boost_for_torque_gap(
        &self,
        base_boost_target: f32,
        can_data: &CanData,
        torque_target_percentage: f32,
        profile_max_boost: f32,
        diagnostics: &mut Vec<String>,
    ) -> Result<f32, CoreError> {
        
        let torque_error = can_data.desired_torque - can_data.actual_torque;
        let torque_target = can_data.desired_torque * torque_target_percentage / 100.0;
        
        let mut adjusted_target = base_boost_target;
        
        if torque_error > self.torque_gap_threshold {
            // Large gap - ECU needs help, increase boost target slightly
            let boost_increase = (torque_error / 50.0).min(self.max_boost_increase);
            adjusted_target = (base_boost_target + boost_increase).min(profile_max_boost);
            
            diagnostics.push(format!(
                "Torque gap {:.1} Nm - boost target +{:.2} PSI",
                torque_error, boost_increase
            ));
            
        } else if can_data.actual_torque > torque_target {
            // Approaching ceiling - back off to prevent ECU intervention
            adjusted_target = base_boost_target * self.ceiling_reduction_factor;
            
            diagnostics.push(format!(
                "Approaching torque ceiling ({:.1}/{:.1} Nm) - reducing boost target",
                can_data.actual_torque, torque_target
            ));
            
        } else {
            // ECU satisfied - maintain current boost target
            diagnostics.push("Torque target satisfied - maintaining boost".to_string());
        }
        
        // Ensure we don't go below zero or above profile maximum
        Ok(adjusted_target.clamp(0.0, profile_max_boost))
    }
}

impl SlewLimiter {
    fn new(max_rate_per_second: f32) -> Self {
        Self {
            max_rate_per_second,
            last_output: 0.0,
            last_update_ms: 0,
        }
    }
    
    /// Apply slew rate limiting to prevent rapid changes
    fn limit(&mut self, target: f32, timestamp_ms: u64) -> f32 {
        if self.last_update_ms == 0 {
            // First call - no limiting
            self.last_output = target;
            self.last_update_ms = timestamp_ms;
            return target;
        }
        
        let dt_ms = timestamp_ms.saturating_sub(self.last_update_ms);
        let dt_seconds = (dt_ms as f32) / 1000.0;
        
        if dt_seconds <= 0.0 || dt_seconds > 0.1 {
            // Invalid time delta - no limiting
            self.last_output = target;
            self.last_update_ms = timestamp_ms;
            return target;
        }
        
        let max_change = self.max_rate_per_second * dt_seconds;
        let change = target - self.last_output;
        
        let limited_change = change.clamp(-max_change, max_change);
        let limited_output = self.last_output + limited_change;
        
        self.last_output = limited_output;
        self.last_update_ms = timestamp_ms;
        
        limited_output
    }
    
    /// Update slew rate limit
    pub fn set_rate(&mut self, rate_per_second: f32) {
        self.max_rate_per_second = rate_per_second;
    }
}