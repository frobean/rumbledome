//! System state management for RumbleDome

use serde::{Deserialize, Serialize};

/// Main system state machine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemState {
    /// System starting up, performing self-tests
    Initializing,
    
    /// System ready but not actively controlling boost
    /// - All sensors operational
    /// - CAN communication established  
    /// - Configuration loaded
    /// - Waiting for arming conditions
    Idle,
    
    /// System armed and actively controlling boost
    /// - Engine running (RPM > threshold)
    /// - All safety checks passing
    /// - Boost control active
    Armed,
    
    /// Auto-calibration in progress
    /// - Learning duty cycle mappings
    /// - Progressive safety limit expansion
    /// - User-initiated calibration sequence
    Calibrating(CalibrationState),
    
    /// Safety cut due to overboost condition
    /// - Manifold pressure exceeded limits
    /// - Duty cycle forced to 0%
    /// - Automatic recovery when pressure drops
    OverboostCut,
    
    /// System fault requiring attention
    /// - Sensor failures, CAN timeouts, storage errors
    /// - Manual intervention required for recovery
    /// - Duty cycle forced to 0% until resolved
    Fault(FaultCode),
}

/// Calibration sub-states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CalibrationState {
    /// Initial conservative calibration phase
    /// - Overboost limit at spring + 1 PSI
    /// - Multiple validation runs required
    /// - Builds confidence in system response
    Conservative {
        target_rpm: u16,
        target_boost: f32,
        runs_completed: u8,
    },
    
    /// Progressive limit expansion phase
    /// - Gradually increase overboost limits
    /// - Validate safety response at each level
    /// - Learn duty cycle mappings
    Progressive {
        current_limit: f32,
        confidence: f32,
        target_rpm: u16,
        target_boost: f32,
    },
    
    /// Calibration complete
    /// - Target boost level achieved safely
    /// - Duty cycle mapping learned and validated
    /// - System ready for normal operation
    Complete,
    
    /// Calibration failed
    /// - Safety response inadequate
    /// - Unable to achieve target safely
    /// - Manual intervention required
    Failed {
        reason: String,
    },
}

/// System fault codes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FaultCode {
    /// Sensor reading invalid or out of range
    SensorFault {
        sensor: String,
        details: String,
    },
    
    /// CAN communication timeout or invalid data
    CanTimeout {
        last_valid_ms: u64,
    },
    
    /// Storage system error (EEPROM/Flash)
    StorageFault {
        operation: String,
        details: String,
    },
    
    /// Hardware component failure
    HardwareFault {
        component: String,
        details: String,
    },
    
    /// Safety system failure
    SafetyFault {
        check: String,
        details: String,
    },
    
    /// Configuration validation error
    ConfigFault {
        parameter: String,
        details: String,
    },
    
    /// Watchdog timer timeout
    WatchdogTimeout,
    
    /// Unknown/unexpected error
    Unknown {
        details: String,
    },
}

impl SystemState {
    /// Check if system is in a safe state (0% duty cycle)
    pub fn is_safe_state(&self) -> bool {
        matches!(self, 
            SystemState::Initializing |
            SystemState::Idle |
            SystemState::OverboostCut |
            SystemState::Fault(_)
        )
    }
    
    /// Check if system is actively controlling boost
    pub fn is_controlling_boost(&self) -> bool {
        matches!(self, 
            SystemState::Armed |
            SystemState::Calibrating(_)
        )
    }
    
    /// Check if system can accept configuration changes
    pub fn can_reconfigure(&self) -> bool {
        matches!(self, 
            SystemState::Idle |
            SystemState::Fault(_)
        )
    }
    
    /// Get human-readable state description
    pub fn description(&self) -> String {
        match self {
            SystemState::Initializing => "System starting up".to_string(),
            SystemState::Idle => "Ready - waiting for engine".to_string(),
            SystemState::Armed => "Active boost control".to_string(),
            SystemState::Calibrating(cal_state) => {
                match cal_state {
                    CalibrationState::Conservative { runs_completed, .. } => {
                        format!("Calibrating - run {}/3", runs_completed)
                    },
                    CalibrationState::Progressive { current_limit, .. } => {
                        format!("Calibrating - limit {:.1} PSI", current_limit)
                    },
                    CalibrationState::Complete => "Calibration complete".to_string(),
                    CalibrationState::Failed { reason } => {
                        format!("Calibration failed: {}", reason)
                    },
                }
            },
            SystemState::OverboostCut => "OVERBOOST CUT".to_string(),
            SystemState::Fault(fault) => {
                format!("FAULT: {}", fault.description())
            },
        }
    }
}

impl CalibrationState {
    /// Check if calibration is in progress
    pub fn is_active(&self) -> bool {
        matches!(self, 
            CalibrationState::Conservative { .. } |
            CalibrationState::Progressive { .. }
        )
    }
    
    /// Check if calibration completed successfully
    pub fn is_complete(&self) -> bool {
        matches!(self, CalibrationState::Complete)
    }
    
    /// Check if calibration failed
    pub fn is_failed(&self) -> bool {
        matches!(self, CalibrationState::Failed { .. })
    }
}

impl FaultCode {
    /// Get human-readable fault description
    pub fn description(&self) -> String {
        match self {
            FaultCode::SensorFault { sensor, details } => {
                format!("{} sensor fault: {}", sensor, details)
            },
            FaultCode::CanTimeout { last_valid_ms } => {
                format!("CAN timeout - last valid {}ms ago", last_valid_ms)
            },
            FaultCode::StorageFault { operation, details } => {
                format!("Storage {} failed: {}", operation, details)
            },
            FaultCode::HardwareFault { component, details } => {
                format!("{} hardware fault: {}", component, details)
            },
            FaultCode::SafetyFault { check, details } => {
                format!("Safety check '{}' failed: {}", check, details)
            },
            FaultCode::ConfigFault { parameter, details } => {
                format!("Config parameter '{}' invalid: {}", parameter, details)
            },
            FaultCode::WatchdogTimeout => {
                "Watchdog timer timeout".to_string()
            },
            FaultCode::Unknown { details } => {
                format!("Unknown fault: {}", details)
            },
        }
    }
    
    /// Check if fault requires immediate shutdown
    pub fn is_critical(&self) -> bool {
        matches!(self,
            FaultCode::SafetyFault { .. } |
            FaultCode::HardwareFault { .. } |
            FaultCode::WatchdogTimeout
        )
    }
    
    /// Check if fault might be recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(self,
            FaultCode::SensorFault { .. } |
            FaultCode::CanTimeout { .. } |
            FaultCode::StorageFault { .. }
        )
    }
}

impl Default for SystemState {
    fn default() -> Self {
        SystemState::Initializing
    }
}