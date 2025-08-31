//! Protocol message definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Protocol version
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Top-level protocol message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Protocol version
    pub version: String,
    
    /// Unique message ID for request/response correlation
    pub id: String,
    
    /// Message timestamp (Unix milliseconds)
    pub timestamp: u64,
    
    /// Message payload
    #[serde(flatten)]
    pub payload: MessagePayload,
}

/// Message payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
    /// Request messages (client to RumbleDome)
    #[serde(rename = "request")]
    Request(RequestPayload),
    
    /// Response messages (RumbleDome to client)
    #[serde(rename = "response")]
    Response(ResponsePayload),
    
    /// Unsolicited notifications (RumbleDome to client)
    #[serde(rename = "notification")]
    Notification(NotificationPayload),
}

/// Request message payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum RequestPayload {
    /// Get system status
    #[serde(rename = "get_status")]
    GetStatus,
    
    /// Get current configuration
    #[serde(rename = "get_config")]
    GetConfig,
    
    /// Update configuration
    #[serde(rename = "set_config")]
    SetConfig {
        config: SystemConfigDto,
    },
    
    /// Start calibration sequence
    #[serde(rename = "start_calibration")]
    StartCalibration {
        target_rpm: u16,
        target_boost: f32,
    },
    
    /// Stop calibration sequence
    #[serde(rename = "stop_calibration")]
    StopCalibration,
    
    /// Reset learned data
    #[serde(rename = "reset_learning")]
    ResetLearning,
    
    /// Change active profile
    #[serde(rename = "set_profile")]
    SetProfile {
        profile_name: String,
    },
    
    /// Get diagnostic data
    #[serde(rename = "get_diagnostics")]
    GetDiagnostics,
    
    /// Get telemetry stream
    #[serde(rename = "subscribe_telemetry")]
    SubscribeTelemetry {
        rate_hz: u16,
    },
    
    /// Stop telemetry stream
    #[serde(rename = "unsubscribe_telemetry")]
    UnsubscribeTelemetry,
    
    /// System shutdown
    #[serde(rename = "shutdown")]
    Shutdown,
}

/// Response message payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum ResponsePayload {
    /// Successful response
    #[serde(rename = "ok")]
    Ok {
        data: Option<ResponseData>,
    },
    
    /// Error response
    #[serde(rename = "error")]
    Error {
        code: ErrorCode,
        message: String,
        details: Option<serde_json::Value>,
    },
}

/// Response data payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseData {
    /// System status response
    #[serde(rename = "status")]
    Status(SystemStatusDto),
    
    /// Configuration response
    #[serde(rename = "config")]
    Config(SystemConfigDto),
    
    /// Diagnostic data response
    #[serde(rename = "diagnostics")]
    Diagnostics(DiagnosticsDto),
}

/// Notification payloads (unsolicited)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum NotificationPayload {
    /// System state changed
    #[serde(rename = "state_changed")]
    StateChanged {
        old_state: String,
        new_state: String,
        timestamp: u64,
    },
    
    /// Fault condition occurred
    #[serde(rename = "fault")]
    Fault {
        fault_code: String,
        description: String,
        severity: String,
        timestamp: u64,
    },
    
    /// Calibration progress update
    #[serde(rename = "calibration_progress")]
    CalibrationProgress {
        phase: String,
        progress_percent: u8,
        current_target: Option<f32>,
        message: String,
    },
    
    /// Real-time telemetry data
    #[serde(rename = "telemetry")]
    Telemetry(TelemetryDto),
}

/// Error codes for protocol responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Invalid request format
    InvalidRequest,
    
    /// Unknown command
    UnknownCommand,
    
    /// Invalid parameters
    InvalidParameters,
    
    /// System not in correct state for operation
    InvalidState,
    
    /// Configuration validation failed
    ConfigurationError,
    
    /// Hardware error
    HardwareError,
    
    /// Safety system prevented operation
    SafetyError,
    
    /// Internal system error
    InternalError,
}

/// System status data transfer object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatusDto {
    /// Current system state
    pub state: String,
    
    /// State description
    pub state_description: String,
    
    /// Active profile name
    pub active_profile: String,
    
    /// Current sensor readings
    pub sensors: SensorReadingsDto,
    
    /// Current CAN data
    pub can_data: CanDataDto,
    
    /// Current control outputs
    pub outputs: OutputsDto,
    
    /// System uptime in seconds
    pub uptime_seconds: u64,
    
    /// Last error (if any)
    pub last_error: Option<String>,
}

/// Configuration data transfer object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfigDto {
    /// Wastegate spring pressure
    pub spring_pressure: f32,
    
    /// Boost profiles
    pub profiles: HashMap<String, BoostProfileDto>,
    
    /// Active profile name
    pub active_profile: String,
    
    /// Scramble profile name
    pub scramble_profile: String,
    
    /// Torque target percentage
    pub torque_target_percentage: f32,
    
    /// Boost slew rate limit
    pub boost_slew_rate: f32,
    
    /// Safety configuration
    pub safety: SafetyConfigDto,
}

/// Boost profile data transfer object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoostProfileDto {
    /// Profile name
    pub name: String,
    
    /// Profile description
    pub description: String,
    
    /// Boost curve points
    pub boost_targets: Vec<BoostPointDto>,
    
    /// Maximum boost limit
    pub max_boost: f32,
    
    /// Overboost cut threshold
    pub overboost_limit: f32,
    
    /// Overboost hysteresis
    pub overboost_hysteresis: f32,
}

/// Boost curve point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoostPointDto {
    pub rpm: u16,
    pub boost_psi: f32,
}

/// Safety configuration DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfigDto {
    pub global_overboost_limit: f32,
    pub max_duty_change_per_cycle: f32,
    pub max_rpm: u16,
    pub min_rpm_for_arming: u16,
    pub can_timeout_ms: u64,
}

/// Sensor readings DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReadingsDto {
    pub dome_input_pressure: f32,
    pub upper_dome_pressure: f32,
    pub manifold_pressure_gauge: f32,
    pub timestamp_ms: u64,
}

/// CAN data DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanDataDto {
    pub rpm: u16,
    pub map_kpa: f32,
    pub desired_torque: f32,
    pub actual_torque: f32,
    pub throttle_position: Option<f32>,
    pub drive_mode: Option<String>,
    pub timestamp_ms: u64,
}

/// Control outputs DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputsDto {
    /// Current PWM duty cycle (%)
    pub duty_cycle: f32,
    
    /// Target boost pressure (PSI)
    pub target_boost: f32,
    
    /// PID controller output
    pub pid_output: f32,
    
    /// Current control mode
    pub control_mode: String,
}

/// Diagnostic data DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsDto {
    /// Hardware status
    pub hardware: HardwareDiagnosticsDto,
    
    /// Control loop performance
    pub performance: PerformanceDiagnosticsDto,
    
    /// Safety system status
    pub safety: SafetyDiagnosticsDto,
    
    /// Learning system status
    pub learning: LearningDiagnosticsDto,
}

/// Hardware diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDiagnosticsDto {
    pub pwm_status: String,
    pub adc_status: String,
    pub can_status: String,
    pub display_status: String,
    pub storage_status: String,
}

/// Performance diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDiagnosticsDto {
    pub control_loop_frequency: f32,
    pub max_loop_time_ms: f32,
    pub avg_loop_time_ms: f32,
    pub missed_cycles: u32,
}

/// Safety diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyDiagnosticsDto {
    pub overboost_events: u32,
    pub fault_events: u32,
    pub safety_cuts: u32,
    pub last_safety_event: Option<String>,
}

/// Learning system diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningDiagnosticsDto {
    pub calibration_points: u32,
    pub confidence_average: f32,
    pub last_learning_update: Option<u64>,
    pub learning_enabled: bool,
}

/// Real-time telemetry data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryDto {
    /// Timestamp
    pub timestamp_ms: u64,
    
    /// Sensor data
    pub sensors: SensorReadingsDto,
    
    /// CAN data
    pub can_data: CanDataDto,
    
    /// Control outputs
    pub outputs: OutputsDto,
    
    /// System state
    pub state: String,
}

impl Message {
    /// Create a new request message
    pub fn new_request(id: String, request: RequestPayload) -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
            id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            payload: MessagePayload::Request(request),
        }
    }
    
    /// Create a new response message
    pub fn new_response(id: String, response: ResponsePayload) -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
            id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            payload: MessagePayload::Response(response),
        }
    }
    
    /// Create a new notification message
    pub fn new_notification(notification: NotificationPayload) -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
            id: format!("notif-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            payload: MessagePayload::Notification(notification),
        }
    }
    
    /// Create a successful response
    pub fn ok_response(id: String, data: Option<ResponseData>) -> Self {
        Self::new_response(id, ResponsePayload::Ok { data })
    }
    
    /// Create an error response
    pub fn error_response(id: String, code: ErrorCode, message: String) -> Self {
        Self::new_response(id, ResponsePayload::Error {
            code,
            message,
            details: None,
        })
    }
}