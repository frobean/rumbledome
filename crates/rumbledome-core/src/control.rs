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
// Mock types for testing when teensy41 feature is not available
#[cfg(not(feature = "teensy41"))]
mod mock_pwm_types {
    #[derive(Debug, Clone)]
    pub struct PwmTimingInfo {
        pub frequency_hz: u16,
        pub cycle_period_us: u32,
        pub recommended_control_frequency: u16,
        pub sync_enabled: bool,
        pub update_strategy: ControlUpdateStrategy,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub enum ControlUpdateStrategy {
        Asynchronous,
        CycleStart,
        CycleMidpoint,
        SubCycle { updates_per_cycle: u8 },
    }
}

#[cfg(feature = "teensy41")]
use rumbledome_hal::teensy41::pwm::{PwmTimingInfo, ControlUpdateStrategy};

#[cfg(not(feature = "teensy41"))]
use mock_pwm_types::{PwmTimingInfo, ControlUpdateStrategy};

/// Main control loop coordinator
pub struct ControlLoop {
    /// PID controller for precise boost delivery
    pid_controller: PidController,
    
    /// Torque-based boost target modulator
    torque_modulator: TorqueModulator,
    
    /// Output slew rate limiter for safety
    slew_limiter: SlewLimiter,
    
    /// Overboost recovery manager
    overboost_manager: OverboostManager,
    
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
    
    /// Last PID output for back-calculation anti-windup
    last_output: f32,
    
    /// Output saturation flag for conditional integration
    output_saturated: bool,
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

/// Overboost recovery manager with configurable strategies
pub struct OverboostManager {
    /// Current recovery state
    state: OverboostState,
    
    /// Recovery strategy configuration
    strategy: OverboostRecoveryStrategy,
    
    /// Recovery parameters
    params: OverboostRecoveryParams,
}

/// Overboost recovery state
#[derive(Debug, Clone, PartialEq)]
enum OverboostState {
    Normal,                    // No overboost condition
    Overboost,                 // Currently overboost - emergency action
    Recovery,                  // Post-overboost recovery phase
    Stabilizing,               // Allowing system to stabilize before resuming
}

/// Configurable overboost recovery strategies
#[derive(Debug, Clone)]
pub enum OverboostRecoveryStrategy {
    /// Immediate return to normal control after boost drops
    Immediate,
    
    /// Gradual ramp-up with conservative target reduction
    Conservative {
        /// Target reduction factor during recovery (0.0-1.0)
        target_reduction: f32,
        /// Recovery ramp rate (PSI/second)
        ramp_rate: f32,
        /// Duration to maintain reduced target (ms)
        stabilize_duration_ms: u64,
    },
    
    /// Adaptive recovery based on overboost severity and history
    Adaptive {
        /// Base recovery time (ms)
        base_recovery_ms: u64,
        /// Additional recovery time per PSI of overboost (ms/PSI)
        severity_factor_ms_per_psi: u64,
        /// Recent overboost history weight (0.0-1.0)
        history_weight: f32,
        /// Maximum additional recovery time (ms)
        max_additional_recovery_ms: u64,
    },
    
    /// Profile-based recovery with learned parameters
    ProfileBased {
        /// Reduction factor for each profile aggressiveness level
        aggressiveness_scaling: f32,
        /// Minimum recovery time regardless of profile (ms)
        min_recovery_ms: u64,
        /// Enable learning of optimal recovery parameters
        enable_learning: bool,
    },
}

/// Overboost recovery parameters
#[derive(Debug, Clone)]
struct OverboostRecoveryParams {
    /// Overboost threshold (PSI above global limit)
    overboost_threshold: f32,
    
    /// When to begin recovery (PSI above target)
    recovery_threshold: f32,
    
    /// Timestamp when overboost began
    overboost_start_ms: u64,
    
    /// Peak overboost pressure reached
    peak_overboost_psi: f32,
    
    /// Duration of overboost event (ms)
    overboost_duration_ms: u64,
    
    /// Recovery start timestamp
    recovery_start_ms: u64,
    
    /// Target boost during recovery phase
    recovery_target_boost: f32,
    
    /// Recent overboost event history for adaptive strategies
    recent_events: Vec<OverboostEvent>,
}

/// Historical overboost event for adaptive recovery
#[derive(Debug, Clone)]
struct OverboostEvent {
    /// Timestamp of event
    timestamp_ms: u64,
    
    /// Peak pressure reached (PSI)
    peak_pressure: f32,
    
    /// Duration of overboost (ms)
    duration_ms: u64,
    
    /// Recovery time required (ms)
    recovery_time_ms: u64,
    
    /// Active profile during event
    profile_name: String,
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
            overboost_manager: OverboostManager::new(),
            state: ControlState {
                last_boost_target: 0.0,
                last_duty_cycle: 0.0,
                last_execution_ms: 0,
                pid_state: PidState::default(),
            },
        })
    }
    
    /// Set overboost recovery strategy
    pub fn set_overboost_strategy(&mut self, strategy: OverboostRecoveryStrategy) {
        self.overboost_manager.set_strategy(strategy);
    }
    
    /// Execute one control cycle with 3-level hierarchy and PWM synchronization
    pub fn execute_cycle(
        &mut self,
        inputs: &SystemInputs,
        config: &SystemConfig,
        learned_data: &LearnedData,
        active_profile: &BoostProfile,
        pwm_timing: Option<&PwmTimingInfo>,
    ) -> Result<ControlResult, CoreError> {
        
        let current_time_ms = inputs.timestamp_ms;
        let current_time_us = current_time_ms * 1000; // Convert to microseconds for PWM sync
        let mut diagnostics = Vec::new();
        
        // Check PWM synchronization timing if available
        if let Some(timing) = pwm_timing {
            self.validate_pwm_timing(timing, current_time_us, &mut diagnostics)?;
        }
        
        // Validate input data freshness
        self.validate_input_freshness(inputs, &mut diagnostics)?;
        
        // LEVEL 1: Torque-Based Boost Target Adjustment with Overboost Recovery
        let base_boost_target = active_profile.get_boost_target(inputs.can.rpm);
        
        // Check for overboost and handle recovery
        let (recovery_adjusted_target, overboost_active) = self.overboost_manager.process_overboost(
            base_boost_target,
            inputs.sensors.manifold_pressure_gauge,
            active_profile,
            current_time_ms,
            &mut diagnostics,
        )?;
        
        // Apply torque modulation only if not in overboost recovery
        let adjusted_boost_target = if overboost_active {
            recovery_adjusted_target
        } else {
            self.torque_modulator.modulate_boost_for_torque_gap(
                recovery_adjusted_target,
                &inputs.can,
                config.torque_target_percentage,
                active_profile.max_boost,
                &mut diagnostics,
            )?
        };
        
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
        
        // Apply PID correction with anti-windup
        let pid_output = self.pid_controller.update_with_feedback(
            boost_error,
            current_time_ms,
            learned_baseline_duty, // Baseline for back-calculation
            overboost_active,      // Disable integral when in overboost
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
        
        // Apply slew rate limiting with PWM synchronization
        let final_duty = self.slew_limiter.limit_with_sync(
            constrained_duty,
            current_time_ms,
            pwm_timing,
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
        
        // Add PWM synchronization status to diagnostics
        if let Some(timing) = pwm_timing {
            if timing.sync_enabled {
                diagnostics.push(format!(
                    "PWM sync: {:?} @ {}Hz", 
                    timing.update_strategy, timing.frequency_hz
                ));
            }
        }
        
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
    
    /// Validate PWM timing coordination for jitter reduction
    fn validate_pwm_timing(
        &self,
        timing: &PwmTimingInfo,
        current_time_us: u64,
        diagnostics: &mut Vec<String>,
    ) -> Result<(), CoreError> {
        
        if !timing.sync_enabled {
            return Ok(()); // No validation needed if sync disabled
        }
        
        // Check if control loop frequency matches PWM timing recommendations
        let control_period_ms = if self.state.last_execution_ms > 0 {
            current_time_us / 1000 - self.state.last_execution_ms
        } else {
            10 // Assume 100Hz default
        };
        
        let actual_control_freq = if control_period_ms > 0 {
            1000 / control_period_ms as u16
        } else {
            100
        };
        
        let recommended_freq = timing.recommended_control_frequency;
        let freq_deviation = ((actual_control_freq as f32 - recommended_freq as f32) / recommended_freq as f32).abs();
        
        if freq_deviation > 0.1 { // More than 10% deviation
            diagnostics.push(format!(
                "Control frequency deviation: actual {}Hz vs recommended {}Hz",
                actual_control_freq, recommended_freq
            ));
        }
        
        // Check for potential beat frequency issues
        let pwm_freq = timing.frequency_hz as f32;
        let control_freq = actual_control_freq as f32;
        
        // Avoid beat frequencies: control freq should not be close to PWM freq harmonics
        let harmonics_to_check = [1.0, 2.0, 3.0, 0.5, 0.33, 0.25];
        for &harmonic in &harmonics_to_check {
            let harmonic_freq = pwm_freq * harmonic;
            let beat_freq = (control_freq - harmonic_freq).abs();
            
            if beat_freq < 5.0 && beat_freq > 0.1 { // Beat frequency between 0.1-5Hz is problematic
                diagnostics.push(format!(
                    "Potential beat frequency: {:.1}Hz between control ({:.1}Hz) and PWM harmonic ({:.1}Hz)",
                    beat_freq, control_freq, harmonic_freq
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
    
    /// Execute PID update with boost error (deprecated - use update_with_feedback)
    fn update(&mut self, error: f32, timestamp_ms: u64) -> Result<f32, CoreError> {
        self.update_with_feedback(error, timestamp_ms, 0.0, false)
    }
    
    /// Execute PID update with advanced anti-windup and feedback
    fn update_with_feedback(
        &mut self, 
        error: f32, 
        timestamp_ms: u64, 
        baseline_output: f32,
        disable_integral: bool
    ) -> Result<f32, CoreError> {
        // Calculate time delta
        let dt = if self.state.last_time_ms == 0 {
            0.01 // First iteration - assume 10ms
        } else {
            let dt_ms = timestamp_ms.saturating_sub(self.state.last_time_ms);
            (dt_ms as f32) / 1000.0 // Convert to seconds
        };
        
        let dt = if dt <= 0.0 || dt > 0.1 { 0.01 } else { dt };
        
        // Proportional term
        let proportional = self.gains.kp * error;
        
        // Derivative term (calculated before updating previous_error)
        let derivative = if dt > 0.0 && self.state.last_time_ms > 0 {
            self.gains.kd * (error - self.state.previous_error) / dt
        } else {
            0.0
        };
        
        // Calculate preliminary output (P + D terms only)
        let pd_output = proportional + derivative;
        let preliminary_total = baseline_output + pd_output + (self.gains.ki * self.state.integral);
        
        // Check if output would saturate (PID output limits ±20% duty cycle)
        let output_limit = 20.0;
        let would_saturate = preliminary_total.abs() > output_limit;
        
        // Conditional integration - only integrate if:
        // 1. Not disabled by overboost condition
        // 2. Output is not saturated, OR error and integral have opposite signs (helping to reduce saturation)
        let should_integrate = !disable_integral && 
            (!would_saturate || (error * self.state.integral <= 0.0));
        
        if should_integrate {
            // Normal integration
            self.state.integral += error * dt;
        } else {
            // Back-calculation anti-windup: adjust integral to prevent windup
            if would_saturate {
                let saturated_output = preliminary_total.clamp(-output_limit, output_limit);
                let output_error = saturated_output - preliminary_total;
                
                // Back-calculate what integral should be to achieve saturated output
                if self.gains.ki != 0.0 {
                    let desired_integral = (saturated_output - baseline_output - pd_output) / self.gains.ki;
                    
                    // Apply back-calculation with tracking time constant
                    let tracking_gain = 1.0 / self.gains.ki; // Tracking time constant
                    self.state.integral += tracking_gain * output_error * dt;
                    
                    // Also limit integral to desired value to prevent further windup
                    self.state.integral = self.state.integral.clamp(
                        desired_integral - 1.0,
                        desired_integral + 1.0
                    );
                }
            }
        }
        
        // Apply integral limits as final safety
        self.state.integral = self.state.integral.clamp(
            -self.gains.integral_limit,
            self.gains.integral_limit
        );
        
        let integral = self.gains.ki * self.state.integral;
        
        // Combine PID terms
        let raw_output = proportional + integral + derivative;
        let limited_output = raw_output.clamp(-output_limit, output_limit);
        
        // Update state for next iteration
        self.state.previous_error = error;
        self.state.last_time_ms = timestamp_ms;
        self.state.last_output = limited_output;
        self.state.output_saturated = limited_output != raw_output;
        
        Ok(limited_output)
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
        self.limit_with_sync(target, timestamp_ms, None)
    }
    
    /// Apply slew rate limiting with PWM synchronization and jitter reduction
    fn limit_with_sync(&mut self, target: f32, timestamp_ms: u64, pwm_timing: Option<&PwmTimingInfo>) -> f32 {
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
        
        // Apply PWM-aware slew rate limiting
        let effective_slew_rate = if let Some(timing) = pwm_timing {
            self.calculate_pwm_aware_slew_rate(timing, dt_seconds)
        } else {
            self.max_rate_per_second
        };
        
        let max_change = effective_slew_rate * dt_seconds;
        let change = target - self.last_output;
        
        // Apply jitter reduction for small changes
        let change_with_jitter_reduction = self.apply_jitter_reduction(change, pwm_timing);
        
        let limited_change = change_with_jitter_reduction.clamp(-max_change, max_change);
        let limited_output = self.last_output + limited_change;
        
        self.last_output = limited_output;
        self.last_update_ms = timestamp_ms;
        
        limited_output
    }
    
    /// Calculate PWM-aware slew rate to prevent phase noise
    fn calculate_pwm_aware_slew_rate(&self, timing: &PwmTimingInfo, dt_seconds: f32) -> f32 {
        let base_rate = self.max_rate_per_second;
        
        match timing.update_strategy {
            ControlUpdateStrategy::Asynchronous => base_rate,
            
            ControlUpdateStrategy::CycleStart | ControlUpdateStrategy::CycleMidpoint => {
                // Synchronous updates can be more aggressive since they avoid switching noise
                base_rate * 1.5
            },
            
            ControlUpdateStrategy::SubCycle { updates_per_cycle } => {
                // Multiple updates per cycle allow finer control
                base_rate * (updates_per_cycle as f32).sqrt()
            },
        }
    }
    
    /// Apply jitter reduction to small duty cycle changes
    fn apply_jitter_reduction(&self, change: f32, pwm_timing: Option<&PwmTimingInfo>) -> f32 {
        let jitter_threshold = if let Some(timing) = pwm_timing {
            // PWM resolution-based threshold: don't make changes smaller than 1 PWM count
            let pwm_resolution = 100.0 / 32767.0; // FlexPWM has 15-bit resolution
            match timing.update_strategy {
                ControlUpdateStrategy::CycleMidpoint => pwm_resolution * 2.0, // More aggressive filtering
                _ => pwm_resolution,
            }
        } else {
            0.1 // Default threshold: 0.1% duty cycle
        };
        
        // Apply deadband to reduce jitter
        if change.abs() < jitter_threshold {
            0.0 // Filter out small changes that would cause jitter
        } else {
            change
        }
    }
    
    /// Update slew rate limit
    pub fn set_rate(&mut self, rate_per_second: f32) {
        self.max_rate_per_second = rate_per_second;
    }
}

impl OverboostManager {
    fn new() -> Self {
        Self {
            state: OverboostState::Normal,
            strategy: OverboostRecoveryStrategy::Conservative {
                target_reduction: 0.8,  // Reduce to 80% of target during recovery
                ramp_rate: 2.0,         // 2 PSI/second recovery rate
                stabilize_duration_ms: 3000, // 3 second stabilization
            },
            params: OverboostRecoveryParams {
                overboost_threshold: 0.5,  // 0.5 PSI above global limit
                recovery_threshold: 0.2,   // 0.2 PSI above target to start recovery
                overboost_start_ms: 0,
                peak_overboost_psi: 0.0,
                overboost_duration_ms: 0,
                recovery_start_ms: 0,
                recovery_target_boost: 0.0,
                recent_events: Vec::new(),
            },
        }
    }
    
    /// Set recovery strategy
    fn set_strategy(&mut self, strategy: OverboostRecoveryStrategy) {
        self.strategy = strategy;
    }
    
    /// Process overboost detection and recovery
    fn process_overboost(
        &mut self,
        target_boost: f32,
        current_boost: f32,
        active_profile: &BoostProfile,
        timestamp_ms: u64,
        diagnostics: &mut Vec<String>,
    ) -> Result<(f32, bool), CoreError> {
        
        let global_overboost_limit = active_profile.overboost_limit;
        let is_overboost = current_boost > (global_overboost_limit + self.params.overboost_threshold);
        let is_above_recovery_threshold = current_boost > (target_boost + self.params.recovery_threshold);
        
        match self.state {
            OverboostState::Normal => {
                if is_overboost {
                    // Begin overboost event
                    self.state = OverboostState::Overboost;
                    self.params.overboost_start_ms = timestamp_ms;
                    self.params.peak_overboost_psi = current_boost;
                    
                    diagnostics.push(format!(
                        "OVERBOOST DETECTED: {:.2} PSI (limit: {:.2} PSI) - EMERGENCY STOP",
                        current_boost, global_overboost_limit
                    ));
                    
                    // Emergency action: return 0% duty (wastegate fully open)
                    return Ok((0.0, true));
                }
                
                // Normal operation
                Ok((target_boost, false))
            },
            
            OverboostState::Overboost => {
                // Update peak pressure
                self.params.peak_overboost_psi = self.params.peak_overboost_psi.max(current_boost);
                
                if !is_overboost {
                    // Boost has dropped below critical threshold - begin recovery
                    self.state = OverboostState::Recovery;
                    self.params.recovery_start_ms = timestamp_ms;
                    self.params.overboost_duration_ms = timestamp_ms - self.params.overboost_start_ms;
                    
                    // Calculate recovery target based on strategy
                    self.params.recovery_target_boost = self.calculate_recovery_target(
                        target_boost,
                        active_profile,
                        diagnostics,
                    );
                    
                    diagnostics.push(format!(
                        "Overboost recovery started - target reduced to {:.2} PSI",
                        self.params.recovery_target_boost
                    ));
                } else {
                    // Still in overboost - maintain emergency action
                    diagnostics.push(format!(
                        "OVERBOOST ACTIVE: {:.2} PSI - maintaining emergency stop",
                        current_boost
                    ));
                }
                
                // Emergency action continues
                Ok((0.0, true))
            },
            
            OverboostState::Recovery => {
                let recovery_progress = self.calculate_recovery_progress(
                    target_boost,
                    timestamp_ms,
                    active_profile,
                    diagnostics,
                );
                
                if recovery_progress >= 1.0 && !is_above_recovery_threshold {
                    // Recovery complete - move to stabilization
                    self.state = OverboostState::Stabilizing;
                    
                    diagnostics.push("Overboost recovery complete - entering stabilization".to_string());
                    
                    Ok((target_boost, false))
                } else {
                    // Continue recovery with adjusted target
                    let adjusted_target = self.params.recovery_target_boost + 
                        (target_boost - self.params.recovery_target_boost) * recovery_progress;
                    
                    diagnostics.push(format!(
                        "Overboost recovery: {:.1}% complete, target: {:.2} PSI",
                        recovery_progress * 100.0, adjusted_target
                    ));
                    
                    Ok((adjusted_target, true))
                }
            },
            
            OverboostState::Stabilizing => {
                // Check if stabilization period is complete
                let stabilization_time = match &self.strategy {
                    OverboostRecoveryStrategy::Conservative { stabilize_duration_ms, .. } => *stabilize_duration_ms,
                    OverboostRecoveryStrategy::Adaptive { base_recovery_ms, .. } => *base_recovery_ms / 2,
                    OverboostRecoveryStrategy::ProfileBased { min_recovery_ms, .. } => *min_recovery_ms,
                    OverboostRecoveryStrategy::Immediate => 0,
                };
                
                if timestamp_ms - self.params.recovery_start_ms > stabilization_time {
                    // Stabilization complete - return to normal
                    self.record_overboost_event(timestamp_ms, active_profile);
                    self.state = OverboostState::Normal;
                    
                    diagnostics.push("Overboost event complete - returning to normal operation".to_string());
                    
                    Ok((target_boost, false))
                } else {
                    // Continue stabilization
                    diagnostics.push("Stabilizing after overboost recovery".to_string());
                    Ok((target_boost, true))
                }
            },
        }
    }
    
    /// Calculate recovery target based on strategy
    fn calculate_recovery_target(
        &self,
        normal_target: f32,
        profile: &BoostProfile,
        diagnostics: &mut Vec<String>,
    ) -> f32 {
        match &self.strategy {
            OverboostRecoveryStrategy::Immediate => {
                diagnostics.push("Using immediate recovery strategy".to_string());
                normal_target
            },
            
            OverboostRecoveryStrategy::Conservative { target_reduction, .. } => {
                let reduced_target = normal_target * target_reduction;
                diagnostics.push(format!(
                    "Conservative recovery: reducing target by {:.1}%",
                    (1.0 - target_reduction) * 100.0
                ));
                reduced_target
            },
            
            OverboostRecoveryStrategy::Adaptive { base_recovery_ms, severity_factor_ms_per_psi, .. } => {
                let severity = self.params.peak_overboost_psi - profile.overboost_limit;
                let severity_factor = 1.0 - (severity * 0.1).min(0.5); // Up to 50% reduction
                let adaptive_target = normal_target * severity_factor;
                
                diagnostics.push(format!(
                    "Adaptive recovery: severity {:.2} PSI, reduction factor {:.2}",
                    severity, severity_factor
                ));
                adaptive_target
            },
            
            OverboostRecoveryStrategy::ProfileBased { aggressiveness_scaling, .. } => {
                // Scale based on profile aggressiveness (0.0-1.0)
                let profile_aggressiveness = profile.aggressiveness.unwrap_or(0.5);
                let scaling = 1.0 - (profile_aggressiveness * aggressiveness_scaling);
                let scaled_target = normal_target * scaling.clamp(0.5, 1.0);
                
                diagnostics.push(format!(
                    "Profile-based recovery: aggressiveness {:.2}, scaling {:.2}",
                    profile_aggressiveness, scaling
                ));
                scaled_target
            },
        }
    }
    
    /// Calculate recovery progress (0.0 to 1.0)
    fn calculate_recovery_progress(
        &self,
        target_boost: f32,
        timestamp_ms: u64,
        profile: &BoostProfile,
        diagnostics: &mut Vec<String>,
    ) -> f32 {
        let recovery_time = timestamp_ms - self.params.recovery_start_ms;
        
        match &self.strategy {
            OverboostRecoveryStrategy::Immediate => 1.0,
            
            OverboostRecoveryStrategy::Conservative { ramp_rate, .. } => {
                let target_difference = target_boost - self.params.recovery_target_boost;
                let time_needed_ms = if *ramp_rate > 0.0 {
                    (target_difference / ramp_rate * 1000.0) as u64
                } else {
                    3000 // Default 3 seconds
                };
                
                (recovery_time as f32 / time_needed_ms as f32).clamp(0.0, 1.0)
            },
            
            OverboostRecoveryStrategy::Adaptive { base_recovery_ms, severity_factor_ms_per_psi, .. } => {
                let severity = self.params.peak_overboost_psi - profile.overboost_limit;
                let total_recovery_time = base_recovery_ms + 
                    (severity * (*severity_factor_ms_per_psi as f32)) as u64;
                
                (recovery_time as f32 / total_recovery_time as f32).clamp(0.0, 1.0)
            },
            
            OverboostRecoveryStrategy::ProfileBased { min_recovery_ms, .. } => {
                let profile_recovery_time = (*min_recovery_ms as f32 * 
                    (1.0 + profile.aggressiveness.unwrap_or(0.5))) as u64;
                
                (recovery_time as f32 / profile_recovery_time as f32).clamp(0.0, 1.0)
            },
        }
    }
    
    /// Record overboost event for learning and statistics
    fn record_overboost_event(&mut self, timestamp_ms: u64, profile: &BoostProfile) {
        let event = OverboostEvent {
            timestamp_ms: self.params.overboost_start_ms,
            peak_pressure: self.params.peak_overboost_psi,
            duration_ms: self.params.overboost_duration_ms,
            recovery_time_ms: timestamp_ms - self.params.recovery_start_ms,
            profile_name: profile.name.clone().unwrap_or("Unknown".to_string()),
        };
        
        self.params.recent_events.push(event);
        
        // Keep only last 10 events
        if self.params.recent_events.len() > 10 {
            self.params.recent_events.remove(0);
        }
    }
    
    /// Get current overboost state for diagnostics
    pub fn get_state(&self) -> &OverboostState {
        &self.state
    }
    
    /// Get recent overboost events for analysis
    pub fn get_recent_events(&self) -> &[OverboostEvent] {
        &self.params.recent_events
    }
}