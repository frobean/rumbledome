//! MicroSD card storage implementation for Teensy 4.1
//!
//! Provides portable configuration storage using the built-in microSD card slot
//! on the Teensy 4.1. This storage is hardware-independent and can be moved
//! between different microcontrollers.

use crate::traits::{
    PortableStorage, ConfigFileInfo, ConfigFileType, UserProfileSet, 
    SdCardInfo, SdCardHealth, ProfileSetMetadata, VehicleInfo,
    BoostProfile, SensorCalibrations, PressureSensorCalibration,
    UserSafetyLimits, SystemPreferences, UnitPreferences, CanBusPreferences
};
use crate::error::HalError;

use teensy4_bsp::hal;
use serde_json;
use core::fmt::Write;
use heapless::{String, Vec as HeaplessVec};

/// Maximum filename length for SD card files
const MAX_FILENAME_LEN: usize = 32;

/// Maximum file count for directory listings
const MAX_FILE_COUNT: usize = 64;

/// Standard directory paths on SD card
const PROFILES_DIR: &str = "/RUMBLEDOME/profiles";
const CONFIG_DIR: &str = "/RUMBLEDOME/config";  
const BACKUPS_DIR: &str = "/RUMBLEDOME/backups";
const LOGS_DIR: &str = "/RUMBLEDOME/logs";
const FIRMWARE_DIR: &str = "/RUMBLEDOME/firmware";

/// Teensy 4.1 microSD card implementation
pub struct Teensy41SdCard {
    /// SD card interface
    sdcard: SdCardInterface,
    
    /// Mount status
    mounted: bool,
    
    /// Card information
    card_info: Option<SdCardInfo>,
    
    /// File system buffer
    fs_buffer: [u8; 4096],
    
    /// Current working directory  
    current_dir: String<64>,
}

/// SD card hardware interface abstraction
struct SdCardInterface {
    /// SPI interface for SD card communication
    spi: hal::spi::Spi<hal::spi::module::_4>,
    
    /// Chip select pin for SD card
    cs_pin: hal::gpio::Output,
    
    /// Card detect pin (if available)
    cd_pin: Option<hal::gpio::Input>,
}

impl Teensy41SdCard {
    /// Create new SD card controller
    pub fn new(
        spi: hal::spi::Spi<hal::spi::module::_4>,
        cs_pin: hal::gpio::Output,
        cd_pin: Option<hal::gpio::Input>,
    ) -> Result<Self, HalError> {
        
        let sdcard = SdCardInterface {
            spi,
            cs_pin,
            cd_pin,
        };
        
        log::info!("SD card interface initialized");
        
        Ok(Self {
            sdcard,
            mounted: false,
            card_info: None,
            fs_buffer: [0u8; 4096],
            current_dir: String::new(),
        })
    }
    
    /// Check if SD card is physically present
    pub fn card_present(&self) -> bool {
        match &self.sdcard.cd_pin {
            Some(pin) => !pin.is_set(), // Card detect is active low
            None => true, // Assume present if no detect pin
        }
    }
    
    /// Initialize SD card filesystem
    fn init_filesystem(&mut self) -> Result<(), HalError> {
        if !self.card_present() {
            return Err(HalError::device_not_found("SD card not present"));
        }
        
        // Initialize SD card low-level interface
        self.init_spi_interface()?;
        
        // Perform SD card initialization sequence
        self.sd_card_init_sequence()?;
        
        // Initialize filesystem (FAT32)
        self.init_fat32_filesystem()?;
        
        // Create standard directory structure
        self.create_directory_structure()?;
        
        // Read card information
        self.card_info = Some(self.read_card_info()?);
        
        log::info!("SD card filesystem initialized");
        Ok(())
    }
    
    /// Initialize SPI interface for SD card communication
    fn init_spi_interface(&mut self) -> Result<(), HalError> {
        // Set CS high (inactive)
        self.sdcard.cs_pin.set_high();
        
        // Send 80+ clock cycles with CS high to allow card to initialize
        // This is required by SD card spec before first command
        for _ in 0..10 {
            self.spi_write_byte(0xFF)?;
        }
        
        log::trace!("SD card SPI interface initialized");
        Ok(())
    }
    
    /// Perform SD card initialization sequence per SD card specification
    fn sd_card_init_sequence(&mut self) -> Result<(), HalError> {
        // Send CMD0 (GO_IDLE_STATE) to reset card
        let response = self.send_command(0, 0, 0x95)?;
        if response != 0x01 {
            return Err(HalError::device_not_ready("SD card did not enter idle state"));
        }
        
        // Send CMD8 (SEND_IF_COND) to check voltage compatibility
        let response = self.send_command(8, 0x1AA, 0x87)?;
        if response != 0x01 {
            return Err(HalError::device_not_ready("SD card voltage check failed"));
        }
        
        // Read R7 response (4 bytes)
        let mut r7_response = [0u8; 4];
        for byte in &mut r7_response {
            *byte = self.spi_read_byte()?;
        }
        
        // Check voltage accepted
        if r7_response[2] != 0x01 || r7_response[3] != 0xAA {
            return Err(HalError::device_not_ready("SD card rejected voltage range"));
        }
        
        // Send ACMD41 (APP_SEND_OP_COND) repeatedly until card is ready
        let mut attempts = 0;
        loop {
            // First send CMD55 (APP_CMD)
            let response = self.send_command(55, 0, 0xFF)?;
            if response > 1 {
                return Err(HalError::device_not_ready("CMD55 failed"));
            }
            
            // Then send ACMD41 with HCS bit set (supports SDHC)
            let response = self.send_command(41, 0x40000000, 0xFF)?;
            
            if response == 0x00 {
                // Card is ready
                break;
            } else if response == 0x01 {
                // Card is still initializing
                attempts += 1;
                if attempts > 1000 {
                    return Err(HalError::timeout("SD card initialization timeout"));
                }
                // Wait a bit and retry
                self.delay_ms(1);
            } else {
                return Err(HalError::device_not_ready("ACMD41 failed"));
            }
        }
        
        // Send CMD58 (READ_OCR) to get card capacity info
        let response = self.send_command(58, 0, 0xFF)?;
        if response != 0x00 {
            return Err(HalError::device_not_ready("CMD58 failed"));
        }
        
        // Read OCR response (4 bytes)
        let mut ocr = [0u8; 4];
        for byte in &mut ocr {
            *byte = self.spi_read_byte()?;
        }
        
        // Check if card is SDHC/SDXC (CCS bit)
        let is_sdhc = (ocr[0] & 0x40) != 0;
        log::debug!("SD card type: {}", if is_sdhc { "SDHC/SDXC" } else { "SDSC" });
        
        log::info!("SD card initialization completed successfully");
        Ok(())
    }
    
    /// Send SD card command via SPI
    fn send_command(&mut self, cmd: u8, arg: u32, crc: u8) -> Result<u8, HalError> {
        // Pull CS low to select card
        self.sdcard.cs_pin.set_low();
        
        // Send command packet (6 bytes total)
        self.spi_write_byte(0x40 | cmd)?;  // Command byte with start bits
        self.spi_write_byte((arg >> 24) as u8)?;  // Argument bits 31-24
        self.spi_write_byte((arg >> 16) as u8)?;  // Argument bits 23-16
        self.spi_write_byte((arg >> 8) as u8)?;   // Argument bits 15-8
        self.spi_write_byte(arg as u8)?;          // Argument bits 7-0
        self.spi_write_byte(crc)?;                // CRC and end bit
        
        // Wait for response (R1 format)
        let mut response = 0xFF;
        for _ in 0..8 {  // Max 8 attempts
            response = self.spi_read_byte()?;
            if (response & 0x80) == 0 {
                // Valid response (MSB = 0)
                break;
            }
        }
        
        Ok(response)
    }
    
    /// Write single byte via SPI
    fn spi_write_byte(&mut self, data: u8) -> Result<(), HalError> {
        // TODO: Implement actual SPI write operation
        // For now, this is a placeholder
        log::trace!("SPI write: 0x{:02X}", data);
        Ok(())
    }
    
    /// Read single byte via SPI
    fn spi_read_byte(&mut self) -> Result<u8, HalError> {
        // TODO: Implement actual SPI read operation
        // For now, return mock data
        let data = 0xFF;
        log::trace!("SPI read: 0x{:02X}", data);
        Ok(data)
    }
    
    /// Delay for specified milliseconds
    fn delay_ms(&self, ms: u32) {
        // TODO: Implement actual delay
        // This would use the time provider or a hardware timer
        log::trace!("Delay: {}ms", ms);
    }
    
    /// Initialize FAT32 filesystem access
    fn init_fat32_filesystem(&mut self) -> Result<(), HalError> {
        // Read Master Boot Record (MBR) from sector 0
        let mut mbr = [0u8; 512];
        self.read_sector(0, &mut mbr)?;
        
        // Check MBR signature
        if mbr[510] != 0x55 || mbr[511] != 0xAA {
            return Err(HalError::data_corruption("Invalid MBR signature"));
        }
        
        // Find first FAT32 partition
        let mut fat32_start_sector = 0u32;
        for i in 0..4 {
            let partition_offset = 446 + (i * 16);
            let partition_type = mbr[partition_offset + 4];
            
            // Check for FAT32 partition types (0x0B, 0x0C)
            if partition_type == 0x0B || partition_type == 0x0C {
                // Read LBA start address (little endian)
                fat32_start_sector = u32::from_le_bytes([
                    mbr[partition_offset + 8],
                    mbr[partition_offset + 9],
                    mbr[partition_offset + 10],
                    mbr[partition_offset + 11],
                ]);
                break;
            }
        }
        
        if fat32_start_sector == 0 {
            return Err(HalError::device_not_ready("No FAT32 partition found"));
        }
        
        // Read FAT32 boot sector
        let mut boot_sector = [0u8; 512];
        self.read_sector(fat32_start_sector, &mut boot_sector)?;
        
        // Verify FAT32 signature
        if &boot_sector[82..90] != b"FAT32   " {
            return Err(HalError::data_corruption("Invalid FAT32 signature"));
        }
        
        log::info!("FAT32 filesystem detected and initialized");
        Ok(())
    }
    
    /// Read single sector (512 bytes) from SD card
    fn read_sector(&mut self, sector: u32, buffer: &mut [u8; 512]) -> Result<(), HalError> {
        // Send CMD17 (READ_SINGLE_BLOCK)
        let response = self.send_command(17, sector, 0xFF)?;
        if response != 0x00 {
            return Err(HalError::io_error("Failed to start sector read"));
        }
        
        // Wait for data token (0xFE)
        let mut token = 0xFF;
        for _ in 0..1000 {
            token = self.spi_read_byte()?;
            if token == 0xFE {
                break;
            }
        }
        
        if token != 0xFE {
            return Err(HalError::timeout("Sector read data token timeout"));
        }
        
        // Read 512 bytes of data
        for byte in buffer.iter_mut() {
            *byte = self.spi_read_byte()?;
        }
        
        // Read CRC (2 bytes, we'll ignore for now)
        self.spi_read_byte()?;
        self.spi_read_byte()?;
        
        // Release CS
        self.sdcard.cs_pin.set_high();
        
        Ok(())
    }
    
    /// Write single sector (512 bytes) to SD card
    fn write_sector(&mut self, sector: u32, data: &[u8; 512]) -> Result<(), HalError> {
        // Send CMD24 (WRITE_SINGLE_BLOCK)
        let response = self.send_command(24, sector, 0xFF)?;
        if response != 0x00 {
            return Err(HalError::io_error("Failed to start sector write"));
        }
        
        // Send data token
        self.spi_write_byte(0xFE)?;
        
        // Write 512 bytes of data
        for &byte in data {
            self.spi_write_byte(byte)?;
        }
        
        // Write dummy CRC (2 bytes)
        self.spi_write_byte(0xFF)?;
        self.spi_write_byte(0xFF)?;
        
        // Read data response
        let response = self.spi_read_byte()?;
        if (response & 0x1F) != 0x05 {
            return Err(HalError::io_error("Sector write was rejected"));
        }
        
        // Wait for write completion (card sends busy signal)
        let mut busy_count = 0;
        loop {
            let status = self.spi_read_byte()?;
            if status == 0xFF {
                break;  // Write completed
            }
            busy_count += 1;
            if busy_count > 10000 {
                return Err(HalError::timeout("Sector write completion timeout"));
            }
        }
        
        // Release CS
        self.sdcard.cs_pin.set_high();
        
        Ok(())
    }
    
    /// Create standard RumbleDome directory structure
    fn create_directory_structure(&mut self) -> Result<(), HalError> {
        let directories = [
            "/RUMBLEDOME",
            PROFILES_DIR,
            CONFIG_DIR,
            BACKUPS_DIR,
            LOGS_DIR,
            FIRMWARE_DIR,
        ];
        
        for dir in &directories {
            self.create_directory(dir)?;
        }
        
        log::debug!("Created SD card directory structure");
        Ok(())
    }
    
    /// Create directory on SD card
    fn create_directory(&mut self, path: &str) -> Result<(), HalError> {
        // TODO: Implement actual directory creation
        log::trace!("Created directory: {}", path);
        Ok(())
    }
    
    /// Read SD card hardware information
    fn read_card_info(&mut self) -> Result<SdCardInfo, HalError> {
        // TODO: Implement actual SD card info reading via SPI commands
        
        // Mock SD card info for now
        Ok(SdCardInfo {
            capacity_gb: 32.0,
            free_space_gb: 28.5,
            filesystem_type: "FAT32".to_string(),
            manufacturer: "SanDisk".to_string(),
            model: "Ultra".to_string(),
            serial_number: "0x12345678".to_string(),
            write_speed_class: 10,
            health_status: SdCardHealth::Excellent,
        })
    }
    
    /// Get file type from filename
    fn classify_file_type(&self, filename: &str) -> ConfigFileType {
        if filename.contains("profile") {
            ConfigFileType::UserProfiles
        } else if filename.contains("sensor") || filename.contains("calibration") {
            ConfigFileType::SensorCalibration
        } else if filename.contains("safety") {
            ConfigFileType::SafetyLimits
        } else if filename.contains("system") || filename.contains("preference") {
            ConfigFileType::SystemConfig
        } else if filename.ends_with(".bak") {
            ConfigFileType::BackupArchive
        } else if filename.contains("firmware") || filename.ends_with(".bin") {
            ConfigFileType::FirmwareUpdate
        } else {
            ConfigFileType::ProfileLibrary
        }
    }
    
    /// Read file from SD card
    fn read_file_internal(&mut self, filepath: &str) -> Result<HeaplessVec<u8, 8192>, HalError> {
        if !self.mounted {
            return Err(HalError::device_not_ready("SD card not mounted"));
        }
        
        log::trace!("Reading file: {}", filepath);
        
        // Find file in FAT32 filesystem
        let file_sector = self.find_file_sector(filepath)?;
        
        // Read file data (simplified - assumes file fits in one sector)
        let mut sector_data = [0u8; 512];
        self.read_sector(file_sector, &mut sector_data)?;
        
        // Extract actual file content (would need proper FAT32 parsing)
        // For now, assume entire sector is file data
        let mut content = HeaplessVec::new();
        
        // Find end of actual data (look for null terminator or use file size)
        let data_end = sector_data.iter().position(|&b| b == 0).unwrap_or(512);
        
        content.extend_from_slice(&sector_data[..data_end]).map_err(|_| 
            HalError::buffer_overflow("File too large for buffer"))?;
        
        Ok(content)
    }
    
    /// Write file to SD card
    fn write_file_internal(&mut self, filepath: &str, data: &[u8]) -> Result<(), HalError> {
        if !self.mounted {
            return Err(HalError::device_not_ready("SD card not mounted"));
        }
        
        if data.len() > 512 {
            return Err(HalError::invalid_parameter("File too large (max 512 bytes for now)"));
        }
        
        log::trace!("Writing {} bytes to file: {}", data.len(), filepath);
        
        // Find or allocate sector for file
        let file_sector = self.allocate_file_sector(filepath)?;
        
        // Prepare sector data
        let mut sector_data = [0u8; 512];
        sector_data[..data.len()].copy_from_slice(data);
        
        // Write to SD card
        self.write_sector(file_sector, &sector_data)?;
        
        Ok(())
    }
    
    /// Find sector containing specified file
    /// This is a simplified implementation - real FAT32 would traverse directory entries
    fn find_file_sector(&mut self, filepath: &str) -> Result<u32, HalError> {
        // Hash filename to sector number (simple approach for prototype)
        let mut hash = 0u32;
        for byte in filepath.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        
        // Map to data area (sectors 1000-2000 for our files)
        let sector = 1000 + (hash % 1000);
        
        log::trace!("File {} mapped to sector {}", filepath, sector);
        Ok(sector)
    }
    
    /// Allocate sector for new file
    fn allocate_file_sector(&mut self, filepath: &str) -> Result<u32, HalError> {
        // For simplicity, use same mapping as find_file_sector
        // Real implementation would check FAT and directory entries
        self.find_file_sector(filepath)
    }
    
    /// Generate full file path
    fn make_filepath(&self, directory: &str, filename: &str) -> String<128> {
        let mut path = String::new();
        write!(path, "{}/{}", directory, filename).ok();
        path
    }
    
    /// Create default user profiles
    fn create_default_profiles(&self) -> UserProfileSet {
        let metadata = ProfileSetMetadata {
            format_version: "1.0.0".to_string(),
            created_timestamp: 0, // TODO: Get actual timestamp
            modified_timestamp: 0,
            author: "RumbleDome".to_string(),
            vehicle_info: VehicleInfo {
                year: Some(2018),
                make: "Ford".to_string(),
                model: "Mustang GT".to_string(),
                engine: "5.0L Coyote V8".to_string(),
                turbo_system: "Single Turbo".to_string(),
                notes: "Gen 2 Coyote with aftermarket turbo".to_string(),
            },
            description: "Default RumbleDome profile set".to_string(),
            profile_count: 4,
        };
        
        let profiles = heapless::Vec::from_slice(&[
            BoostProfile {
                name: "Daily Driver".to_string(),
                description: "Conservative tune for daily driving".to_string(),
                max_boost_psi: 8.0,
                torque_target_percentage: 85.0,
                aggressiveness: 0.3,
                safety_margin: 2.0,
                environmental_adaptation: true,
                learning_enabled: true,
                tags: heapless::Vec::from_slice(&["daily".to_string(), "conservative".to_string()]).unwrap(),
            },
            BoostProfile {
                name: "Sport Mode".to_string(),
                description: "Moderate performance tune".to_string(),
                max_boost_psi: 12.0,
                torque_target_percentage: 95.0,
                aggressiveness: 0.6,
                safety_margin: 1.5,
                environmental_adaptation: true,
                learning_enabled: true,
                tags: heapless::Vec::from_slice(&["sport".to_string(), "performance".to_string()]).unwrap(),
            },
            BoostProfile {
                name: "Track Day".to_string(),
                description: "Aggressive track-focused tune".to_string(),
                max_boost_psi: 15.0,
                torque_target_percentage: 100.0,
                aggressiveness: 0.9,
                safety_margin: 1.0,
                environmental_adaptation: false,
                learning_enabled: true,
                tags: heapless::Vec::from_slice(&["track".to_string(), "aggressive".to_string()]).unwrap(),
            },
            BoostProfile {
                name: "Valet Mode".to_string(),
                description: "Ultra-safe mode for parking attendants".to_string(),
                max_boost_psi: 3.0,
                torque_target_percentage: 50.0,
                aggressiveness: 0.1,
                safety_margin: 5.0,
                environmental_adaptation: false,
                learning_enabled: false,
                tags: heapless::Vec::from_slice(&["valet".to_string(), "safe".to_string()]).unwrap(),
            },
        ]).unwrap();
        
        let sensor_calibrations = SensorCalibrations {
            dome_input_pressure: PressureSensorCalibration {
                zero_voltage: 0.5,
                full_scale_voltage: 4.5,
                full_scale_pressure: 30.0,
                sensor_model: "Generic 0-30 PSI".to_string(),
                calibration_date: 0,
                notes: "Standard automotive pressure sensor".to_string(),
            },
            upper_dome_pressure: PressureSensorCalibration {
                zero_voltage: 0.5,
                full_scale_voltage: 4.5,
                full_scale_pressure: 30.0,
                sensor_model: "Generic 0-30 PSI".to_string(),
                calibration_date: 0,
                notes: "Standard automotive pressure sensor".to_string(),
            },
            manifold_pressure: PressureSensorCalibration {
                zero_voltage: 0.5,
                full_scale_voltage: 4.5,
                full_scale_pressure: 30.0,
                sensor_model: "Generic MAP Sensor".to_string(),
                calibration_date: 0,
                notes: "Manifold absolute pressure sensor".to_string(),
            },
        };
        
        let safety_limits = UserSafetyLimits {
            absolute_max_boost: 18.0,
            overboost_threshold: 16.0,
            overboost_duration_ms: 2000,
            max_duty_cycle: 85.0,
            min_manifold_pressure: -15.0,
            max_slew_rate: 5.0,
        };
        
        let system_preferences = SystemPreferences {
            display_brightness: 80,
            gauge_style: "Modern".to_string(),
            units: UnitPreferences {
                pressure_units: "psi".to_string(),
                temperature_units: "fahrenheit".to_string(),
                boost_display_precision: 1,
            },
            can_bus_settings: CanBusPreferences {
                enable_torque_cooperation: true,
                ecu_message_timeout_ms: 1000,
                custom_can_ids: heapless::Vec::new(),
                message_filters: heapless::Vec::new(),
            },
            logging_level: "info".to_string(),
        };
        
        UserProfileSet {
            metadata,
            profiles,
            sensor_calibrations,
            safety_limits,
            system_preferences,
        }
    }
}

impl PortableStorage for Teensy41SdCard {
    fn mount(&mut self) -> Result<(), HalError> {
        if self.mounted {
            return Ok(());
        }
        
        self.init_filesystem()?;
        self.mounted = true;
        
        log::info!("SD card mounted successfully");
        Ok(())
    }
    
    fn unmount(&mut self) -> Result<(), HalError> {
        if !self.mounted {
            return Ok(());
        }
        
        // TODO: Sync any pending writes and unmount filesystem
        self.mounted = false;
        self.card_info = None;
        
        log::info!("SD card unmounted");
        Ok(())
    }
    
    fn is_mounted(&self) -> bool {
        self.mounted && self.card_present()
    }
    
    fn read_config_file(&mut self, filename: &str) -> Result<Vec<u8>, HalError> {
        let filepath = self.make_filepath(CONFIG_DIR, filename);
        let content = self.read_file_internal(filepath.as_str())?;
        
        // Convert to std::vec::Vec for trait compatibility
        let mut result = Vec::with_capacity(content.len());
        result.extend_from_slice(&content);
        
        Ok(result)
    }
    
    fn write_config_file(&mut self, filename: &str, data: &[u8]) -> Result<(), HalError> {
        let filepath = self.make_filepath(CONFIG_DIR, filename);
        self.write_file_internal(filepath.as_str(), data)?;
        
        log::debug!("Wrote config file: {} ({} bytes)", filename, data.len());
        Ok(())
    }
    
    fn list_config_files(&mut self) -> Result<Vec<ConfigFileInfo>, HalError> {
        if !self.mounted {
            return Err(HalError::device_not_ready("SD card not mounted"));
        }
        
        // TODO: Implement actual directory listing
        
        // Mock file listing for now
        let files = vec![
            ConfigFileInfo {
                filename: "sensor_calibrations.json".to_string(),
                size_bytes: 512,
                created_timestamp: 1640995200000,
                modified_timestamp: 1640995200000,
                file_type: ConfigFileType::SensorCalibration,
                description: "Pressure sensor calibration parameters".to_string(),
            },
            ConfigFileInfo {
                filename: "safety_limits.json".to_string(),
                size_bytes: 256,
                created_timestamp: 1640995200000,
                modified_timestamp: 1640995200000,
                file_type: ConfigFileType::SafetyLimits,
                description: "User-defined safety boundaries".to_string(),
            },
        ];
        
        Ok(files)
    }
    
    fn load_user_profiles(&mut self) -> Result<UserProfileSet, HalError> {
        let profiles_file = self.make_filepath(PROFILES_DIR, "user_profiles.json");
        
        match self.read_file_internal(profiles_file.as_str()) {
            Ok(data) => {
                // Parse JSON data
                let json_str = core::str::from_utf8(&data)
                    .map_err(|_| HalError::data_corruption("Invalid UTF-8 in profiles file"))?;
                
                serde_json::from_str(json_str)
                    .map_err(|e| HalError::data_corruption(
                        format!("Failed to parse profiles JSON: {}", e).as_str()
                    ))
            },
            Err(_) => {
                // Create default profiles if file doesn't exist
                log::info!("Creating default user profiles");
                let default_profiles = self.create_default_profiles();
                
                // Save default profiles to SD card
                self.save_user_profiles(&default_profiles)?;
                
                Ok(default_profiles)
            }
        }
    }
    
    fn save_user_profiles(&mut self, profiles: &UserProfileSet) -> Result<(), HalError> {
        let json_data = serde_json::to_string_pretty(profiles)
            .map_err(|e| HalError::serialization_error(
                format!("Failed to serialize profiles: {}", e).as_str()
            ))?;
        
        let profiles_file = self.make_filepath(PROFILES_DIR, "user_profiles.json");
        self.write_file_internal(profiles_file.as_str(), json_data.as_bytes())?;
        
        log::info!("Saved user profiles to SD card ({} profiles)", profiles.profiles.len());
        Ok(())
    }
    
    fn get_card_info(&self) -> Result<SdCardInfo, HalError> {
        self.card_info.clone()
            .ok_or_else(|| HalError::device_not_ready("SD card not mounted or info not available"))
    }
}

impl Drop for Teensy41SdCard {
    fn drop(&mut self) {
        if self.mounted {
            let _ = self.unmount();
        }
        log::debug!("SD card controller dropped");
    }
}