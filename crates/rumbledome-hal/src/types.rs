//! Hardware abstraction layer type definitions

use serde::{Deserialize, Serialize};

/// PWM channel identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Last error timestamp
    pub last_error_ms: Option<u64>,
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

/// System input data combining all sensors and CAN
#[derive(Debug, Clone)]
pub struct SystemInputs {
    /// Pressure sensor readings
    pub sensors: SensorReadings,
    /// CAN bus data
    pub can: CanData,
    /// Current system time
    pub timestamp_ms: u64,
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