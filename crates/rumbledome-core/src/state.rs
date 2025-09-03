//! System State Management
//! 
//! 🔗 T4-CORE-017: System State Implementation
//! Derived From: T3-BUILD-003 (Core Control State Machine) + Safety.md state requirements
//! AI Traceability: Predictable state transitions, fault handling, safety state management

use alloc::string::String;
use serde::{Deserialize, Serialize};

/// System operational states
/// 
/// 🔗 T4-CORE-018: System State Enumeration
/// Derived From: Safety requirements + control flow requirements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemState {
    /// System initializing - hardware setup and self-test
    Initializing,
    
    /// System ready but not actively controlling boost
    /// Safe state with 0% PWM duty (wastegate open)
    Idle,
    
    /// Normal operation - torque-following boost control active
    Armed,
    
    /// Auto-calibration in progress
    Calibrating(CalibrationProgress),
    
    /// Overboost protection active - immediate 0% duty until pressure drops
    OverboostCut,
    
    /// System fault detected - requires user intervention
    Fault(FaultCode),
}

impl Default for SystemState {
    fn default() -> Self {
        SystemState::Initializing
    }
}

/// Auto-calibration progress tracking
/// 
/// 🔗 T4-CORE-019: Calibration Progress Implementation
/// Derived From: T2-CONTROL-007 (Progressive Safety Auto-Calibration)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationProgress {
    /// Current calibration phase (1-3)
    pub phase: u8,
    
    /// Progress within current phase (0.0-1.0)
    pub phase_progress: f32,
    
    /// Overall calibration progress (0.0-1.0)
    pub overall_progress: f32,
    
    /// Current boost target being calibrated (PSI)
    pub current_target_psi: f32,
    
    /// Current RPM point being calibrated
    pub current_rpm: u16,
    
    /// Number of validation runs completed for current point
    pub validation_runs: u8,
    
    /// Description of current calibration activity
    pub description: String,
}

/// System fault codes
/// 
/// 🔗 T4-CORE-020: Fault Code Classification
/// Derived From: Safety.md fault response hierarchy + diagnostic requirements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FaultCode {
    // Hardware Faults (Critical - immediate shutdown)
    /// Hardware self-test failed during initialization
    SelfTestFailed,
    
    /// PWM hardware failure detected
    PwmHardwareFault,
    
    /// Pressure sensor failure or out-of-range reading
    PressureSensorFault(String),
    
    /// CAN bus communication failure
    CanCommunicationLost,
    
    /// Storage system failure (SD card error)
    StorageSystemFault,
    
    // Safety Faults (Critical - immediate protection response)
    /// Manifold pressure exceeded overboost limit
    OverboostLimitExceeded { pressure_psi: f32, limit_psi: f32 },
    
    /// Pneumatic system failure (pressure not responding to duty cycle changes)
    PneumaticSystemFailure,
    
    /// Safety response time validation failed
    SafetyResponseTooSlow,
    
    // Configuration Faults (Warning - continue with defaults)
    /// Invalid user configuration detected
    InvalidConfiguration(String),
    
    /// Learned calibration data corrupted
    CalibrationDataCorrupted,
    
    // Sensor Faults (Warning - degraded operation)
    /// CAN torque signals not available or invalid
    TorqueSignalsInvalid,
    
    /// Pressure sensor reading implausible
    ImplausibleSensorReading { sensor: String, value: f32 },
    
    // Learning System Faults (Warning - continue without learning)
    /// Auto-calibration failed to converge
    CalibrationFailed(String),
    
    /// Learning system detected inconsistent data
    LearningInconsistency,
}

impl FaultCode {
    /// Check if fault is critical and requires immediate system shutdown
    /// 
    /// 🔗 T4-CORE-021: Critical Fault Classification
    /// Derived From: Safety.md fault response hierarchy
    pub fn is_critical(&self) -> bool {
        match self {
            // Hardware faults are always critical
            FaultCode::SelfTestFailed 
            | FaultCode::PwmHardwareFault
            | FaultCode::StorageSystemFault => true,
            
            // Safety faults are always critical
            FaultCode::OverboostLimitExceeded { .. }
            | FaultCode::PneumaticSystemFailure
            | FaultCode::SafetyResponseTooSlow => true,
            
            // CAN loss is critical for torque-following system
            FaultCode::CanCommunicationLost => true,
            
            // Pressure sensor faults may be critical depending on which sensor
            FaultCode::PressureSensorFault(sensor) => {
                sensor.contains("manifold") // Manifold pressure is critical for safety
            },
            
            // Other faults are warnings
            _ => false,
        }
    }
    
    /// Get human-readable description of fault
    pub fn description(&self) -> String {
        match self {
            FaultCode::SelfTestFailed => 
                "Hardware self-test failed during initialization".to_string(),
            
            FaultCode::PwmHardwareFault => 
                "PWM hardware failure - solenoid control unavailable".to_string(),
            
            FaultCode::PressureSensorFault(sensor) => 
                format!("Pressure sensor fault: {}", sensor),
            
            FaultCode::CanCommunicationLost => 
                "CAN bus communication lost - no ECU data available".to_string(),
            
            FaultCode::StorageSystemFault => 
                "SD card storage failure - configuration may be lost".to_string(),
            
            FaultCode::OverboostLimitExceeded { pressure_psi, limit_psi } => 
                format!("Overboost protection: {:.1} PSI exceeded limit of {:.1} PSI", 
                    pressure_psi, limit_psi),
            
            FaultCode::PneumaticSystemFailure => 
                "Pneumatic system not responding - wastegate control ineffective".to_string(),
            
            FaultCode::SafetyResponseTooSlow => 
                "Safety response time exceeded specification - system unsafe".to_string(),
            
            FaultCode::InvalidConfiguration(msg) => 
                format!("Invalid configuration: {}", msg),
            
            FaultCode::CalibrationDataCorrupted => 
                "Calibration data corrupted - using defaults".to_string(),
            
            FaultCode::TorqueSignalsInvalid => 
                "ECU torque signals invalid - reduced functionality".to_string(),
            
            FaultCode::ImplausibleSensorReading { sensor, value } => 
                format!("Implausible reading from {}: {:.2}", sensor, value),
            
            FaultCode::CalibrationFailed(msg) => 
                format!("Auto-calibration failed: {}", msg),
            
            FaultCode::LearningInconsistency => 
                "Learning system detected inconsistent data".to_string(),
        }
    }
    
    /// Get recommended user action for fault
    pub fn recommended_action(&self) -> String {
        match self {
            FaultCode::SelfTestFailed => 
                "Check all hardware connections and power supply".to_string(),
            
            FaultCode::PwmHardwareFault => 
                "Check solenoid wiring and driver circuit".to_string(),
            
            FaultCode::PressureSensorFault(_) => 
                "Check pressure sensor connections and air lines".to_string(),
            
            FaultCode::CanCommunicationLost => 
                "Check CAN bus connections and ECU power".to_string(),
            
            FaultCode::StorageSystemFault => 
                "Replace SD card and restore configuration backup".to_string(),
            
            FaultCode::OverboostLimitExceeded { .. } => 
                "Reduce boost targets or check wastegate operation".to_string(),
            
            FaultCode::PneumaticSystemFailure => 
                "Check air supply pressure and wastegate linkage".to_string(),
            
            FaultCode::SafetyResponseTooSlow => 
                "Check pneumatic system for leaks or restrictions".to_string(),
            
            FaultCode::InvalidConfiguration(_) => 
                "Review and correct configuration parameters".to_string(),
            
            FaultCode::CalibrationDataCorrupted => 
                "Reset calibration data and re-run auto-calibration".to_string(),
            
            FaultCode::TorqueSignalsInvalid => 
                "Verify CAN signal mapping for your ECU".to_string(),
            
            FaultCode::ImplausibleSensorReading { .. } => 
                "Check sensor calibration and connections".to_string(),
            
            FaultCode::CalibrationFailed(_) => 
                "Check pneumatic system and retry calibration".to_string(),
            
            FaultCode::LearningInconsistency => 
                "Reset learned data if problem persists".to_string(),
        }
    }
}

impl SystemState {
    /// Check if system can transition to armed state
    /// 
    /// 🔗 T4-CORE-022: State Transition Validation
    /// Derived From: Safety requirements for operational state transitions
    pub fn can_transition_to_armed(&self) -> bool {
        match self {
            SystemState::Idle => true,
            SystemState::OverboostCut => false, // Must clear overboost first
            SystemState::Fault(fault) => !fault.is_critical(),
            SystemState::Calibrating(_) => false, // Must complete calibration
            SystemState::Initializing => false, // Must complete initialization
            SystemState::Armed => true, // Already armed
        }
    }
    
    /// Check if system should force 0% PWM duty
    /// 
    /// 🔗 T4-CORE-023: Failsafe State Detection
    /// Derived From: T1-SAFETY-001 (Overboost as Fault Condition)
    pub fn requires_failsafe_pwm(&self) -> bool {
        match self {
            SystemState::Initializing => true,
            SystemState::Idle => true,
            SystemState::OverboostCut => true,
            SystemState::Fault(fault) => fault.is_critical(),
            SystemState::Armed => false,
            SystemState::Calibrating(_) => false, // Calibration controls PWM
        }
    }
    
    /// Get display status text for current state
    pub fn display_text(&self) -> String {
        match self {
            SystemState::Initializing => "INIT".to_string(),
            SystemState::Idle => "IDLE".to_string(),
            SystemState::Armed => "ARMED".to_string(),
            SystemState::Calibrating(progress) => 
                format!("CAL {}%", (progress.overall_progress * 100.0) as u8),
            SystemState::OverboostCut => "OVERBOOST".to_string(),
            SystemState::Fault(_) => "FAULT".to_string(),
        }
    }
    
    /// Get state priority for display (higher = more important to show)
    pub fn display_priority(&self) -> u8 {
        match self {
            SystemState::Fault(_) => 255,        // Highest priority - always show
            SystemState::OverboostCut => 200,    // Very high - safety critical
            SystemState::Initializing => 150,    // High - startup state
            SystemState::Calibrating(_) => 100,  // Medium - user should know
            SystemState::Armed => 50,            // Low - normal operation
            SystemState::Idle => 25,             // Lowest - standby state
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_critical_fault_detection() {
        assert!(FaultCode::SelfTestFailed.is_critical());
        assert!(FaultCode::OverboostLimitExceeded { pressure_psi: 16.0, limit_psi: 15.0 }.is_critical());
        assert!(!FaultCode::InvalidConfiguration("test".to_string()).is_critical());
        assert!(!FaultCode::LearningInconsistency.is_critical());
    }
    
    #[test]
    fn test_failsafe_state_detection() {
        assert!(SystemState::Initializing.requires_failsafe_pwm());
        assert!(SystemState::Idle.requires_failsafe_pwm());
        assert!(SystemState::OverboostCut.requires_failsafe_pwm());
        assert!(SystemState::Fault(FaultCode::SelfTestFailed).requires_failsafe_pwm());
        assert!(!SystemState::Armed.requires_failsafe_pwm());
    }
    
    #[test]
    fn test_state_transitions() {
        assert!(SystemState::Idle.can_transition_to_armed());
        assert!(!SystemState::OverboostCut.can_transition_to_armed());
        assert!(!SystemState::Fault(FaultCode::SelfTestFailed).can_transition_to_armed());
        assert!(SystemState::Fault(FaultCode::LearningInconsistency).can_transition_to_armed());
    }
    
    #[test]
    fn test_display_priority() {
        assert!(SystemState::Fault(FaultCode::SelfTestFailed).display_priority() > 
                SystemState::Armed.display_priority());
        assert!(SystemState::OverboostCut.display_priority() > 
                SystemState::Calibrating(CalibrationProgress {
                    phase: 1,
                    phase_progress: 0.5,
                    overall_progress: 0.2,
                    current_target_psi: 8.0,
                    current_rpm: 2000,
                    validation_runs: 2,
                    description: "Test".to_string(),
                }).display_priority());
    }
}