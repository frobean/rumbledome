//! Hardware abstraction traits for RumbleDome
//! 
//! These traits define the interface between the core control logic
//! and platform-specific hardware implementations.

use crate::types::*;
use crate::error::HalError;

/// Main hardware abstraction trait
/// 
/// Combines all hardware subsystem traits into a single interface
/// that the core control logic can use.
pub trait HalTrait: Send + Sync {
    type Time: TimeProvider;
    type Pwm: PwmController;
    type Analog: AnalogReader;
    type Storage: NonVolatileStorage;
    type Can: CanBus;
    type Display: DisplayController;
    type Gpio: GpioController;
    type Watchdog: WatchdogTimer;
}

/// Time and timing services
pub trait TimeProvider: Send + Sync {
    /// Get current system time in milliseconds since boot
    fn now_ms(&self) -> u64;
    
    /// Delay execution for specified milliseconds
    fn delay_ms(&mut self, ms: u32);
    
    /// Get high-resolution timestamp for performance measurement
    fn timestamp_us(&self) -> u64;
}

/// PWM output control for solenoids
pub trait PwmController: Send + Sync {
    /// Set PWM duty cycle as percentage (0.0 to 100.0)
    /// 
    /// # Safety
    /// 0% duty cycle MUST result in wastegate forced open (failsafe)
    fn set_duty_cycle(&mut self, channel: PwmChannel, duty_percent: f32) -> Result<(), HalError>;
    
    /// Set PWM frequency in Hz
    fn set_frequency(&mut self, channel: PwmChannel, frequency_hz: u16) -> Result<(), HalError>;
    
    /// Enable PWM output
    fn enable(&mut self, channel: PwmChannel) -> Result<(), HalError>;
    
    /// Disable PWM output (failsafe state)
    fn disable(&mut self, channel: PwmChannel) -> Result<(), HalError>;
    
    /// Get current duty cycle setting
    fn get_duty_cycle(&self, channel: PwmChannel) -> Result<f32, HalError>;
}

/// Analog sensor reading
pub trait AnalogReader: Send + Sync {
    /// Read raw ADC value (implementation-specific range)
    fn read_raw(&mut self, channel: AnalogChannel) -> Result<u16, HalError>;
    
    /// Read voltage in volts
    fn read_voltage(&mut self, channel: AnalogChannel) -> Result<f32, HalError>;
    
    /// Read calibrated pressure in PSI
    /// 
    /// Applies sensor-specific calibration curve:
    /// PSI = ((Vout - 0.5V) / 4.0V) * 30 PSI for 0-30 PSI sensors
    fn read_pressure_psi(&mut self, channel: AnalogChannel) -> Result<f32, HalError>;
}

/// Non-volatile storage for configuration and learned data
pub trait NonVolatileStorage: Send + Sync {
    /// Read data from storage
    fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), HalError>;
    
    /// Write data to storage
    fn write(&mut self, address: u32, data: &[u8]) -> Result<(), HalError>;
    
    /// Erase storage sector
    fn erase_sector(&mut self, address: u32) -> Result<(), HalError>;
    
    /// Get storage capacity in bytes
    fn capacity(&self) -> u32;
    
    /// Perform wear leveling maintenance
    fn maintenance(&mut self) -> Result<(), HalError>;
}

/// CAN bus communication
pub trait CanBus: Send + Sync {
    /// Send CAN message
    fn send(&mut self, message: &CanMessage) -> Result<(), HalError>;
    
    /// Receive CAN message (non-blocking)
    fn receive(&mut self) -> Result<Option<CanMessage>, HalError>;
    
    /// Check if CAN bus is connected and operational
    fn is_connected(&self) -> bool;
    
    /// Get bus error statistics
    fn get_error_stats(&self) -> CanErrorStats;
    
    /// Reset CAN controller
    fn reset(&mut self) -> Result<(), HalError>;
}

/// Display control
pub trait DisplayController: Send + Sync {
    /// Clear display
    fn clear(&mut self) -> Result<(), HalError>;
    
    /// Draw text at specified position
    fn draw_text(&mut self, x: u16, y: u16, text: &str, font_size: FontSize) -> Result<(), HalError>;
    
    /// Draw filled rectangle
    fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: Color) -> Result<(), HalError>;
    
    /// Draw line
    fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: Color) -> Result<(), HalError>;
    
    /// Draw circle (for gauge displays)
    fn draw_circle(&mut self, x: u16, y: u16, radius: u16, color: Color) -> Result<(), HalError>;
    
    /// Update display (refresh from framebuffer)
    fn update(&mut self) -> Result<(), HalError>;
    
    /// Set display brightness (0-100%)
    fn set_brightness(&mut self, percent: u8) -> Result<(), HalError>;
}

/// GPIO control for buttons and status LEDs
pub trait GpioController: Send + Sync {
    /// Read digital input pin
    fn read_pin(&mut self, pin: GpioPin) -> Result<bool, HalError>;
    
    /// Write digital output pin
    fn write_pin(&mut self, pin: GpioPin, state: bool) -> Result<(), HalError>;
    
    /// Configure pin as input with pull-up
    fn set_input_pullup(&mut self, pin: GpioPin) -> Result<(), HalError>;
    
    /// Configure pin as output
    fn set_output(&mut self, pin: GpioPin) -> Result<(), HalError>;
}

/// Watchdog timer for safety monitoring
pub trait WatchdogTimer: Send + Sync {
    /// Start watchdog with specified timeout in milliseconds
    fn start(&mut self, timeout_ms: u32) -> Result<(), HalError>;
    
    /// Feed/kick the watchdog to reset timeout
    fn feed(&mut self) -> Result<(), HalError>;
    
    /// Stop watchdog timer
    fn stop(&mut self) -> Result<(), HalError>;
    
    /// Check if watchdog is running
    fn is_running(&self) -> bool;
}

/// Bluetooth Serial Port Profile interface
/// 
/// Provides wireless serial access that appears identical to USB-C console
/// from the microcontroller's perspective. Mobile apps send the same CLI
/// commands and receive the same responses as direct USB connection.
pub trait BluetoothSerial: Send + Sync {
    /// Check if a device is currently connected
    fn is_connected(&self) -> bool;
    
    /// Send console output to connected device
    /// This is called by the console/CLI system to send responses
    fn send_console_output(&mut self, data: &[u8]) -> Result<(), HalError>;
    
    /// Receive console input from connected device
    /// Returns command data from mobile app (same format as USB input)
    fn receive_console_input(&mut self) -> Result<Option<Vec<u8>>, HalError>;
    
    /// Get human-readable connection information
    fn get_connection_info(&self) -> Result<String, HalError>;
    
    /// Disconnect current device
    fn disconnect_device(&mut self) -> Result<(), HalError>;
}

/// System backup and restore for microcontroller replacement
pub trait SystemBackup {
    /// Create complete system backup including all configuration and learned data
    fn create_full_backup(&mut self) -> Result<SystemBackupData, HalError>;
    
    /// Restore complete system state from backup data
    fn restore_from_backup(&mut self, backup: &SystemBackupData) -> Result<RestoreResult, HalError>;
    
    /// Verify backup integrity and compatibility
    fn verify_backup(&self, backup: &SystemBackupData) -> Result<BackupVerification, HalError>;
    
    /// Get system identification for backup compatibility checking
    fn get_system_info(&self) -> SystemInfo;
}

/// MicroSD card storage for portable, hardware-agnostic configuration
pub trait PortableStorage {
    /// Mount microSD card and verify filesystem
    fn mount(&mut self) -> Result<(), HalError>;
    
    /// Unmount microSD card safely
    fn unmount(&mut self) -> Result<(), HalError>;
    
    /// Check if microSD card is present and mounted
    fn is_mounted(&self) -> bool;
    
    /// Read portable configuration file
    fn read_config_file(&mut self, filename: &str) -> Result<Vec<u8>, HalError>;
    
    /// Write portable configuration file
    fn write_config_file(&mut self, filename: &str, data: &[u8]) -> Result<(), HalError>;
    
    /// List available configuration files
    fn list_config_files(&mut self) -> Result<Vec<ConfigFileInfo>, HalError>;
    
    /// Load user profiles from SD card
    fn load_user_profiles(&mut self) -> Result<UserProfileSet, HalError>;
    
    /// Save user profiles to SD card
    fn save_user_profiles(&mut self, profiles: &UserProfileSet) -> Result<(), HalError>;
    
    /// Get SD card info and health status
    fn get_card_info(&self) -> Result<SdCardInfo, HalError>;
}

/// Configuration file information
#[derive(Debug, Clone)]
pub struct ConfigFileInfo {
    pub filename: String,
    pub size_bytes: usize,
    pub created_timestamp: u64,
    pub modified_timestamp: u64,
    pub file_type: ConfigFileType,
    pub description: String,
}

/// Types of configuration files
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigFileType {
    UserProfiles,        // Boost profiles (daily, sport, track, etc.)
    SensorCalibration,   // Pressure sensor parameters
    SafetyLimits,        // User-defined safety boundaries
    SystemConfig,        // Display, CAN, general settings
    BackupArchive,       // Complete system backup
    FirmwareUpdate,      // Firmware update package
    ProfileLibrary,      // Shared profile collection
}

/// Portable user profile set
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserProfileSet {
    pub metadata: ProfileSetMetadata,
    pub profiles: Vec<BoostProfile>,
    pub sensor_calibrations: SensorCalibrations,
    pub safety_limits: UserSafetyLimits,
    pub system_preferences: SystemPreferences,
}

/// Profile set metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfileSetMetadata {
    pub format_version: String,
    pub created_timestamp: u64,
    pub modified_timestamp: u64,
    pub author: String,
    pub vehicle_info: VehicleInfo,
    pub description: String,
    pub profile_count: usize,
}

/// Vehicle-specific information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VehicleInfo {
    pub year: Option<u16>,
    pub make: String,
    pub model: String,
    pub engine: String,
    pub turbo_system: String,
    pub notes: String,
}

/// Boost profile definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoostProfile {
    pub name: String,
    pub description: String,
    pub max_boost_psi: f32,
    pub torque_target_percentage: f32,
    pub aggressiveness: f32,
    pub safety_margin: f32,
    pub environmental_adaptation: bool,
    pub learning_enabled: bool,
    pub tags: Vec<String>, // "daily", "track", "winter", etc.
}

/// Sensor calibration parameters
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SensorCalibrations {
    pub dome_input_pressure: PressureSensorCalibration,
    pub upper_dome_pressure: PressureSensorCalibration,
    pub manifold_pressure: PressureSensorCalibration,
}

/// Individual pressure sensor calibration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PressureSensorCalibration {
    pub zero_voltage: f32,      // 0 PSI voltage
    pub full_scale_voltage: f32, // Max PSI voltage  
    pub full_scale_pressure: f32, // Max PSI reading
    pub sensor_model: String,
    pub calibration_date: u64,
    pub notes: String,
}

/// User-defined safety limits
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserSafetyLimits {
    pub absolute_max_boost: f32,
    pub overboost_threshold: f32,
    pub overboost_duration_ms: u32,
    pub max_duty_cycle: f32,
    pub min_manifold_pressure: f32,
    pub max_slew_rate: f32,
}

/// System preferences and UI settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemPreferences {
    pub display_brightness: u8,
    pub gauge_style: String,
    pub units: UnitPreferences,
    pub can_bus_settings: CanBusPreferences,
    pub logging_level: String,
}

/// Unit preferences
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnitPreferences {
    pub pressure_units: String, // "psi", "bar", "kpa"
    pub temperature_units: String, // "celsius", "fahrenheit"
    pub boost_display_precision: u8, // decimal places
}

/// CAN bus preferences
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CanBusPreferences {
    pub enable_torque_cooperation: bool,
    pub ecu_message_timeout_ms: u32,
    pub custom_can_ids: Vec<u32>,
    pub message_filters: Vec<String>,
}

/// MicroSD card information
#[derive(Debug, Clone)]
pub struct SdCardInfo {
    pub capacity_gb: f32,
    pub free_space_gb: f32,
    pub filesystem_type: String,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub write_speed_class: u8,
    pub health_status: SdCardHealth,
}

/// SD card health status
#[derive(Debug, Clone, PartialEq)]
pub enum SdCardHealth {
    Excellent,
    Good,
    Warning,    // Getting old, consider replacement
    Critical,   // Frequent errors, replace soon
    Failed,     // Card failure detected
}

/// Complete system backup data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemBackupData {
    /// Backup metadata and version info
    pub metadata: BackupMetadata,
    
    /// User configuration data
    pub user_config: Vec<u8>,
    
    /// Learned calibration data
    pub learned_data: Vec<u8>,
    
    /// Auto-calibration state and progress
    pub calibration_state: Vec<u8>,
    
    /// Safety event log (for analysis)
    pub safety_log: Vec<u8>,
    
    /// Storage wear tracking data (for new micro)
    pub wear_tracking: WearTrackingBackup,
    
    /// System statistics and history
    pub system_stats: SystemStatsBackup,
    
    /// Backup integrity checksum
    pub checksum: u32,
}

/// Backup metadata for version compatibility and validation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupMetadata {
    /// Backup format version
    pub backup_version: String,
    
    /// Firmware version that created this backup
    pub firmware_version: String,
    
    /// Hardware platform (e.g., "teensy41", "stm32f4")
    pub hardware_platform: String,
    
    /// Backup creation timestamp
    pub created_timestamp: u64,
    
    /// Source system serial number/ID
    pub source_system_id: String,
    
    /// Human-readable backup description
    pub description: String,
    
    /// Backup size in bytes
    pub total_size: usize,
}

/// Wear tracking data for new microcontroller
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WearTrackingBackup {
    /// Note: New micro starts fresh, but we preserve historical data for analysis
    pub previous_write_counts: [u32; 8],
    pub total_lifetime_writes: u32,
    pub total_data_written_kb: f32,
    pub estimated_previous_lifespan_years: f32,
    pub replacement_reason: String, // "planned", "failure", "development", etc.
}

/// System statistics backup for continuity
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemStatsBackup {
    /// Total runtime hours across all micros
    pub cumulative_runtime_hours: f32,
    
    /// Total learning sessions completed
    pub total_learning_sessions: u32,
    
    /// Successful calibration runs
    pub successful_calibrations: u32,
    
    /// Safety events encountered
    pub safety_event_count: u32,
    
    /// Performance metrics
    pub average_control_loop_time_us: f32,
}

/// Result of backup restoration
#[derive(Debug, Clone)]
pub struct RestoreResult {
    /// Overall restoration success
    pub success: bool,
    
    /// Detailed results per section
    pub section_results: RestoreSectionResults,
    
    /// Warnings or compatibility notes
    pub warnings: Vec<String>,
    
    /// Actions required after restore
    pub required_actions: Vec<String>,
    
    /// Data migration notes
    pub migration_notes: Vec<String>,
}

/// Restoration results per data section
#[derive(Debug, Clone)]
pub struct RestoreSectionResults {
    pub user_config: RestoreStatus,
    pub learned_data: RestoreStatus,
    pub calibration_state: RestoreStatus,
    pub safety_log: RestoreStatus,
    pub system_stats: RestoreStatus,
}

/// Status of individual section restore
#[derive(Debug, Clone, PartialEq)]
pub enum RestoreStatus {
    Success,
    SuccessWithWarnings,
    PartialRestore, // Some data restored, some skipped due to compatibility
    Failed,
    Skipped, // User chose to skip this section
}

/// Backup verification results
#[derive(Debug, Clone)]
pub struct BackupVerification {
    /// Overall backup validity
    pub is_valid: bool,
    
    /// Checksum verification
    pub checksum_valid: bool,
    
    /// Version compatibility
    pub version_compatible: bool,
    
    /// Hardware platform compatibility  
    pub hardware_compatible: bool,
    
    /// Detailed compatibility analysis
    pub compatibility_report: CompatibilityReport,
    
    /// Detected issues or warnings
    pub issues: Vec<String>,
}

/// Hardware/software compatibility analysis
#[derive(Debug, Clone)]
pub struct CompatibilityReport {
    /// Can learned data be directly restored?
    pub learned_data_compatible: bool,
    
    /// Can user config be directly restored?
    pub config_compatible: bool,
    
    /// Firmware version differences
    pub version_delta: VersionDelta,
    
    /// Required data migrations
    pub required_migrations: Vec<String>,
}

/// Version difference analysis
#[derive(Debug, Clone)]
pub struct VersionDelta {
    pub major_version_change: bool,
    pub minor_version_change: bool,
    pub patch_version_change: bool,
    pub breaking_changes: Vec<String>,
}

/// System identification information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hardware_platform: String,
    pub firmware_version: String,
    pub system_serial: String,
    pub flash_size: usize,
    pub ram_size: usize,
    pub features: Vec<String>,
}