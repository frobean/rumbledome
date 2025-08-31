//! Hardware abstraction layer type definitions

use serde::{Deserialize, Serialize};

/// PWM channel identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PwmChannel {
    /// Main boost control solenoid (4-port MAC)
    BoostSolenoid,
}

/// Analog input channel identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalogChannel {
    /// Dome input pressure sensor (air supply)
    DomeInputPressure,
    /// Upper dome pressure sensor (wastegate actuation)
    UpperDomePressure,
    /// Manifold pressure sensor (boost measurement)
    ManifoldPressure,
}

/// GPIO pin identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioPin {
    /// Profile selection button
    ProfileButton,
    /// Scramble boost button
    ScrambleButton,
    /// Status LED (system operational)
    StatusLed,
    /// Fault LED (system fault condition)
    FaultLed,
}

/// CAN message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanMessage {
    /// CAN ID
    pub id: u32,
    /// Message data (up to 8 bytes)
    pub data: Vec<u8>,
    /// Extended frame format
    pub extended: bool,
    /// Remote transmission request
    pub rtr: bool,
}

/// CAN bus error statistics
#[derive(Debug, Clone, Default)]
pub struct CanErrorStats {
    /// Transmit error count
    pub tx_errors: u32,
    /// Receive error count  
    pub rx_errors: u32,
    /// Bus off events
    pub bus_off_count: u32,
    /// Error passive event count
    pub error_passive_count: u32,
    /// Last error timestamp
    pub last_error_ms: Option<u64>,
}

/// CAN health status levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum CanHealthStatus {
    Good,
    Warning,
    Poor,
    Critical,
}

/// Detailed CAN error statistics for diagnostics
#[derive(Debug, Clone)]
pub struct DetailedCanErrorStats {
    pub basic_stats: CanErrorStats,
    pub connection_status: bool,
    pub last_message_age_ms: Option<u64>,
    pub error_rate: f32, // Errors per second
}

/// Display color definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    /// Custom RGB color
    Rgb(u8, u8, u8),
}

/// Font size options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSize {
    Small,   // 8x8 pixels
    Medium,  // 12x12 pixels  
    Large,   // 16x16 pixels
}

/// System sensor readings
#[derive(Debug, Clone)]
pub struct SensorReadings {
    /// Dome input pressure in PSI (air supply)
    pub dome_input_pressure: f32,
    /// Upper dome pressure in PSI (wastegate control)
    pub upper_dome_pressure: f32,
    /// Manifold pressure in PSI gauge (boost measurement)
    pub manifold_pressure_gauge: f32,
    /// Timestamp of readings in milliseconds
    pub timestamp_ms: u64,
}

/// CAN bus torque and engine data
#[derive(Debug, Clone)]
pub struct CanData {
    /// Engine RPM
    pub rpm: u16,
    /// Manifold absolute pressure in kPa
    pub map_kpa: f32,
    /// ECU desired torque in Nm
    pub desired_torque: f32,
    /// ECU actual torque in Nm  
    pub actual_torque: f32,
    /// Throttle position percentage (0-100)
    pub throttle_position: Option<f32>,
    /// Drive mode (for Phase 2+)
    pub drive_mode: Option<DriveMode>,
    /// Timestamp of last update
    pub timestamp_ms: u64,
}

/// Vehicle drive mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriveMode {
    Normal,
    Sport,
    SportPlus,
    Track,
}

/// Display mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Gauge display with real-time boost/target readings
    Gauges,
    /// Full status display with detailed system information  
    Status,
    /// Diagnostic display for troubleshooting
    Diagnostics,
}

/// Gauge configuration parameters
#[derive(Debug, Clone)]
pub struct GaugeConfig {
    pub min_value: f32,
    pub max_value: f32,
    pub warning_threshold: f32,
    pub danger_threshold: f32,
    pub label: &'static str,
}

/// System input data combining all sensors and CAN
#[derive(Debug, Clone)]
pub struct SystemInputs {
    /// Pressure sensor readings
    pub sensors: SensorReadings,
    /// CAN bus data
    pub can: CanData,
    /// Combined manifold pressure (vacuum + boost range)
    pub combined_manifold_pressure: CombinedManifoldPressure,
    /// Current system time
    pub timestamp_ms: u64,
}

/// Combined manifold pressure from CAN MAP + boost gauge
#[derive(Debug, Clone)]
pub struct CombinedManifoldPressure {
    /// Full-range manifold pressure in PSI gauge (-14.7 to +30 PSI)
    pub pressure_gauge_psi: f32,
    /// Data source for current reading
    pub primary_source: ManifoldPressureSource,
    /// Sensor agreement status
    pub sensor_agreement: SensorAgreement,
    /// Transition zone handling
    pub in_transition_zone: bool,
}

/// Which sensor is providing primary manifold pressure reading
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ManifoldPressureSource {
    /// Using CAN MAP sensor (vacuum conditions)
    CanMapSensor,
    /// Using added boost gauge sensor (boost conditions) 
    BoostGaugeSensor,
    /// Blended reading during transition
    BlendedSensors,
}

/// Sensor agreement validation between CAN MAP and boost gauge
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorAgreement {
    /// Sensors agree within expected tolerance
    Good,
    /// Minor disagreement - use primary source
    MinorDisagreement,
    /// Major disagreement - possible sensor fault
    MajorDisagreement,
    /// Sensors outside overlap range - normal operation
    OutOfRange,
}

impl Default for SensorReadings {
    fn default() -> Self {
        Self {
            dome_input_pressure: 0.0,
            upper_dome_pressure: 0.0,
            manifold_pressure_gauge: 0.0,
            timestamp_ms: 0,
        }
    }
}

impl Default for CanData {
    fn default() -> Self {
        Self {
            rpm: 0,
            map_kpa: 101.3, // Atmospheric pressure
            desired_torque: 0.0,
            actual_torque: 0.0,
            throttle_position: None,
            drive_mode: None,
            timestamp_ms: 0,
        }
    }
}