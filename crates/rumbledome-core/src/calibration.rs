//! Auto-calibration system for RumbleDome

use crate::config::{SystemConfig, BoostProfile};
use crate::learning::LearnedData;
use crate::error::CoreError;
use crate::state::CalibrationState;
use rumbledome_hal::{SystemInputs, PwmChannel};
use serde::{Deserialize, Serialize};

/// Auto-calibration system implementation
pub struct AutoCalibration {
    /// Current calibration session state
    session_state: Option<CalibrationSession>,
    
    /// Calibration parameters
    parameters: CalibrationParameters,
    
    /// Safety limits for calibration
    safety_limits: CalibrationSafetyLimits,
}

/// Active calibration session data
#[derive(Debug, Clone)]
struct CalibrationSession {
    /// Target RPM for calibration
    target_rpm: u16,
    
    /// Target boost pressure (PSI)
    target_boost: f32,
    
    /// Current phase of calibration
    phase: CalibrationPhase,
    
    /// Run attempt counter
    run_number: u8,
    
    /// Current duty cycle being tested
    current_test_duty: f32,
    
    /// Results from current run
    current_run_results: Vec<CalibrationSample>,
    
    /// Session start timestamp
    session_start_ms: u64,
    
    /// Current run start timestamp
    run_start_ms: u64,
}

/// Calibration phase progression
#[derive(Debug, Clone, PartialEq)]
enum CalibrationPhase {
    /// Initial conservative testing
    Conservative,
    
    /// Progressive limit expansion
    Progressive,
    
    /// Fine-tuning duty cycle
    FineTuning,
    
    /// Validation runs
    Validation,
}

/// Individual calibration sample
#[derive(Debug, Clone)]
struct CalibrationSample {
    /// Sample timestamp
    timestamp_ms: u64,
    
    /// Engine RPM at sample
    rpm: u16,
    
    /// Measured boost pressure
    boost_psi: f32,
    
    /// Duty cycle at sample
    duty_cycle: f32,
    
    /// Sample validity (within target ranges)
    valid: bool,
}

/// Calibration system parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationParameters {
    /// Duty cycle step size for testing
    pub duty_step_size: f32,
    
    /// Maximum duty cycle for initial testing
    pub max_initial_duty: f32,
    
    /// RPM tolerance for valid samples
    pub rpm_tolerance: u16,
    
    /// Boost pressure tolerance for success
    pub boost_tolerance: f32,
    
    /// Minimum samples required per run
    pub min_samples_per_run: u32,
    
    /// Maximum run duration (ms)
    pub max_run_duration_ms: u64,
    
    /// Minimum time at steady state
    pub min_steady_state_ms: u64,
}

/// Safety limits during calibration
#[derive(Debug, Clone)]
struct CalibrationSafetyLimits {
    /// Maximum overboost during calibration
    max_overboost: f32,
    
    /// Progressive overboost limits
    progressive_limits: Vec<f32>,
    
    /// Current limit index
    current_limit_index: usize,
    
    /// Required successful runs before limit increase
    runs_before_progression: u8,
    
    /// Successful runs at current limit
    successful_runs_at_limit: u8,
}

/// Calibration run result
#[derive(Debug, Clone)]
pub struct CalibrationResult {
    /// Success/failure status
    pub success: bool,
    
    /// Learned duty cycle (if successful)
    pub learned_duty: Option<f32>,
    
    /// Achieved boost accuracy
    pub boost_accuracy: f32,
    
    /// Number of valid samples
    pub valid_samples: u32,
    
    /// Run duration
    pub duration_ms: u64,
    
    /// Failure reason (if applicable)
    pub failure_reason: Option<String>,
    
    /// Next recommended action
    pub next_action: CalibrationAction,
}

/// Recommended next calibration action
#[derive(Debug, Clone, PartialEq)]
pub enum CalibrationAction {
    /// Continue with next run
    ContinueRun,
    
    /// Advance to next phase
    AdvancePhase,
    
    /// Increase progressive limits
    IncreaseLimit,
    
    /// Complete calibration successfully
    Complete,
    
    /// Abort calibration due to failure
    Abort { reason: String },
    
    /// Retry current run
    RetryRun,
}

impl AutoCalibration {
    /// Create new auto-calibration system
    pub fn new() -> Self {
        Self {
            session_state: None,
            parameters: CalibrationParameters::default(),
            safety_limits: CalibrationSafetyLimits::new(5.0), // Default spring pressure
        }
    }
    
    /// Start new calibration session
    pub fn start_session(&mut self, target_rpm: u16, target_boost: f32) -> Result<(), CoreError> {
        if self.session_state.is_some() {
            return Err(CoreError::calibration("Calibration session already active"));
        }
        
        // Validate calibration target
        if target_rpm < 1000 || target_rpm > 7000 {
            return Err(CoreError::calibration(format!("Invalid RPM target: {}", target_rpm)));
        }
        
        if target_boost < 0.0 || target_boost > 15.0 {
            return Err(CoreError::calibration(format!("Invalid boost target: {} PSI", target_boost)));
        }
        
        // Initialize calibration session
        let session = CalibrationSession {
            target_rpm,
            target_boost,
            phase: CalibrationPhase::Conservative,
            run_number: 1,
            current_test_duty: 5.0, // Start with conservative duty cycle
            current_run_results: Vec::new(),
            session_start_ms: 0, // Will be set on first update
            run_start_ms: 0,
        };
        
        self.session_state = Some(session);
        
        log::info!("Started calibration session: {} RPM, {} PSI target", target_rpm, target_boost);
        Ok(())
    }
    
    /// Process calibration cycle and return duty cycle command
    pub fn process_calibration_cycle(
        &mut self,
        inputs: &SystemInputs,
        _config: &SystemConfig,
        learned_data: &mut LearnedData,
    ) -> Result<CalibrationResult, CoreError> {
        
        let session = self.session_state.as_mut()
            .ok_or_else(|| CoreError::calibration("No active calibration session"))?;
        
        // Initialize timestamps on first call
        if session.session_start_ms == 0 {
            session.session_start_ms = inputs.timestamp_ms;
            session.run_start_ms = inputs.timestamp_ms;
        }
        
        // Check safety limits
        self.check_calibration_safety(inputs, session)?;
        
        // Collect calibration sample
        let sample = CalibrationSample {
            timestamp_ms: inputs.timestamp_ms,
            rpm: inputs.can.rpm,
            boost_psi: inputs.sensors.manifold_pressure_gauge,
            duty_cycle: session.current_test_duty,
            valid: self.is_sample_valid(inputs, session),
        };
        
        session.current_run_results.push(sample);
        
        // Check if run is complete
        let run_duration = inputs.timestamp_ms.saturating_sub(session.run_start_ms);
        
        if self.is_run_complete(session, run_duration) {
            let result = self.process_run_completion(session, learned_data, inputs.timestamp_ms)?;
            
            if matches!(result.next_action, CalibrationAction::Complete | CalibrationAction::Abort { .. }) {
                self.session_state = None; // End session
            }
            
            return Ok(result);
        }
        
        // Continue current run
        Ok(CalibrationResult {
            success: false, // Run still in progress
            learned_duty: None,
            boost_accuracy: 0.0,
            valid_samples: session.current_run_results.iter()
                .filter(|s| s.valid)
                .count() as u32,
            duration_ms: run_duration,
            failure_reason: None,
            next_action: CalibrationAction::ContinueRun,
        })
    }
    
    /// Get current calibration duty cycle command
    pub fn get_current_duty_cycle(&self) -> f32 {
        self.session_state
            .as_ref()
            .map(|s| s.current_test_duty)
            .unwrap_or(0.0)
    }
    
    /// Check calibration safety limits
    fn check_calibration_safety(
        &self,
        inputs: &SystemInputs,
        session: &CalibrationSession,
    ) -> Result<(), CoreError> {
        
        let current_limit = self.safety_limits.get_current_limit();
        
        if inputs.sensors.manifold_pressure_gauge > current_limit {
            return Err(CoreError::safety(format!(
                "Calibration overboost: {:.1} > {:.1} PSI",
                inputs.sensors.manifold_pressure_gauge, current_limit
            )));
        }
        
        Ok(())
    }
    
    /// Check if calibration sample is valid
    fn is_sample_valid(&self, inputs: &SystemInputs, session: &CalibrationSession) -> bool {
        // Check RPM is within tolerance
        let rpm_error = (inputs.can.rpm as i32 - session.target_rpm as i32).abs();
        if rpm_error > self.parameters.rpm_tolerance as i32 {
            return false;
        }
        
        // Check engine is under load (throttle position if available)
        if let Some(throttle) = inputs.can.throttle_position {
            if throttle < 80.0 {
                return false; // Not WOT
            }
        }
        
        // Check boost is reasonable (not negative, not excessive)
        if inputs.sensors.manifold_pressure_gauge < -2.0 || 
           inputs.sensors.manifold_pressure_gauge > session.target_boost + 3.0 {
            return false;
        }
        
        true
    }
    
    /// Check if current calibration run is complete
    fn is_run_complete(&self, session: &CalibrationSession, run_duration: u64) -> bool {
        let valid_samples = session.current_run_results.iter()
            .filter(|s| s.valid)
            .count() as u32;
        
        // Complete if we have enough samples or exceeded time limit
        valid_samples >= self.parameters.min_samples_per_run ||
        run_duration >= self.parameters.max_run_duration_ms
    }
    
    /// Process completion of calibration run
    fn process_run_completion(
        &mut self,
        session: &mut CalibrationSession,
        learned_data: &mut LearnedData,
        timestamp: u64,
    ) -> Result<CalibrationResult, CoreError> {
        
        let valid_samples: Vec<&CalibrationSample> = session.current_run_results
            .iter()
            .filter(|s| s.valid)
            .collect();
        
        let run_duration = timestamp.saturating_sub(session.run_start_ms);
        
        if valid_samples.is_empty() {
            return Ok(CalibrationResult {
                success: false,
                learned_duty: None,
                boost_accuracy: 0.0,
                valid_samples: 0,
                duration_ms: run_duration,
                failure_reason: Some("No valid samples collected".to_string()),
                next_action: if session.run_number < 3 {
                    CalibrationAction::RetryRun
                } else {
                    CalibrationAction::Abort { 
                        reason: "Multiple failed runs".to_string() 
                    }
                },
            });
        }
        
        // Calculate average boost achieved
        let avg_boost: f32 = valid_samples.iter()
            .map(|s| s.boost_psi)
            .sum::<f32>() / valid_samples.len() as f32;
        
        let boost_error = (avg_boost - session.target_boost).abs();
        
        // Check if run was successful
        let success = boost_error <= self.parameters.boost_tolerance;
        
        let mut next_action = CalibrationAction::ContinueRun;
        let mut learned_duty = None;
        
        if success {
            // Learn this duty cycle mapping
            learned_duty = Some(session.current_test_duty);
            
            learned_data.update_calibration_point(
                session.target_rpm,
                session.target_boost,
                avg_boost,
                session.current_test_duty,
                timestamp,
            )?;
            
            self.safety_limits.successful_runs_at_limit += 1;
            
            // Check if we should advance
            match session.phase {
                CalibrationPhase::Conservative => {
                    if session.run_number >= 3 {
                        next_action = CalibrationAction::AdvancePhase;
                    }
                },
                CalibrationPhase::Progressive => {
                    if self.safety_limits.successful_runs_at_limit >= 
                       self.safety_limits.runs_before_progression {
                        next_action = CalibrationAction::IncreaseLimit;
                    }
                },
                CalibrationPhase::FineTuning => {
                    next_action = CalibrationAction::AdvancePhase;
                },
                CalibrationPhase::Validation => {
                    next_action = CalibrationAction::Complete;
                },
            }
        } else {
            // Adjust duty cycle for next run
            if avg_boost < session.target_boost - self.parameters.boost_tolerance {
                // Need more boost - increase duty cycle
                session.current_test_duty += self.parameters.duty_step_size;
            } else if avg_boost > session.target_boost + self.parameters.boost_tolerance {
                // Too much boost - decrease duty cycle
                session.current_test_duty -= self.parameters.duty_step_size;
            }
            
            // Prevent excessive duty cycles
            session.current_test_duty = session.current_test_duty.clamp(0.0, 
                self.parameters.max_initial_duty);
        }
        
        // Prepare for next run
        session.run_number += 1;
        session.current_run_results.clear();
        session.run_start_ms = timestamp;
        
        Ok(CalibrationResult {
            success,
            learned_duty,
            boost_accuracy: boost_error,
            valid_samples: valid_samples.len() as u32,
            duration_ms: run_duration,
            failure_reason: if success { None } else { 
                Some(format!("Boost error: {:.2} PSI", boost_error)) 
            },
            next_action,
        })
    }
    
    /// Stop current calibration session
    pub fn stop_session(&mut self) -> bool {
        let was_active = self.session_state.is_some();
        self.session_state = None;
        was_active
    }
    
    /// Get current calibration state for UI
    pub fn get_calibration_state(&self) -> Option<CalibrationState> {
        self.session_state.as_ref().map(|session| {
            match session.phase {
                CalibrationPhase::Conservative => CalibrationState::Conservative {
                    target_rpm: session.target_rpm,
                    target_boost: session.target_boost,
                    runs_completed: session.run_number.saturating_sub(1),
                },
                CalibrationPhase::Progressive => CalibrationState::Progressive {
                    current_limit: self.safety_limits.get_current_limit(),
                    confidence: 0.8, // TODO: Calculate from learned data
                    target_rpm: session.target_rpm,
                    target_boost: session.target_boost,
                },
                CalibrationPhase::FineTuning | CalibrationPhase::Validation => {
                    CalibrationState::Complete
                },
            }
        })
    }
}

impl CalibrationSafetyLimits {
    fn new(spring_pressure: f32) -> Self {
        // Progressive limits: spring+1, spring+2, spring+3, etc.
        let progressive_limits = vec![
            spring_pressure + 1.0,
            spring_pressure + 2.0,  
            spring_pressure + 3.0,
            spring_pressure + 5.0,
            spring_pressure + 7.0,
        ];
        
        Self {
            max_overboost: spring_pressure + 10.0,
            progressive_limits,
            current_limit_index: 0,
            runs_before_progression: 3,
            successful_runs_at_limit: 0,
        }
    }
    
    fn get_current_limit(&self) -> f32 {
        self.progressive_limits
            .get(self.current_limit_index)
            .copied()
            .unwrap_or(self.max_overboost)
    }
    
    fn advance_limit(&mut self) -> bool {
        if self.current_limit_index + 1 < self.progressive_limits.len() {
            self.current_limit_index += 1;
            self.successful_runs_at_limit = 0;
            true
        } else {
            false
        }
    }
}

impl Default for CalibrationParameters {
    fn default() -> Self {
        Self {
            duty_step_size: 2.0,      // 2% duty cycle steps
            max_initial_duty: 30.0,   // Conservative maximum
            rpm_tolerance: 200,       // ±200 RPM
            boost_tolerance: 0.5,     // ±0.5 PSI
            min_samples_per_run: 20,  // 20 samples minimum
            max_run_duration_ms: 10000, // 10 seconds maximum
            min_steady_state_ms: 2000,  // 2 seconds steady state
        }
    }
}