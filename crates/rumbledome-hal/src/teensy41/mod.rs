//! Teensy 4.1 hardware implementation
//! 
//! Provides concrete implementation of HAL traits for Teensy 4.1 hardware
//! based on the i.MX RT1062 microcontroller.

use crate::traits::*;
use crate::types::*;
use crate::error::HalError;

use teensy4_bsp::{board, hal};
use hal::ral;

mod pwm;
mod analog;  
mod storage;
mod can;
mod display;
mod gpio;
mod time;
mod sdcard;
mod bluetooth;

pub use pwm::Teensy41Pwm;
pub use analog::Teensy41Analog;
pub use storage::{Teensy41Storage, StorageHealthReport, StorageHealth, WearTrackingData};
pub use can::{Teensy41Can, CoyoteCanParser, CoyoteEngineData};
pub use display::Teensy41Display;
pub use gpio::Teensy41Gpio;
pub use time::Teensy41Time;
pub use sdcard::Teensy41SdCard;
pub use bluetooth::Teensy41Bluetooth;

/// Main Teensy 4.1 HAL implementation
pub struct Teensy41Hal {
    /// Time provider
    pub time: Teensy41Time,
    
    /// PWM controller
    pub pwm: Teensy41Pwm,
    
    /// Analog input reader
    pub analog: Teensy41Analog,
    
    /// Non-volatile storage
    pub storage: Teensy41Storage,
    
    /// CAN bus controller
    pub can: Teensy41Can,
    
    /// Display controller
    pub display: Teensy41Display,
    
    /// GPIO controller
    pub gpio: Teensy41Gpio,
    
    /// Watchdog timer
    pub watchdog: Teensy41Watchdog,
    
    /// MicroSD card storage
    pub sdcard: Teensy41SdCard,
    
    /// Bluetooth serial interface
    pub bluetooth: Teensy41Bluetooth,
}

/// Teensy 4.1 watchdog implementation
pub struct Teensy41Watchdog {
    /// Watchdog timer instance
    wdog: hal::wdog::Wdog<1>,
    
    /// Watchdog enabled state
    enabled: bool,
    
    /// Timeout value in milliseconds
    timeout_ms: u32,
}

impl Teensy41Hal {
    /// Create new Teensy 4.1 HAL instance
    pub fn new(
        pins: board::t41::Pins,
        _usb: board::Usb,
        display_spi: hal::spi::Spi<hal::spi::module::_4>,
        sdcard_spi: hal::spi::Spi<hal::spi::module::_1>,
        bluetooth_uart: hal::uart::Uart<hal::uart::module::_2>,
        ccm: hal::ccm::Handle,
    ) -> Result<Self, HalError> {
        
        log::info!("Initializing Teensy 4.1 HAL");
        
        // Initialize time provider
        let time = Teensy41Time::new();
        
        // Split pins for different peripherals
        let (gpio_pins, display_pins, pwm_pins, sdcard_pins) = Self::split_pins(pins);
        
        // Initialize PWM for solenoid control
        let pwm = Teensy41Pwm::new(pwm_pins, &ccm)
            .map_err(|e| HalError::initialization(format!("PWM init failed: {:?}", e)))?;
        
        // Initialize ADC for pressure sensors
        let analog = Teensy41Analog::new()
            .map_err(|e| HalError::initialization(format!("ADC init failed: {:?}", e)))?;
        
        // Initialize EEPROM emulation for storage
        let storage = Teensy41Storage::new()
            .map_err(|e| HalError::initialization(format!("Storage init failed: {:?}", e)))?;
        
        // Initialize CAN bus
        let can = Teensy41Can::new()
            .map_err(|e| HalError::initialization(format!("CAN init failed: {:?}", e)))?;
        
        // Initialize SPI display
        let (dc_pin, rst_pin) = display_pins;
        let display = Teensy41Display::new(display_spi, dc_pin, rst_pin)
            .map_err(|e| HalError::initialization(format!("Display init failed: {:?}", e)))?;
        
        // Initialize GPIO for buttons and LEDs
        let gpio = Teensy41Gpio::new(gpio_pins)
            .map_err(|e| HalError::initialization(format!("GPIO init failed: {:?}", e)))?;
        
        // Initialize microSD card
        let (cs_pin, cd_pin) = sdcard_pins;
        let sdcard = Teensy41SdCard::new(sdcard_spi, cs_pin, cd_pin)
            .map_err(|e| HalError::initialization(format!("SD card init failed: {:?}", e)))?;
        
        // Initialize Bluetooth serial interface
        let bluetooth = Teensy41Bluetooth::new(bluetooth_uart)
            .map_err(|e| HalError::initialization(format!("Bluetooth init failed: {:?}", e)))?;
        
        // Initialize watchdog
        let watchdog = Teensy41Watchdog::new()
            .map_err(|e| HalError::initialization(format!("Watchdog init failed: {:?}", e)))?;
        
        log::info!("Teensy 4.1 HAL initialized successfully");
        
        Ok(Self {
            time,
            pwm,
            analog,
            storage,
            can,
            display,
            gpio,
            watchdog,
            sdcard,
            bluetooth,
        })
    }
    
    /// Split pins for different peripheral requirements
    /// This is a placeholder - actual implementation would properly split pins
    fn split_pins(pins: board::t41::Pins) -> (board::t41::Pins, (hal::gpio::Output, hal::gpio::Output), board::t41::Pins, (hal::gpio::Output, Option<hal::gpio::Input>)) {
        // In a real implementation, this would properly distribute pins to different peripherals
        // For now, we'll use unsafe cloning as a placeholder
        let gpio_pins = unsafe { core::ptr::read(&pins) };
        let pwm_pins = unsafe { core::ptr::read(&pins) };
        
        // Create dummy display control pins (would be real pins in actual implementation)
        let dc_pin = unsafe { hal::gpio::Output::new() };
        let rst_pin = unsafe { hal::gpio::Output::new() };
        
        // Create dummy SD card pins (CS and optional card detect)
        let cs_pin = unsafe { hal::gpio::Output::new() };
        let cd_pin = Some(unsafe { hal::gpio::Input::new() });
        
        (gpio_pins, (dc_pin, rst_pin), pwm_pins, (cs_pin, cd_pin))
    }
}

impl HalTrait for Teensy41Hal {
    type Time = Teensy41Time;
    type Pwm = Teensy41Pwm;
    type Analog = Teensy41Analog;
    type Storage = Teensy41Storage;
    type Can = Teensy41Can;
    type Display = Teensy41Display;
    type Gpio = Teensy41Gpio;
    type Watchdog = Teensy41Watchdog;
}

impl Teensy41Watchdog {
    /// Create new watchdog timer
    pub fn new() -> Result<Self, HalError> {
        // TODO: Initialize hardware watchdog
        // For now, create a software-only implementation
        
        Ok(Self {
            wdog: unsafe { hal::wdog::Wdog::new(ral::wdog::WDOG1::steal()) },
            enabled: false,
            timeout_ms: 1000,
        })
    }
}

impl SystemBackup for Teensy41Hal {
    fn create_full_backup(&mut self) -> Result<SystemBackupData, HalError> {
        log::info!("Creating full system backup...");
        
        let current_time = self.time.now_ms();
        
        // Read all storage sections
        let mut user_config = vec![0u8; 512];
        let mut learned_data = vec![0u8; 2048]; 
        let mut calibration_state = vec![0u8; 1024];
        let mut safety_log = vec![0u8; 512];
        
        self.storage.read(0, &mut user_config)?;
        self.storage.read(512, &mut learned_data)?;
        self.storage.read(2560, &mut calibration_state)?;
        self.storage.read(3584, &mut safety_log)?;
        
        // Get storage health for wear tracking backup
        let health_report = self.storage.get_health_report(current_time);
        
        let wear_tracking = WearTrackingBackup {
            previous_write_counts: health_report.write_statistics.total_writes_lifetime,
            total_lifetime_writes: health_report.write_statistics.total_writes_lifetime,
            total_data_written_kb: health_report.write_statistics.total_data_written_kb,
            estimated_previous_lifespan_years: health_report.estimated_lifespan_years,
            replacement_reason: "manual_backup".to_string(),
        };
        
        // Create system stats backup
        let system_stats = SystemStatsBackup {
            cumulative_runtime_hours: health_report.write_statistics.uptime_hours,
            total_learning_sessions: 0, // TODO: Track this in system stats
            successful_calibrations: 0, // TODO: Track this in system stats
            safety_event_count: 0, // TODO: Track this in system stats
            average_control_loop_time_us: 10000.0, // 100Hz = 10ms = 10000us
        };
        
        // Generate system ID based on unique chip characteristics
        let system_id = format!("teensy41-{:08x}", 
            unsafe { *(0x401F4410 as *const u32) }); // i.MX RT1062 unique ID register
        
        let metadata = BackupMetadata {
            backup_version: "1.0.0".to_string(),
            firmware_version: env!("CARGO_PKG_VERSION").to_string(),
            hardware_platform: "teensy41".to_string(),
            created_timestamp: current_time,
            source_system_id: system_id,
            description: format!("Full system backup created at {}", current_time),
            total_size: user_config.len() + learned_data.len() + calibration_state.len() + safety_log.len(),
        };
        
        // Calculate backup checksum
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&user_config);
        hasher.update(&learned_data);
        hasher.update(&calibration_state);
        hasher.update(&safety_log);
        let checksum = hasher.finalize();
        
        let backup = SystemBackupData {
            metadata,
            user_config,
            learned_data,
            calibration_state,
            safety_log,
            wear_tracking,
            system_stats,
            checksum,
        };
        
        log::info!("System backup created successfully ({} bytes)", backup.metadata.total_size);
        Ok(backup)
    }
    
    fn restore_from_backup(&mut self, backup: &SystemBackupData) -> Result<RestoreResult, HalError> {
        log::info!("Starting system restore from backup...");
        
        // Verify backup first
        let verification = self.verify_backup(backup)?;
        if !verification.is_valid {
            return Err(HalError::invalid_parameter("Backup verification failed"));
        }
        
        let mut section_results = RestoreSectionResults {
            user_config: RestoreStatus::Failed,
            learned_data: RestoreStatus::Failed,
            calibration_state: RestoreStatus::Failed,
            safety_log: RestoreStatus::Failed,
            system_stats: RestoreStatus::Failed,
        };
        
        let mut warnings = Vec::new();
        let mut required_actions = Vec::new();
        let mut migration_notes = Vec::new();
        
        // Restore user configuration
        match self.storage.write(0, &backup.user_config) {
            Ok(()) => {
                section_results.user_config = RestoreStatus::Success;
                log::info!("User configuration restored successfully");
            },
            Err(e) => {
                warnings.push(format!("Failed to restore user config: {:?}", e));
                section_results.user_config = RestoreStatus::Failed;
            }
        }
        
        // Restore learned data
        if verification.compatibility_report.learned_data_compatible {
            match self.storage.write(512, &backup.learned_data) {
                Ok(()) => {
                    section_results.learned_data = RestoreStatus::Success;
                    log::info!("Learned data restored successfully");
                },
                Err(e) => {
                    warnings.push(format!("Failed to restore learned data: {:?}", e));
                    section_results.learned_data = RestoreStatus::Failed;
                }
            }
        } else {
            section_results.learned_data = RestoreStatus::Skipped;
            warnings.push("Learned data skipped due to compatibility issues".to_string());
            migration_notes.push("Re-calibration recommended after restore".to_string());
        }
        
        // Restore calibration state
        match self.storage.write(2560, &backup.calibration_state) {
            Ok(()) => {
                section_results.calibration_state = RestoreStatus::Success;
                log::info!("Calibration state restored successfully");
            },
            Err(e) => {
                warnings.push(format!("Failed to restore calibration state: {:?}", e));
                section_results.calibration_state = RestoreStatus::Failed;
            }
        }
        
        // Restore safety log (for analysis continuity)
        match self.storage.write(3584, &backup.safety_log) {
            Ok(()) => {
                section_results.safety_log = RestoreStatus::Success;
                log::info!("Safety log restored successfully");
            },
            Err(e) => {
                warnings.push(format!("Failed to restore safety log: {:?}", e));
                section_results.safety_log = RestoreStatus::Failed;
            }
        }
        
        // System stats are informational only
        section_results.system_stats = RestoreStatus::Success;
        
        // Add standard post-restore actions
        required_actions.push("Restart system to activate restored configuration".to_string());
        required_actions.push("Verify all sensors are calibrated correctly".to_string());
        required_actions.push("Test system operation in safe environment".to_string());
        
        if backup.metadata.hardware_platform != "teensy41" {
            required_actions.push("Hardware platform mismatch - manual configuration review required".to_string());
        }
        
        let success = section_results.user_config == RestoreStatus::Success ||
                     section_results.learned_data == RestoreStatus::Success;
        
        let result = RestoreResult {
            success,
            section_results,
            warnings,
            required_actions,
            migration_notes,
        };
        
        log::info!("System restore completed with success: {}", result.success);
        Ok(result)
    }
    
    fn verify_backup(&self, backup: &SystemBackupData) -> Result<BackupVerification, HalError> {
        let mut is_valid = true;
        let mut issues = Vec::new();
        
        // Verify checksum
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&backup.user_config);
        hasher.update(&backup.learned_data);
        hasher.update(&backup.calibration_state);
        hasher.update(&backup.safety_log);
        let calculated_checksum = hasher.finalize();
        
        let checksum_valid = calculated_checksum == backup.checksum;
        if !checksum_valid {
            is_valid = false;
            issues.push("Backup checksum verification failed - data may be corrupted".to_string());
        }
        
        // Version compatibility check
        let current_version = env!("CARGO_PKG_VERSION");
        let version_compatible = backup.metadata.firmware_version == current_version;
        if !version_compatible {
            issues.push(format!("Firmware version mismatch: backup {} vs current {}", 
                backup.metadata.firmware_version, current_version));
        }
        
        // Hardware compatibility
        let hardware_compatible = backup.metadata.hardware_platform == "teensy41";
        if !hardware_compatible {
            is_valid = false;
            issues.push(format!("Hardware platform mismatch: backup {} vs teensy41", 
                backup.metadata.hardware_platform));
        }
        
        // Analyze version differences
        let version_delta = VersionDelta {
            major_version_change: false, // TODO: Implement semantic version parsing
            minor_version_change: !version_compatible,
            patch_version_change: !version_compatible,
            breaking_changes: Vec::new(),
        };
        
        let compatibility_report = CompatibilityReport {
            learned_data_compatible: version_compatible && hardware_compatible,
            config_compatible: hardware_compatible,
            version_delta,
            required_migrations: Vec::new(),
        };
        
        Ok(BackupVerification {
            is_valid,
            checksum_valid,
            version_compatible,
            hardware_compatible,
            compatibility_report,
            issues,
        })
    }
    
    fn get_system_info(&self) -> SystemInfo {
        let system_id = format!("teensy41-{:08x}", 
            unsafe { *(0x401F4410 as *const u32) }); // i.MX RT1062 unique ID
            
        SystemInfo {
            hardware_platform: "teensy41".to_string(),
            firmware_version: env!("CARGO_PKG_VERSION").to_string(),
            system_serial: system_id,
            flash_size: 8 * 1024 * 1024, // 8MB
            ram_size: 1024 * 1024, // 1MB
            features: vec![
                "flexcan".to_string(),
                "flexpwm".to_string(),
                "eeprom_emulation".to_string(),
                "dual_adc".to_string(),
                "st7735_display".to_string(),
            ],
        }
    }
}

impl WatchdogTimer for Teensy41Watchdog {
    fn start(&mut self, timeout_ms: u32) -> Result<(), HalError> {
        self.timeout_ms = timeout_ms;
        
        // Configure watchdog timer
        self.wdog.set_timeout_ms(timeout_ms as u16);
        self.wdog.enable();
        
        self.enabled = true;
        log::debug!("Watchdog started with {} ms timeout", timeout_ms);
        
        Ok(())
    }
    
    fn feed(&mut self) -> Result<(), HalError> {
        if self.enabled {
            self.wdog.feed();
            log::trace!("Watchdog fed");
        }
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), HalError> {
        self.wdog.disable();
        self.enabled = false;
        log::debug!("Watchdog stopped");
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.enabled
    }
}