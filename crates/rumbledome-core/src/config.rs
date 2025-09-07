//! Configuration Management
//! 
//! 🔗 T4-CORE-010: Configuration Implementation
//! Derived From: T3-BUILD-004 (5-Parameter Configuration Implementation) + T2-HAL-003
//! AI Traceability: Single-knob philosophy implementation, parameter validation

use alloc::string::String;
use serde::{Deserialize, Serialize};
use crate::CoreError;

/// User configuration structure - exactly 5 parameters
/// 
/// 🔗 T4-CORE-011: 5-Parameter Configuration Structure
/// Derived From: T1-UI-001 (Single Parameter Philosophy) + T2-CONFIG-001 (Pressure-Based Configuration)
/// AI Traceability: Implements single-knob control philosophy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemConfig {
    /// Aggression level (0.0-1.0) - scales all system behavior
    /// 0.0 = OFF (as close to naturally aspirated as physically possible)
    /// 1.0 = Maximum system aggression (instant ECU torque request assistance)
    pub aggression: f32,
    
    /// Wastegate spring pressure in PSI
    /// Used for control authority calculations and mechanical failsafe baseline
    pub spring_pressure: f32,
    
    /// Maximum boost pressure ceiling in PSI (not knob-scaled target)
    /// Safety ceiling for boost pressure - system will not exceed this value
    pub max_boost_psi: f32,
    
    /// Hard safety fault threshold in PSI - never exceed
    /// Overboost protection triggers fault condition if exceeded
    pub overboost_limit: f32,
    
    /// Enable scramble button feature (temporary maximum aggression override)
    pub scramble_enabled: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            aggression: 0.3,           // Conservative 30% for daily driving
            spring_pressure: 5.0,      // Typical wastegate spring pressure
            max_boost_psi: 12.0,       // Conservative boost ceiling
            overboost_limit: 15.0,     // Hard safety limit
            scramble_enabled: true,    // Enable scramble override
        }
    }
}

impl SystemConfig {
    /// Validate configuration parameters
    /// 
    /// 🔗 T4-CORE-012: Configuration Validation
    /// Derived From: Safety.md parameter validation requirements
    pub fn validate(&self) -> Result<(), CoreError> {
        // Validate aggression range
        if !(0.0..=1.0).contains(&self.aggression) {
            return Err(CoreError::ConfigurationError(
                format!("Aggression must be 0.0-1.0, got {}", self.aggression)
            ));
        }
        
        // Validate spring pressure
        if self.spring_pressure < 1.0 || self.spring_pressure > 20.0 {
            return Err(CoreError::ConfigurationError(
                format!("Spring pressure must be 1.0-20.0 PSI, got {}", self.spring_pressure)
            ));
        }
        
        // Validate max boost
        if self.max_boost_psi < self.spring_pressure {
            return Err(CoreError::ConfigurationError(
                format!("Max boost ({} PSI) must be >= spring pressure ({} PSI)", 
                    self.max_boost_psi, self.spring_pressure)
            ));
        }
        
        if self.max_boost_psi > 25.0 {
            return Err(CoreError::ConfigurationError(
                format!("Max boost must be <= 25.0 PSI, got {}", self.max_boost_psi)
            ));
        }
        
        // Validate overboost limit with safety margin
        const MINIMUM_SAFETY_MARGIN_PSI: f32 = 1.5;
        if self.overboost_limit <= (self.max_boost_psi + MINIMUM_SAFETY_MARGIN_PSI) {
            return Err(CoreError::ConfigurationError(
                format!("Overboost limit ({} PSI) must be at least {} PSI above max boost ({} PSI)", 
                    self.overboost_limit, MINIMUM_SAFETY_MARGIN_PSI, self.max_boost_psi)
            ));
        }
        
        if self.overboost_limit > 30.0 {
            return Err(CoreError::ConfigurationError(
                format!("Overboost limit must be <= 30.0 PSI, got {}", self.overboost_limit)
            ));
        }
        
        Ok(())
    }
    
    /// Get response characteristics derived from aggression setting
    /// 
    /// 🔗 T4-CORE-013: Aggression-Based Behavior Scaling
    /// Derived From: T2-CONTROL-001 (Priority Hierarchy) + behavioral scaling requirements
    /// All complex system behavior derived from single aggression parameter
    pub fn get_response_characteristics(&self) -> ResponseProfile {
        ResponseProfile {
            // Tip-in sensitivity: how quickly system responds to torque requests
            tip_in_sensitivity: self.aggression * 2.0,
            
            // Tip-out decay: how quickly system backs off when torque demand drops
            tip_out_decay_rate: self.aggression * 0.5 + 0.2,
            
            // Torque following gain: amplification of ECU assistance
            torque_following_gain: self.aggression * 1.5 + 0.3,
            
            // Boost ramp rate: maximum rate of boost pressure increase
            boost_ramp_rate: self.aggression * 3.0 + 1.0,
            
            // Safety margin: how close to limits before backing off
            safety_margin_factor: 1.0 - (self.aggression * 0.2),
            
            // PID aggressiveness: how hard PID controller pushes
            pid_aggressiveness: self.aggression * 0.8 + 0.2,
        }
    }
    
    /// Get OFF behavior settings (aggression = 0.0)
    /// 
    /// 🔗 T4-CORE-014: OFF Mode Implementation
    /// Derived From: T2-CONFIG-001 (0.0% OFF Requirement)
    /// As close to naturally aspirated operation as physically possible
    pub fn is_off_mode(&self) -> bool {
        self.aggression == 0.0
    }
    
    /// Get scramble behavior settings (temporary 100% aggression)
    /// 
    /// 🔗 T4-CORE-015: Scramble Mode Implementation
    /// Derived From: Scramble override requirements
    pub fn get_scramble_characteristics(&self) -> ResponseProfile {
        if !self.scramble_enabled {
            return self.get_response_characteristics();
        }
        
        // Temporary maximum aggression override
        ResponseProfile {
            tip_in_sensitivity: 2.0,      // Maximum responsiveness
            tip_out_decay_rate: 0.7,      // Fast decay when released
            torque_following_gain: 1.8,   // Maximum ECU assistance
            boost_ramp_rate: 4.0,         // Maximum ramp rate
            safety_margin_factor: 0.8,    // Reduced safety margin
            pid_aggressiveness: 1.0,      // Maximum PID aggression
        }
    }
    
    /// Update aggression setting with validation
    /// 
    /// Used for live adjustment via rotary encoder
    pub fn set_aggression(&mut self, new_aggression: f32) -> Result<(), CoreError> {
        if !(0.0..=1.0).contains(&new_aggression) {
            return Err(CoreError::ConfigurationError(
                format!("Aggression must be 0.0-1.0, got {}", new_aggression)
            ));
        }
        
        self.aggression = new_aggression;
        Ok(())
    }
    
    /// Convert to JSON for storage
    pub fn to_json(&self) -> Result<String, CoreError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| CoreError::ConfigurationError(format!("JSON serialization failed: {}", e)))
    }
    
    /// Load from JSON string
    pub fn from_json(json: &str) -> Result<Self, CoreError> {
        let config: SystemConfig = serde_json::from_str(json)
            .map_err(|e| CoreError::ConfigurationError(format!("JSON parsing failed: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
}

/// Response characteristics derived from aggression setting
/// 
/// 🔗 T4-CORE-016: Response Profile Implementation
/// Derived From: Aggression scaling requirements + control behavior specification
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseProfile {
    /// How quickly system responds to torque requests (0.0-2.0)
    pub tip_in_sensitivity: f32,
    
    /// How quickly system backs off when demand drops (0.2-0.7)
    pub tip_out_decay_rate: f32,
    
    /// Amplification factor for ECU torque assistance (0.3-1.8)
    pub torque_following_gain: f32,
    
    /// Maximum boost ramp rate in PSI/second (1.0-4.0)
    pub boost_ramp_rate: f32,
    
    /// Safety margin factor before backing off (0.8-1.0)
    pub safety_margin_factor: f32,
    
    /// PID controller aggressiveness (0.2-1.0)
    pub pid_aggressiveness: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config_valid() {
        let config = SystemConfig::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_aggression_validation() {
        let mut config = SystemConfig::default();
        
        // Valid aggression values
        assert!(config.set_aggression(0.0).is_ok());
        assert!(config.set_aggression(0.5).is_ok());
        assert!(config.set_aggression(1.0).is_ok());
        
        // Invalid aggression values
        assert!(config.set_aggression(-0.1).is_err());
        assert!(config.set_aggression(1.1).is_err());
    }
    
    #[test]
    fn test_boost_limit_validation() {
        let mut config = SystemConfig::default();
        
        // Max boost must be >= spring pressure
        config.spring_pressure = 10.0;
        config.max_boost_psi = 5.0;
        assert!(config.validate().is_err());
        
        // Overboost must be > max boost
        config.max_boost_psi = 15.0;
        config.overboost_limit = 10.0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_response_profile_scaling() {
        let config = SystemConfig::default();
        
        // Test minimum aggression (0.0)
        let mut config_min = config.clone();
        config_min.aggression = 0.0;
        let profile_min = config_min.get_response_characteristics();
        
        // Test maximum aggression (1.0) 
        let mut config_max = config.clone();
        config_max.aggression = 1.0;
        let profile_max = config_max.get_response_characteristics();
        
        // Higher aggression should increase all response characteristics
        assert!(profile_max.tip_in_sensitivity > profile_min.tip_in_sensitivity);
        assert!(profile_max.torque_following_gain > profile_min.torque_following_gain);
        assert!(profile_max.boost_ramp_rate > profile_min.boost_ramp_rate);
    }
    
    #[test]
    fn test_json_serialization() {
        let config = SystemConfig::default();
        let json = config.to_json().unwrap();
        let deserialized = SystemConfig::from_json(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}