//! Non-volatile storage implementation for Teensy 4.1
//! 
//! Provides EEPROM emulation using the i.MX RT1062 FlexRAM 
//! for configuration and learned data persistence.

use crate::traits::NonVolatileStorage;
use crate::error::HalError;

/// EEPROM size in bytes (4KB for Teensy 4.1)
const EEPROM_SIZE: usize = 4096;

/// Configuration data storage offset
const CONFIG_OFFSET: usize = 0;
const CONFIG_SIZE: usize = 512;

/// Learned data storage offset  
const LEARNED_DATA_OFFSET: usize = CONFIG_SIZE;
const LEARNED_DATA_SIZE: usize = 2048;

/// Calibration data storage offset
const CALIBRATION_OFFSET: usize = CONFIG_SIZE + LEARNED_DATA_SIZE;
const CALIBRATION_SIZE: usize = 1024;

/// Safety log storage offset
const SAFETY_LOG_OFFSET: usize = CONFIG_SIZE + LEARNED_DATA_SIZE + CALIBRATION_SIZE;
const SAFETY_LOG_SIZE: usize = 512;

/// EEPROM wear limit per region (conservative estimate)
const EEPROM_WEAR_LIMIT: u32 = 100_000;

/// Storage region health status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageHealth {
    Excellent,
    Good,
    Warning,
    Critical,
    Failed,
}

/// Teensy 4.1 non-volatile storage implementation
pub struct Teensy41Storage {
    /// EEPROM buffer for caching
    eeprom_cache: [u8; EEPROM_SIZE],
    
    /// Write cycle counters for wear leveling awareness
    write_counters: [u32; 8],
    
    /// Storage initialization timestamp for uptime tracking
    init_timestamp: u64,
}

impl Teensy41Storage {
    /// Create new storage controller
    pub fn new() -> Result<Self, HalError> {
        // Initialize EEPROM cache by reading current contents
        let mut eeprom_cache = [0u8; EEPROM_SIZE];
        
        #[cfg(target_arch = "arm")]
        unsafe {
            let eeprom_base = 0x1401C000u32 as *const u8; // EEPROM base address
            core::ptr::copy_nonoverlapping(
                eeprom_base,
                eeprom_cache.as_mut_ptr(),
                EEPROM_SIZE
            );
        }
        
        log::info!("EEPROM initialized: {} bytes available", EEPROM_SIZE);
        log::debug!("Memory layout - Config: {}B, Learned: {}B, Cal: {}B, Log: {}B",
            CONFIG_SIZE, LEARNED_DATA_SIZE, CALIBRATION_SIZE, SAFETY_LOG_SIZE);
        
        Ok(Self {
            eeprom_cache,
            write_counters: [0; 8],
            init_timestamp: 0,
        })
    }
    
    /// Get region index for address
    fn get_region_index(&self, offset: usize) -> usize {
        offset / 512 // 512 bytes per region
    }
    
    /// Validate storage layout and detect corruption
    pub fn validate_storage(&mut self) -> Result<bool, HalError> {
        // Simple checksum validation for each storage section
        let config_checksum = self.calculate_checksum(CONFIG_OFFSET, CONFIG_SIZE);
        let learned_checksum = self.calculate_checksum(LEARNED_DATA_OFFSET, LEARNED_DATA_SIZE);
        let cal_checksum = self.calculate_checksum(CALIBRATION_OFFSET, CALIBRATION_SIZE);
        
        // Check for obvious corruption patterns
        let config_valid = config_checksum != 0 && config_checksum != 0xFFFF;
        let learned_valid = learned_checksum != 0;
        let cal_valid = cal_checksum != 0;
        
        log::debug!("Storage validation - Config: {}, Learned: {}, Cal: {}",
            config_valid, learned_valid, cal_valid);
        
        Ok(config_valid && (learned_valid || cal_valid))
    }
    
    /// Calculate simple 16-bit checksum for data validation
    fn calculate_checksum(&self, offset: usize, len: usize) -> u16 {
        let mut checksum: u16 = 0;
        let end = core::cmp::min(offset + len, EEPROM_SIZE);
        
        for i in offset..end {
            checksum = checksum.wrapping_add(self.eeprom_cache[i] as u16);
        }
        
        checksum
    }
    
    /// Calculate health status based on write count
    fn calculate_health_status(write_count: u32) -> StorageHealth {
        let wear_percentage = (write_count as f32 / EEPROM_WEAR_LIMIT as f32) * 100.0;
        
        match wear_percentage {
            p if p < 50.0 => StorageHealth::Excellent,
            p if p < 80.0 => StorageHealth::Good,
            p if p < 95.0 => StorageHealth::Warning,
            p if p < 100.0 => StorageHealth::Critical,
            _ => StorageHealth::Failed,
        }
    }
    
    /// Get current wear statistics
    pub fn get_wear_stats(&self) -> [u32; 8] {
        self.write_counters
    }
    
    /// Get overall storage health
    pub fn get_health_status(&self) -> StorageHealth {
        let max_writes = *self.write_counters.iter().max().unwrap_or(&0);
        Self::calculate_health_status(max_writes)
    }
}

impl NonVolatileStorage for Teensy41Storage {
    fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), HalError> {
        let offset = address as usize;
        
        if offset >= EEPROM_SIZE {
            return Err(HalError::invalid_parameter(
                format!("Read address {} exceeds EEPROM size", address)
            ));
        }
        
        let read_len = core::cmp::min(buffer.len(), EEPROM_SIZE - offset);
        buffer[..read_len].copy_from_slice(&self.eeprom_cache[offset..offset + read_len]);
        
        log::trace!("EEPROM read: {} bytes from address {}", read_len, address);
        
        Ok(())
    }
    
    fn write(&mut self, address: u32, data: &[u8]) -> Result<(), HalError> {
        let offset = address as usize;
        
        if offset >= EEPROM_SIZE {
            return Err(HalError::invalid_parameter(
                format!("Write address {} exceeds EEPROM size", address)
            ));
        }
        
        let write_len = core::cmp::min(data.len(), EEPROM_SIZE - offset);
        
        // Update cache
        self.eeprom_cache[offset..offset + write_len].copy_from_slice(&data[..write_len]);
        
        // AUTOMOTIVE REALITY: Write immediately to EEPROM since there's no graceful shutdown
        #[cfg(target_arch = "arm")]
        unsafe {
            let eeprom_base = 0x1401C000u32 as *mut u8;
            core::ptr::copy_nonoverlapping(
                data.as_ptr(),
                eeprom_base.add(offset),
                write_len
            );
        }
        
        // Update wear tracking
        let region = self.get_region_index(offset);
        if region < self.write_counters.len() {
            self.write_counters[region] += 1;
        }
        
        log::trace!("EEPROM write: {} bytes to address {} (immediate)", write_len, address);
        
        Ok(())
    }
    
    fn erase_sector(&mut self, address: u32) -> Result<(), HalError> {
        let offset = address as usize;
        let sector_size = 512; // 512 byte sectors
        
        if offset >= EEPROM_SIZE {
            return Err(HalError::invalid_parameter(
                format!("Erase address {} exceeds EEPROM size", address)
            ));
        }
        
        let erase_start = (offset / sector_size) * sector_size;
        let erase_end = core::cmp::min(erase_start + sector_size, EEPROM_SIZE);
        
        // Clear cache sector
        self.eeprom_cache[erase_start..erase_end].fill(0xFF);
        
        // Write to hardware immediately
        #[cfg(target_arch = "arm")]
        unsafe {
            let eeprom_base = 0x1401C000u32 as *mut u8;
            core::ptr::write_bytes(
                eeprom_base.add(erase_start),
                0xFF,
                erase_end - erase_start
            );
        }
        
        // Update wear tracking
        let region = self.get_region_index(offset);
        if region < self.write_counters.len() {
            self.write_counters[region] += 1;
        }
        
        log::info!("EEPROM sector erased: address {:#X}, {} bytes", address, erase_end - erase_start);
        
        Ok(())
    }
    
    fn capacity(&self) -> u32 {
        EEPROM_SIZE as u32
    }
    
    fn maintenance(&mut self) -> Result<(), HalError> {
        // EEPROM doesn't require wear leveling maintenance like flash
        // This is mainly for monitoring and health reporting
        
        let health_status = self.get_health_status();
        
        // Log health status for monitoring
        match health_status {
            StorageHealth::Warning => {
                log::warn!("EEPROM showing wear - consider reducing write frequency");
            },
            StorageHealth::Critical => {
                log::error!("EEPROM critically worn - plan ECU replacement soon");
            },
            StorageHealth::Failed => {
                log::error!("EEPROM failure detected - replace ECU immediately");
            },
            _ => {
                log::debug!("EEPROM health: {:?}", health_status);
            }
        }
        
        Ok(())
    }
}

/// Storage section accessor for type-safe access
pub struct StorageSection {
    offset: usize,
    size: usize,
}

impl StorageSection {
    pub const fn new(offset: usize, size: usize) -> Self {
        Self { offset, size }
    }
    
    /// Configuration data section
    pub const CONFIG: Self = Self::new(CONFIG_OFFSET, CONFIG_SIZE);
    
    /// Learned data section (interpolation tables, etc.)
    pub const LEARNED_DATA: Self = Self::new(LEARNED_DATA_OFFSET, LEARNED_DATA_SIZE);
    
    /// Calibration data section
    pub const CALIBRATION: Self = Self::new(CALIBRATION_OFFSET, CALIBRATION_SIZE);
    
    /// Safety event log section
    pub const SAFETY_LOG: Self = Self::new(SAFETY_LOG_OFFSET, SAFETY_LOG_SIZE);
    
    pub fn offset(&self) -> usize { self.offset }
    pub fn size(&self) -> usize { self.size }
    
    /// Read from this storage section
    pub fn read(&self, storage: &mut Teensy41Storage, buffer: &mut [u8]) -> Result<(), HalError> {
        if buffer.len() > self.size {
            return Err(HalError::invalid_parameter("Buffer larger than section"));
        }
        storage.read(self.offset as u32, buffer)
    }
    
    /// Write to this storage section
    pub fn write(&self, storage: &mut Teensy41Storage, data: &[u8]) -> Result<(), HalError> {
        if data.len() > self.size {
            return Err(HalError::invalid_parameter("Data larger than section"));
        }
        storage.write(self.offset as u32, data)
    }
}

impl Drop for Teensy41Storage {
    fn drop(&mut self) {
        // In automotive applications, Drop rarely executes due to abrupt power loss (key-off)
        // All writes are immediate, so no data loss occurs even without graceful shutdown
        log::debug!("Storage controller dropped (automotive: immediate writes ensure no data loss)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn setup_test_storage() -> Teensy41Storage {
        Teensy41Storage::new().unwrap()
    }
    
    #[test]
    fn test_storage_capacity() {
        let storage = setup_test_storage();
        assert_eq!(storage.capacity(), 4096);
    }
    
    #[test]
    fn test_basic_read_write() {
        let mut storage = setup_test_storage();
        
        let test_data = b"Hello, EEPROM!";
        let mut read_buffer = [0u8; 20];
        
        // Write test data
        storage.write(100, test_data).unwrap();
        
        // Read it back
        storage.read(100, &mut read_buffer).unwrap();
        
        // Verify data matches
        assert_eq!(&read_buffer[..test_data.len()], test_data);
    }
    
    #[test]
    fn test_sector_erase() {
        let mut storage = setup_test_storage();
        
        // Write some data
        let test_data = b"This will be erased";
        storage.write(0, test_data).unwrap();
        
        // Erase sector
        storage.erase_sector(0).unwrap();
        
        // Verify data is erased (0xFF)
        let mut read_buffer = [0u8; 512];
        storage.read(0, &mut read_buffer).unwrap();
        assert!(read_buffer.iter().all(|&b| b == 0xFF));
    }
    
    #[test]
    fn test_wear_tracking() {
        let mut storage = setup_test_storage();
        let initial_wear = storage.get_wear_stats();
        
        // Perform several writes to region 0
        for i in 0..5 {
            storage.write(i * 10, &[0x42]).unwrap();
        }
        
        let updated_wear = storage.get_wear_stats();
        
        // Region 0 should have increased write count
        assert!(updated_wear[0] > initial_wear[0]);
    }
    
    #[test]
    fn test_storage_sections() {
        let mut storage = setup_test_storage();
        
        // Test each storage section
        let config_data = b"CONFIG";
        StorageSection::CONFIG.write(&mut storage, config_data).unwrap();
        
        let learned_data = b"LEARNED";
        StorageSection::LEARNED_DATA.write(&mut storage, learned_data).unwrap();
        
        let cal_data = b"CALIBRATION";
        StorageSection::CALIBRATION.write(&mut storage, cal_data).unwrap();
        
        let log_data = b"SAFETY";
        StorageSection::SAFETY_LOG.write(&mut storage, log_data).unwrap();
        
        // Verify each section
        let mut buffer = [0u8; 20];
        
        StorageSection::CONFIG.read(&mut storage, &mut buffer).unwrap();
        assert_eq!(&buffer[..config_data.len()], config_data);
        
        StorageSection::LEARNED_DATA.read(&mut storage, &mut buffer).unwrap();
        assert_eq!(&buffer[..learned_data.len()], learned_data);
    }
    
    #[test]
    fn test_address_bounds() {
        let mut storage = setup_test_storage();
        
        // Test read beyond capacity
        let mut buffer = [0u8; 10];
        let result = storage.read(5000, &mut buffer);
        assert!(result.is_err());
        
        // Test write beyond capacity
        let result = storage.write(5000, &[0x42]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_health_monitoring() {
        let mut storage = setup_test_storage();
        
        // Initially should be excellent health
        assert_eq!(storage.get_health_status(), StorageHealth::Excellent);
        
        // Maintenance should not fail
        assert!(storage.maintenance().is_ok());
    }
}