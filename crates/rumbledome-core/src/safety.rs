//! Safety monitoring system for RumbleDome

use crate::config::{SystemConfig, BoostProfile, SafetyConfig};
use crate::error::CoreError;
use crate::state::{SystemState, FaultCode};
use rumbledome_hal::{SystemInputs, SensorReadings, CanData};

/// Safety monitoring and override system
pub struct SafetyMonitor {
    /// Overboost detection and response
    overboost_detector: OverboostDetector,
    
    /// Pneumatic system safety validator
    pneumatic_validator: PneumaticValidator,
    
    /// Progressive safety limits manager
    progressive_limits: ProgressiveLimits,
    
    /// Safety event history
    safety_history: SafetyHistory,
}

/// Overboost detection and hysteresis management
#[derive(Debug, Clone)]
pub struct OverboostDetector {
    /// Current overboost state
    is_overboost: bool,
    
    /// Timestamp of overboost detection
    overboost_start_ms: Option<u64>,
    
    /// Recovery hysteresis (PSI below limit to clear)
    hysteresis: f32,
    
    /// Consecutive overboost readings required
    consecutive_readings_required: u8,
    
    /// Current consecutive reading count
    consecutive_readings: u8,
}

/// Pneumatic system response validation
#[derive(Debug, Clone)]
pub struct PneumaticValidator {
    /// Expected pressure response time (ms)
    max_response_time_ms: u64,
    
    /// Minimum dome pressure delta for validation
    min_pressure_delta: f32,
    
    /// Last validation timestamp
    last_validation_ms: u64,
    
    /// Validation success history
    validation_history: Vec<bool>,
}

/// Progressive safety limits based on system confidence
#[derive(Debug, Clone)]  
pub struct ProgressiveLimits {
    /// Base overboost limit (spring + safety margin)
    base_limit: f32,
    
    /// Current allowed overboost limit
    current_limit: f32,
    
    /// Maximum allowed limit progression
    max_progression: f32,
    
    /// System confidence required for limit increases
    confidence_threshold: f32,
    
    /// Number of successful responses required to increase limits
    successful_responses_required: u32,
    
    /// Current successful response count
    successful_responses: u32,
}

/// Safety event history and statistics
#[derive(Debug, Clone)]
pub struct SafetyHistory {
    /// Recent overboost events
    overboost_events: Vec<SafetyEvent>,
    
    /// Recent fault events
    fault_events: Vec<SafetyEvent>,
    
    /// Safety response times (ms)
    response_times: Vec<u64>,
    
    /// Total safety cuts performed
    total_safety_cuts: u32,
    
    /// Last safety event timestamp
    last_event_ms: Option<u64>,
}

/// Individual safety event record
#[derive(Debug, Clone)]
pub struct SafetyEvent {
    /// Event timestamp
    pub timestamp_ms: u64,
    
    /// Event type
    pub event_type: SafetyEventType,
    
    /// Trigger value (pressure, time, etc.)
    pub trigger_value: f32,
    
    /// System response time (ms)
    pub response_time_ms: u64,
    
    /// Recovery success
    pub recovered: bool,
}

/// Types of safety events
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyEventType {
    /// Manifold overboost condition
    Overboost,
    
    /// Pneumatic response failure
    PneumaticFailure,
    
    /// Sensor validation failure
    SensorFailure,
    
    /// Control system fault
    ControlFault,
    
    /// Configuration safety violation
    ConfigurationFault,
}

impl SafetyMonitor {
    /// Create new safety monitor
    pub fn new() -> Self {
        Self {
            overboost_detector: OverboostDetector::new(),
            pneumatic_validator: PneumaticValidator::new(),
            progressive_limits: ProgressiveLimits::new(5.0), // 5 PSI spring pressure default
            safety_history: SafetyHistory::new(),
        }
    }
    
    /// Execute complete safety check and return actions
    pub fn check_safety(
        &mut self,
        inputs: &SystemInputs,
        config: &SystemConfig,
        profile: &BoostProfile,
        current_duty: f32,
    ) -> Result<SafetyAction, CoreError> {
        
        // Check for overboost condition
        if let Some(overboost_action) = self.check_overboost(inputs, profile)? {
            return Ok(overboost_action);
        }
        
        // Validate sensor readings
        self.validate_sensors(inputs, &config.safety)?;
        
        // Validate pneumatic system response
        self.validate_pneumatic_response(inputs)?;
        
        // Check progressive safety limits
        let progressive_limit = self.progressive_limits.get_current_limit(profile);
        if inputs.sensors.manifold_pressure_gauge > progressive_limit {
            let event = SafetyEvent {
                timestamp_ms: inputs.timestamp_ms,
                event_type: SafetyEventType::Overboost,
                trigger_value: inputs.sensors.manifold_pressure_gauge,
                response_time_ms: 0, // Will be updated when action taken
                recovered: false,
            };
            
            self.safety_history.record_event(event);
            
            return Ok(SafetyAction::ImmediateCut {
                reason: format!("Progressive limit exceeded: {:.1} > {:.1} PSI", 
                    inputs.sensors.manifold_pressure_gauge, progressive_limit),
            });
        }
        
        // Check control system safety constraints
        if let Some(control_action) = self.check_control_safety(current_duty, config)? {
            return Ok(control_action);
        }
        
        Ok(SafetyAction::Continue)
    }
    
    /// Check for overboost condition with hysteresis
    fn check_overboost(
        &mut self,
        inputs: &SystemInputs,
        profile: &BoostProfile,
    ) -> Result<Option<SafetyAction>, CoreError> {
        
        let current_pressure = inputs.sensors.manifold_pressure_gauge;
        let overboost_limit = profile.overboost_limit;
        
        if !self.overboost_detector.is_overboost {
            // Not currently in overboost - check for trigger
            if current_pressure > overboost_limit {
                self.overboost_detector.consecutive_readings += 1;
                
                if self.overboost_detector.consecutive_readings >= 
                   self.overboost_detector.consecutive_readings_required {
                    
                    // Trigger overboost condition
                    self.overboost_detector.is_overboost = true;
                    self.overboost_detector.overboost_start_ms = Some(inputs.timestamp_ms);
                    
                    let event = SafetyEvent {
                        timestamp_ms: inputs.timestamp_ms,
                        event_type: SafetyEventType::Overboost,
                        trigger_value: current_pressure,
                        response_time_ms: 0,
                        recovered: false,
                    };
                    
                    self.safety_history.record_event(event);
                    
                    return Ok(Some(SafetyAction::ImmediateCut {
                        reason: format!("Overboost detected: {:.1} > {:.1} PSI", 
                            current_pressure, overboost_limit),
                    }));
                }
            } else {
                // Reset consecutive readings
                self.overboost_detector.consecutive_readings = 0;
            }
            
        } else {
            // Currently in overboost - check for recovery
            let recovery_threshold = overboost_limit - profile.overboost_hysteresis;
            
            if current_pressure < recovery_threshold {
                // Recovery achieved
                self.overboost_detector.is_overboost = false;
                self.overboost_detector.consecutive_readings = 0;
                
                if let Some(start_time) = self.overboost_detector.overboost_start_ms {
                    let recovery_time = inputs.timestamp_ms.saturating_sub(start_time);
                    self.safety_history.response_times.push(recovery_time);
                }
                
                return Ok(Some(SafetyAction::AllowRecovery {
                    message: format!("Overboost recovery: {:.1} < {:.1} PSI", 
                        current_pressure, recovery_threshold),
                }));
            } else {
                // Still in overboost - maintain cut
                return Ok(Some(SafetyAction::MaintainCut {
                    reason: format!("Overboost persists: {:.1} PSI", current_pressure),
                }));
            }
        }
        
        Ok(None)
    }
    
    /// Validate sensor readings are within expected ranges
    fn validate_sensors(
        &mut self,
        inputs: &SystemInputs,
        safety_config: &SafetyConfig,
    ) -> Result<(), CoreError> {
        
        let sensors = &inputs.sensors;
        
        // Validate dome input pressure
        let (min_dome, max_dome) = safety_config.sensor_ranges.dome_input_pressure;
        if sensors.dome_input_pressure < min_dome || sensors.dome_input_pressure > max_dome {
            return Err(CoreError::safety(format!(
                "Dome input pressure out of range: {:.1} PSI (expected {:.1}-{:.1})",
                sensors.dome_input_pressure, min_dome, max_dome
            )));
        }
        
        // Validate upper dome pressure
        let (min_upper, max_upper) = safety_config.sensor_ranges.upper_dome_pressure;
        if sensors.upper_dome_pressure < min_upper || sensors.upper_dome_pressure > max_upper {
            return Err(CoreError::safety(format!(
                "Upper dome pressure out of range: {:.1} PSI (expected {:.1}-{:.1})",
                sensors.upper_dome_pressure, min_upper, max_upper
            )));
        }
        
        // Validate manifold pressure
        let (min_map, max_map) = safety_config.sensor_ranges.manifold_pressure;
        if sensors.manifold_pressure_gauge < min_map || sensors.manifold_pressure_gauge > max_map {
            return Err(CoreError::safety(format!(
                "Manifold pressure out of range: {:.1} PSI (expected {:.1}-{:.1})",
                sensors.manifold_pressure_gauge, min_map, max_map
            )));
        }
        
        Ok(())
    }
    
    /// Validate pneumatic system can respond adequately to control inputs
    fn validate_pneumatic_response(&mut self, inputs: &SystemInputs) -> Result<(), CoreError> {
        
        // Check if we have adequate dome pressure differential for control
        let pressure_differential = inputs.sensors.dome_input_pressure - 
            inputs.sensors.upper_dome_pressure;
        
        if pressure_differential < self.pneumatic_validator.min_pressure_delta {
            return Err(CoreError::safety(format!(
                "Insufficient pneumatic pressure differential: {:.1} PSI",
                pressure_differential
            )));
        }
        
        // Record successful validation
        self.pneumatic_validator.validation_history.push(true);
        
        // Keep only recent validation history
        if self.pneumatic_validator.validation_history.len() > 100 {
            self.pneumatic_validator.validation_history.remove(0);
        }
        
        Ok(())
    }
    
    /// Check control system safety constraints
    fn check_control_safety(
        &self,
        current_duty: f32,
        config: &SystemConfig,
    ) -> Result<Option<SafetyAction>, CoreError> {
        
        // Check for excessive duty cycle
        if current_duty > 90.0 {
            return Ok(Some(SafetyAction::LimitDuty {
                max_duty: 90.0,
                reason: "Excessive duty cycle limited for safety".to_string(),
            }));
        }
        
        // Check duty cycle change rate (implemented in slew limiter)
        // This is a secondary check
        
        Ok(None)
    }
    
    /// Update progressive limits based on system performance
    pub fn update_progressive_limits(
        &mut self,
        system_confidence: f32,
        response_success: bool,
    ) {
        if response_success {
            self.progressive_limits.successful_responses += 1;
        } else {
            // Reset progress on failed response
            self.progressive_limits.successful_responses = 0;
        }
        
        // Increase limits if we have sufficient confidence and successful responses
        if system_confidence >= self.progressive_limits.confidence_threshold &&
           self.progressive_limits.successful_responses >= 
           self.progressive_limits.successful_responses_required {
            
            let increase = 0.5; // 0.5 PSI increase
            let new_limit = (self.progressive_limits.current_limit + increase)
                .min(self.progressive_limits.max_progression);
            
            if new_limit > self.progressive_limits.current_limit {
                self.progressive_limits.current_limit = new_limit;
                self.progressive_limits.successful_responses = 0; // Reset counter
            }
        }
    }
    
    /// Get safety system diagnostics
    pub fn get_diagnostics(&self) -> SafetyDiagnostics {
        SafetyDiagnostics {
            overboost_events: self.safety_history.overboost_events.len() as u32,
            fault_events: self.safety_history.fault_events.len() as u32,
            safety_cuts: self.safety_history.total_safety_cuts,
            current_progressive_limit: self.progressive_limits.current_limit,
            successful_responses: self.progressive_limits.successful_responses,
            average_response_time: self.calculate_average_response_time(),
        }
    }
    
    fn calculate_average_response_time(&self) -> f32 {
        if self.safety_history.response_times.is_empty() {
            0.0
        } else {
            let sum: u64 = self.safety_history.response_times.iter().sum();
            (sum as f32) / (self.safety_history.response_times.len() as f32)
        }
    }
}

/// Safety system action to take
#[derive(Debug, Clone)]
pub enum SafetyAction {
    /// Continue normal operation
    Continue,
    
    /// Immediately cut duty cycle to 0%
    ImmediateCut {
        reason: String,
    },
    
    /// Maintain current 0% duty cycle
    MaintainCut {
        reason: String,
    },
    
    /// Allow recovery from safety cut
    AllowRecovery {
        message: String,
    },
    
    /// Limit duty cycle to maximum value
    LimitDuty {
        max_duty: f32,
        reason: String,
    },
}

/// Safety system diagnostic information
#[derive(Debug, Clone)]
pub struct SafetyDiagnostics {
    pub overboost_events: u32,
    pub fault_events: u32,
    pub safety_cuts: u32,
    pub current_progressive_limit: f32,
    pub successful_responses: u32,
    pub average_response_time: f32,
}

// Implementation details for internal types...

impl OverboostDetector {
    fn new() -> Self {
        Self {
            is_overboost: false,
            overboost_start_ms: None,
            hysteresis: 0.3, // Default hysteresis
            consecutive_readings_required: 2, // Require 2 consecutive readings
            consecutive_readings: 0,
        }
    }
}

impl PneumaticValidator {
    fn new() -> Self {
        Self {
            max_response_time_ms: 100,
            min_pressure_delta: 2.0, // Minimum 2 PSI differential
            last_validation_ms: 0,
            validation_history: Vec::new(),
        }
    }
}

impl ProgressiveLimits {
    fn new(spring_pressure: f32) -> Self {
        let base_limit = spring_pressure + 1.0; // Start conservatively
        
        Self {
            base_limit,
            current_limit: base_limit,
            max_progression: spring_pressure + 10.0, // Maximum progression
            confidence_threshold: 0.8,
            successful_responses_required: 5,
            successful_responses: 0,
        }
    }
    
    fn get_current_limit(&self, profile: &BoostProfile) -> f32 {
        // Use the most restrictive of progressive limit or profile limit
        self.current_limit.min(profile.overboost_limit)
    }
}

impl SafetyHistory {
    fn new() -> Self {
        Self {
            overboost_events: Vec::new(),
            fault_events: Vec::new(),
            response_times: Vec::new(),
            total_safety_cuts: 0,
            last_event_ms: None,
        }
    }
    
    fn record_event(&mut self, mut event: SafetyEvent) {
        match event.event_type {
            SafetyEventType::Overboost => {
                self.overboost_events.push(event);
                self.total_safety_cuts += 1;
            },
            _ => {
                self.fault_events.push(event);
            }
        }
        
        self.last_event_ms = Some(event.timestamp_ms);
        
        // Limit history size
        if self.overboost_events.len() > 50 {
            self.overboost_events.remove(0);
        }
        if self.fault_events.len() > 50 {
            self.fault_events.remove(0);
        }
        if self.response_times.len() > 100 {
            self.response_times.remove(0);
        }
    }
}