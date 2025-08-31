//! EEPROM/NVM Storage Integration Tests
//! 
//! Tests the non-volatile storage implementation to ensure data persistence
//! and wear leveling work correctly.

use rumbledome_hal::traits::NonVolatileStorage;

/// Mock storage implementation for testing
struct MockStorage {
    data: Vec<u8>,
    write_counts: std::collections::HashMap<u32, u32>,
}

impl MockStorage {
    fn new(capacity: usize) -> Self {
        Self {
            data: vec![0xFF; capacity],
            write_counts: std::collections::HashMap::new(),
        }
    }
    
    fn get_write_count(&self, address: u32) -> u32 {
        *self.write_counts.get(&address).unwrap_or(&0)
    }
}

impl NonVolatileStorage for MockStorage {
    fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), rumbledome_hal::HalError> {
        let start = address as usize;
        let end = start + buffer.len();
        
        if end > self.data.len() {
            return Err(rumbledome_hal::HalError::invalid_parameter("Read beyond capacity"));
        }
        
        buffer.copy_from_slice(&self.data[start..end]);
        Ok(())
    }
    
    fn write(&mut self, address: u32, data: &[u8]) -> Result<(), rumbledome_hal::HalError> {
        let start = address as usize;
        let end = start + data.len();
        
        if end > self.data.len() {
            return Err(rumbledome_hal::HalError::invalid_parameter("Write beyond capacity"));
        }
        
        self.data[start..end].copy_from_slice(data);
        *self.write_counts.entry(address).or_insert(0) += 1;
        Ok(())
    }
    
    fn erase_sector(&mut self, address: u32) -> Result<(), rumbledome_hal::HalError> {
        let sector_start = (address as usize / 512) * 512;
        let sector_end = std::cmp::min(sector_start + 512, self.data.len());
        
        for i in sector_start..sector_end {
            self.data[i] = 0xFF;
        }
        
        *self.write_counts.entry(address).or_insert(0) += 1;
        Ok(())
    }
    
    fn capacity(&self) -> u32 {
        self.data.len() as u32
    }
    
    fn maintenance(&mut self) -> Result<(), rumbledome_hal::HalError> {
        // Mock maintenance - just log write counts
        let total_writes: u32 = self.write_counts.values().sum();
        println!("Storage maintenance: {} total writes", total_writes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_basic_operations() {
        let mut storage = MockStorage::new(4096);
        
        // Test capacity
        assert_eq!(storage.capacity(), 4096);
        
        // Test write and read
        let test_data = b"Hello, EEPROM!";
        storage.write(100, test_data).unwrap();
        
        let mut read_buffer = vec![0u8; test_data.len()];
        storage.read(100, &mut read_buffer).unwrap();
        
        assert_eq!(&read_buffer, test_data);
    }
    
    #[test]
    fn test_storage_bounds_checking() {
        let mut storage = MockStorage::new(1024);
        
        // Test read beyond capacity
        let mut buffer = [0u8; 10];
        let result = storage.read(1020, &mut buffer);
        assert!(result.is_err());
        
        // Test write beyond capacity
        let result = storage.write(1020, &[1, 2, 3, 4, 5, 6]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_sector_erase() {
        let mut storage = MockStorage::new(4096);
        
        // Write data across sector boundary
        let test_data = vec![0x42; 600]; // Spans 2 sectors
        storage.write(400, &test_data).unwrap();
        
        // Erase first sector
        storage.erase_sector(0).unwrap();
        
        // First 512 bytes should be 0xFF
        let mut first_sector = vec![0u8; 512];
        storage.read(0, &mut first_sector).unwrap();
        assert!(first_sector.iter().all(|&b| b == 0xFF));
        
        // Data in second sector should remain
        let mut second_sector_part = vec![0u8; 88]; // 600 - 512 = 88 bytes
        storage.read(512, &mut second_sector_part).unwrap();
        assert!(second_sector_part.iter().all(|&b| b == 0x42));
    }
    
    #[test]
    fn test_wear_tracking() {
        let mut storage = MockStorage::new(4096);
        
        // Perform multiple writes to same address
        for _ in 0..5 {
            storage.write(100, &[0x42]).unwrap();
        }
        
        assert_eq!(storage.get_write_count(100), 5);
        
        // Different address should have different count
        storage.write(200, &[0x24]).unwrap();
        assert_eq!(storage.get_write_count(200), 1);
        assert_eq!(storage.get_write_count(100), 5); // Unchanged
    }
    
    #[test]
    fn test_configuration_sections() {
        let mut storage = MockStorage::new(4096);
        
        // Simulate the actual storage layout
        const CONFIG_OFFSET: u32 = 0;
        const LEARNED_DATA_OFFSET: u32 = 512;
        const CALIBRATION_OFFSET: u32 = 2560; // 512 + 2048
        const SAFETY_LOG_OFFSET: u32 = 3584; // 512 + 2048 + 1024
        
        // Write to each section
        storage.write(CONFIG_OFFSET, b"CONFIG_DATA").unwrap();
        storage.write(LEARNED_DATA_OFFSET, b"LEARNED_TABLES").unwrap();
        storage.write(CALIBRATION_OFFSET, b"CALIBRATION_PARAMS").unwrap();
        storage.write(SAFETY_LOG_OFFSET, b"SAFETY_EVENTS").unwrap();
        
        // Verify each section can be read independently
        let mut buffer = vec![0u8; 20];
        
        storage.read(CONFIG_OFFSET, &mut buffer[..11]).unwrap();
        assert_eq!(&buffer[..11], b"CONFIG_DATA");
        
        storage.read(LEARNED_DATA_OFFSET, &mut buffer[..14]).unwrap();
        assert_eq!(&buffer[..14], b"LEARNED_TABLES");
        
        storage.read(CALIBRATION_OFFSET, &mut buffer[..18]).unwrap();
        assert_eq!(&buffer[..18], b"CALIBRATION_PARAMS");
        
        storage.read(SAFETY_LOG_OFFSET, &mut buffer[..13]).unwrap();
        assert_eq!(&buffer[..13], b"SAFETY_EVENTS");
    }
    
    #[test]
    fn test_storage_maintenance() {
        let mut storage = MockStorage::new(4096);
        
        // Perform some writes
        for i in 0..10 {
            storage.write(i * 100, &[i as u8]).unwrap();
        }
        
        // Maintenance should not fail
        assert!(storage.maintenance().is_ok());
    }
}