//! Non-volatile storage implementation for Teensy 4.1
//! 
//! Provides EEPROM emulation using the i.MX RT1062 FlexRAM 
//! for configuration and learned data persistence.
//!
//! ## Automotive Reality: Immediate Writes
//! 
//! Unlike desktop applications, automotive ECUs experience abrupt power loss
//! when the key is turned off - no graceful shutdown sequence. Therefore:
//! 
//! - **All writes are immediate** to ensure persistence
//! - **No deferred/cached writes** that could be lost on power loss
//! - **Learning system must minimize write frequency** to preserve EEPROM lifespan
//! - **Write cycle tracking** for wear leveling awareness
//!
//! The learning system should batch updates and only persist significant changes
//! rather than writing every minor calibration adjustment.

use crate::traits::NonVolatileStorage;
use crate::error::HalError;

use teensy4_bsp::hal;
use hal::ral::{flexram, FLEXRAM};

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

/// Wear warning threshold (80% of limit)
const WEAR_WARNING_THRESHOLD: u32 = (EEPROM_WEAR_LIMIT as f32 * 0.8) as u32;

/// Wear critical threshold (95% of limit) 
const WEAR_CRITICAL_THRESHOLD: u32 = (EEPROM_WEAR_LIMIT as f32 * 0.95) as u32;

/// Comprehensive wear tracking and health monitoring
#[derive(Debug, Clone)]
pub struct WearTrackingData {
    /// Write cycle counters per region
    pub region_write_counts: [u32; 8],
    
    /// Total bytes written since initialization
    pub total_bytes_written: u64,
    
    /// Total write operations since initialization
    pub total_write_operations: u32,
    
    /// First write timestamp per region (0 = never written)
    pub first_write_timestamps: [u64; 8],
    
    /// Last write timestamp per region  
    pub last_write_timestamps: [u64; 8],
    
    /// Running average write size per region
    pub average_write_sizes: [f32; 8],
    
    /// Peak write rate (writes per minute) observed
    pub peak_write_rate: f32,
    
    /// Current session write count
    pub session_write_count: u32,
    
    /// Health status per region
    pub region_health: [StorageHealth; 8],
}

/// Storage region health status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageHealth {
    /// Excellent condition (< 50% of wear limit)
    Excellent,
    /// Good condition (50-79% of wear limit)  
    Good,
    /// Warning condition (80-94% of wear limit)
    Warning,
    /// Critical condition (95-99% of wear limit)
    Critical,
    /// Failed or near failure (≥ wear limit)
    Failed,
}

/// Human-readable storage health report
#[derive(Debug, Clone)]
pub struct StorageHealthReport {
    /// Overall system health status
    pub overall_health: StorageHealth,
    
    /// Health per storage section
    pub section_health: SectionHealthReport,
    
    /// Total estimated lifespan remaining (years)
    pub estimated_lifespan_years: f32,
    
    /// Most worn region information
    pub most_worn_region: RegionWearInfo,
    
    /// Write activity statistics
    pub write_statistics: WriteStatistics,
    
    /// Human-readable health summary
    pub health_summary: String,
    
    /// Recommended actions
    pub recommendations: Vec<String>,
}

/// Health report per storage section
#[derive(Debug, Clone)]
pub struct SectionHealthReport {
    pub config_health: StorageHealth,
    pub learned_data_health: StorageHealth, 
    pub calibration_health: StorageHealth,
    pub safety_log_health: StorageHealth,
}

/// Most worn region details
#[derive(Debug, Clone)]
pub struct RegionWearInfo {
    pub region_id: usize,
    pub section_name: &'static str,
    pub write_count: u32,
    pub wear_percentage: f32,
    pub estimated_cycles_remaining: u32,
}

/// Write activity statistics
#[derive(Debug, Clone)]
pub struct WriteStatistics {
    pub total_writes_lifetime: u32,
    pub session_writes: u32,
    pub average_writes_per_hour: f32,
    pub peak_write_rate_per_minute: f32,
    pub total_data_written_kb: f32,
    pub uptime_hours: f32,
}

/// Teensy 4.1 non-volatile storage implementation
pub struct Teensy41Storage {
    /// FlexRAM instance configured for EEPROM emulation
    flexram: FLEXRAM,
    
    /// EEPROM buffer for caching
    eeprom_cache: [u8; EEPROM_SIZE],
    
    /// Comprehensive wear tracking data
    wear_tracking: WearTrackingData,
    
    /// Write cycle counters for wear leveling awareness
    write_counters: [u32; 8],
    
    /// Storage initialization timestamp for uptime tracking
    init_timestamp: u64,
}

impl Teensy41Storage {
    /// Create new storage controller
    pub fn new() -> Result<Self, HalError> {
        
        // Access FlexRAM peripheral
        let flexram = unsafe { FLEXRAM::instance() };
        
        // Configure FlexRAM for EEPROM emulation
        // Bank 7 (4KB) configured as EEPROM
        flexram.gpr_ctrl.modify(|_, w| {
            w.flexram_bank_cfg_sel().bit(true) // Use FlexRAM_BANK_CFG
        });
        
        // Set bank 7 as EEPROM (0b10)
        flexram.flexram_bank_cfg.modify(|_, w| unsafe {
            w.flexram_bank_cfg().bits(0x55555550) // All DTCM except bank 7 (EEPROM)
        });
        
        // Initialize EEPROM cache by reading current contents
        let mut eeprom_cache = [0u8; EEPROM_SIZE];
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
            flexram,
            eeprom_cache,
            wear_tracking: WearTrackingData::new(),
            write_counters: [0; 8],
            init_timestamp: 0, // TODO: Get actual timestamp from time provider
        })
    }
    
    /// Get region index for address
    fn get_region_index(&self, offset: usize) -> usize {
        offset / 512 // 512 bytes per region
    }
    
    /// Mark region as dirty for deferred write
    fn mark_dirty(&mut self, offset: usize, len: usize) {
        let start_region = self.get_region_index(offset);
        let end_region = self.get_region_index(offset + len - 1);
        
        for region in start_region..=end_region {
            if region < self.dirty_regions.len() {
                self.dirty_regions[region] = true;
            }
        }
    }
    
    /// Flush dirty regions to EEPROM
    fn flush_dirty_regions(&mut self) -> Result<(), HalError> {
        let eeprom_base = 0x1401C000u32 as *mut u8;
        
        for (region_idx, &dirty) in self.dirty_regions.iter().enumerate() {
            if dirty {
                let region_offset = region_idx * 512;
                let region_size = core::cmp::min(512, EEPROM_SIZE - region_offset);
                
                // Copy region from cache to EEPROM
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        self.eeprom_cache.as_ptr().add(region_offset),
                        eeprom_base.add(region_offset),
                        region_size
                    );
                }
                
                // Update counters and clear dirty flag
                self.write_counters[region_idx] += 1;
                self.dirty_regions[region_idx] = false;
                
                log::trace!("Flushed EEPROM region {} ({} bytes, {} writes)",
                    region_idx, region_size, self.write_counters[region_idx]);
            }
        }
        
        Ok(())
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
    
    /// Update comprehensive wear tracking data
    fn update_wear_tracking(&mut self, offset: usize, write_len: usize, timestamp: u64) {
        let start_region = self.get_region_index(offset);
        let end_region = self.get_region_index(offset + write_len - 1);
        
        for region in start_region..=end_region {
            if region < 8 {
                // Update write count
                self.wear_tracking.region_write_counts[region] += 1;
                
                // Track first write timestamp
                if self.wear_tracking.first_write_timestamps[region] == 0 {
                    self.wear_tracking.first_write_timestamps[region] = timestamp;
                }
                
                // Update last write timestamp
                self.wear_tracking.last_write_timestamps[region] = timestamp;
                
                // Update running average write size
                let current_avg = self.wear_tracking.average_write_sizes[region];
                let count = self.wear_tracking.region_write_counts[region] as f32;
                self.wear_tracking.average_write_sizes[region] = 
                    (current_avg * (count - 1.0) + write_len as f32) / count;
                
                // Update health status
                self.wear_tracking.region_health[region] = Self::calculate_health_status(
                    self.wear_tracking.region_write_counts[region]
                );
            }
        }
        
        // Update global statistics
        self.wear_tracking.total_bytes_written += write_len as u64;
        self.wear_tracking.total_write_operations += 1;
        self.wear_tracking.session_write_count += 1;
        
        // TODO: Update peak write rate tracking
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
    
    /// Generate comprehensive human-readable health report
    pub fn get_health_report(&self, current_time: u64) -> StorageHealthReport {
        let uptime_hours = (current_time.saturating_sub(self.init_timestamp)) as f32 / 3600000.0; // ms to hours
        
        // Find most worn region
        let (most_worn_idx, most_worn_count) = self.wear_tracking.region_write_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(idx, &count)| (idx, count))
            .unwrap_or((0, 0));
        
        let most_worn_region = RegionWearInfo {
            region_id: most_worn_idx,
            section_name: Self::get_section_name_for_region(most_worn_idx),
            write_count: most_worn_count,
            wear_percentage: (most_worn_count as f32 / EEPROM_WEAR_LIMIT as f32) * 100.0,
            estimated_cycles_remaining: EEPROM_WEAR_LIMIT.saturating_sub(most_worn_count),
        };
        
        // Calculate overall health (worst region determines overall status)
        let overall_health = self.wear_tracking.region_health
            .iter()
            .min()
            .copied()
            .unwrap_or(StorageHealth::Excellent);
        
        // Calculate estimated lifespan
        let current_rate = if uptime_hours > 0.0 {
            most_worn_count as f32 / uptime_hours
        } else { 0.0 };
        
        let estimated_lifespan_years = if current_rate > 0.0 {
            let remaining_cycles = most_worn_region.estimated_cycles_remaining as f32;
            let hours_remaining = remaining_cycles / current_rate;
            hours_remaining / (24.0 * 365.25) // Convert to years
        } else { f32::INFINITY };
        
        // Generate health summary and recommendations
        let (health_summary, recommendations) = self.generate_health_summary_and_recommendations(&overall_health, &most_worn_region);
        
        StorageHealthReport {
            overall_health,
            section_health: self.get_section_health(),
            estimated_lifespan_years,
            most_worn_region,
            write_statistics: WriteStatistics {
                total_writes_lifetime: self.wear_tracking.total_write_operations,
                session_writes: self.wear_tracking.session_write_count,
                average_writes_per_hour: if uptime_hours > 0.0 { 
                    self.wear_tracking.total_write_operations as f32 / uptime_hours 
                } else { 0.0 },
                peak_write_rate_per_minute: self.wear_tracking.peak_write_rate,
                total_data_written_kb: self.wear_tracking.total_bytes_written as f32 / 1024.0,
                uptime_hours,
            },
            health_summary,
            recommendations,
        }
    }
    
    /// Get health status for each storage section
    fn get_section_health(&self) -> SectionHealthReport {
        SectionHealthReport {
            config_health: self.wear_tracking.region_health[0], // Config in region 0
            learned_data_health: self.get_worst_health_in_range(1, 4), // Learned data spans regions 1-4
            calibration_health: self.get_worst_health_in_range(5, 6), // Calibration in regions 5-6
            safety_log_health: self.wear_tracking.region_health[7], // Safety log in region 7
        }
    }
    
    /// Get worst health status in a range of regions
    fn get_worst_health_in_range(&self, start: usize, end: usize) -> StorageHealth {
        (start..=end)
            .filter_map(|i| self.wear_tracking.region_health.get(i))
            .min()
            .copied()
            .unwrap_or(StorageHealth::Excellent)
    }
    
    /// Get human-readable section name for region
    fn get_section_name_for_region(region: usize) -> &'static str {
        match region {
            0 => "Configuration",
            1..=4 => "Learned Data",
            5..=6 => "Calibration",
            7 => "Safety Log",
            _ => "Unknown",
        }
    }
    
    /// Generate human-readable health summary and recommendations
    fn generate_health_summary_and_recommendations(&self, overall_health: &StorageHealth, most_worn: &RegionWearInfo) -> (String, Vec<String>) {
        let summary = match overall_health {
            StorageHealth::Excellent => {
                format!("Storage is in excellent condition. Most worn section '{}' at {:.1}% wear. Expected lifespan: many decades.", 
                    most_worn.section_name, most_worn.wear_percentage)
            },
            StorageHealth::Good => {
                format!("Storage is in good condition. Most worn section '{}' at {:.1}% wear. No immediate concerns.", 
                    most_worn.section_name, most_worn.wear_percentage)
            },
            StorageHealth::Warning => {
                format!("⚠️ Storage showing wear. Section '{}' at {:.1}% wear ({} cycles remaining). Monitor usage patterns.", 
                    most_worn.section_name, most_worn.wear_percentage, most_worn.estimated_cycles_remaining)
            },
            StorageHealth::Critical => {
                format!("🚨 CRITICAL: Storage heavily worn! Section '{}' at {:.1}% wear ({} cycles remaining). Plan replacement soon.", 
                    most_worn.section_name, most_worn.wear_percentage, most_worn.estimated_cycles_remaining)
            },
            StorageHealth::Failed => {
                format!("❌ STORAGE FAILURE: Section '{}' has exceeded wear limit! Replace ECU immediately.", 
                    most_worn.section_name)
            },
        };
        
        let mut recommendations = Vec::new();
        
        match overall_health {
            StorageHealth::Excellent | StorageHealth::Good => {
                recommendations.push("Continue normal operation. No action needed.".to_string());
            },
            StorageHealth::Warning => {
                recommendations.push("Monitor storage health more frequently.".to_string());
                recommendations.push("Consider reducing learning aggressiveness to extend lifespan.".to_string());
                recommendations.push("Plan for ECU replacement within next 2-3 years.".to_string());
            },
            StorageHealth::Critical => {
                recommendations.push("⚠️ Plan ECU replacement within 6-12 months.".to_string());
                recommendations.push("Reduce learning rate to preserve remaining cycles.".to_string());
                recommendations.push("Backup configuration and learned data.".to_string());
                recommendations.push("Monitor for data corruption or write failures.".to_string());
            },
            StorageHealth::Failed => {
                recommendations.push("🚨 REPLACE ECU IMMEDIATELY!".to_string());
                recommendations.push("Storage failure may cause data loss or unpredictable behavior.".to_string());
                recommendations.push("Do not rely on learning or configuration persistence.".to_string());
            },
        }
        
        (summary, recommendations)
    }
    
    /// Format health report for human-readable console output
    pub fn format_health_report_for_console(&self, current_time: u64) -> String {
        let report = self.get_health_report(current_time);
        
        let mut output = String::new();
        
        output.push_str("═══════════════════════════════════════════════════════════════════════\n");
        output.push_str("                         EEPROM HEALTH REPORT\n");
        output.push_str("═══════════════════════════════════════════════════════════════════════\n\n");
        
        // Overall status with emoji
        let status_emoji = match report.overall_health {
            StorageHealth::Excellent => "✅",
            StorageHealth::Good => "🟢", 
            StorageHealth::Warning => "⚠️",
            StorageHealth::Critical => "🚨",
            StorageHealth::Failed => "❌",
        };
        
        output.push_str(&format!("{} OVERALL STATUS: {:?}\n", status_emoji, report.overall_health));
        output.push_str(&format!("📊 {}\n\n", report.health_summary));
        
        // Storage section health
        output.push_str("STORAGE SECTION HEALTH:\n");
        output.push_str(&format!("  📋 Configuration:  {:?}\n", report.section_health.config_health));
        output.push_str(&format!("  🧠 Learned Data:   {:?}\n", report.section_health.learned_data_health));
        output.push_str(&format!("  🎛️  Calibration:   {:?}\n", report.section_health.calibration_health));
        output.push_str(&format!("  🛡️  Safety Log:    {:?}\n\n", report.section_health.safety_log_health));
        
        // Most worn region details
        output.push_str("MOST WORN REGION:\n");
        output.push_str(&format!("  📍 Section: {} (Region {})\n", 
            report.most_worn_region.section_name, report.most_worn_region.region_id));
        output.push_str(&format!("  📈 Wear: {:.1}% ({} of {} cycles used)\n", 
            report.most_worn_region.wear_percentage, 
            report.most_worn_region.write_count,
            EEPROM_WEAR_LIMIT));
        output.push_str(&format!("  ⏳ Remaining: {} cycles\n\n", 
            report.most_worn_region.estimated_cycles_remaining));
        
        // Write statistics
        output.push_str("WRITE ACTIVITY STATISTICS:\n");
        output.push_str(&format!("  📝 Total Writes (Lifetime): {}\n", report.write_statistics.total_writes_lifetime));
        output.push_str(&format!("  🔄 Session Writes: {}\n", report.write_statistics.session_writes));
        output.push_str(&format!("  📊 Average Writes/Hour: {:.1}\n", report.write_statistics.average_writes_per_hour));
        output.push_str(&format!("  🏃 Peak Rate: {:.1} writes/minute\n", report.write_statistics.peak_write_rate_per_minute));
        output.push_str(&format!("  💾 Total Data Written: {:.1} KB\n", report.write_statistics.total_data_written_kb));
        output.push_str(&format!("  ⏰ Uptime: {:.1} hours\n\n", report.write_statistics.uptime_hours));
        
        // Lifespan estimate
        if report.estimated_lifespan_years.is_infinite() {
            output.push_str("⏳ ESTIMATED LIFESPAN: Indefinite (no wear detected yet)\n\n");
        } else if report.estimated_lifespan_years > 100.0 {
            output.push_str("⏳ ESTIMATED LIFESPAN: >100 years (excellent condition)\n\n");
        } else {
            output.push_str(&format!("⏳ ESTIMATED LIFESPAN: {:.1} years at current usage rate\n\n", report.estimated_lifespan_years));
        }
        
        // Recommendations
        if !report.recommendations.is_empty() {
            output.push_str("RECOMMENDATIONS:\n");
            for (i, rec) in report.recommendations.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, rec));
            }
            output.push_str("\n");
        }
        
        output.push_str("═══════════════════════════════════════════════════════════════════════\n");
        
        output
    }
}

impl WearTrackingData {
    /// Create new wear tracking data structure
    fn new() -> Self {
        Self {
            region_write_counts: [0; 8],
            total_bytes_written: 0,
            total_write_operations: 0,
            first_write_timestamps: [0; 8],
            last_write_timestamps: [0; 8],
            average_write_sizes: [0.0; 8],
            peak_write_rate: 0.0,
            session_write_count: 0,
            region_health: [StorageHealth::Excellent; 8],
        }
    }
}

impl NonVolatileStorage for Teensy41Storage {
    fn read(&mut self, offset: usize, buffer: &mut [u8]) -> Result<usize, HalError> {
        if offset >= EEPROM_SIZE {
            return Err(HalError::invalid_parameter(
                format!("Read offset {} exceeds EEPROM size", offset)
            ));
        }
        
        let read_len = core::cmp::min(buffer.len(), EEPROM_SIZE - offset);
        buffer[..read_len].copy_from_slice(&self.eeprom_cache[offset..offset + read_len]);
        
        log::trace!("EEPROM read: {} bytes from offset {}", read_len, offset);
        
        Ok(read_len)
    }
    
    fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), HalError> {
        if offset >= EEPROM_SIZE {
            return Err(HalError::invalid_parameter(
                format!("Write offset {} exceeds EEPROM size", offset)
            ));
        }
        
        let write_len = core::cmp::min(data.len(), EEPROM_SIZE - offset);
        
        // Update cache
        self.eeprom_cache[offset..offset + write_len].copy_from_slice(&data[..write_len]);
        
        // AUTOMOTIVE REALITY: Write immediately to EEPROM since there's no graceful shutdown
        // Power loss (key-off) can happen at any time without warning
        let eeprom_base = 0x1401C000u32 as *mut u8;
        unsafe {
            core::ptr::copy_nonoverlapping(
                data.as_ptr(),
                eeprom_base.add(offset),
                write_len
            );
        }
        
        // Update comprehensive wear tracking
        let current_time = 0; // TODO: Get actual timestamp
        self.update_wear_tracking(offset, write_len, current_time);
        
        // Update legacy write counters for backward compatibility
        let start_region = self.get_region_index(offset);
        let end_region = self.get_region_index(offset + write_len - 1);
        for region in start_region..=end_region {
            if region < self.write_counters.len() {
                self.write_counters[region] += 1;
            }
        }
        
        log::trace!("EEPROM write: {} bytes to offset {} (immediate)", write_len, offset);
        
        Ok(())
    }
    
    fn erase_all(&mut self) -> Result<(), HalError> {
        // Clear cache
        self.eeprom_cache.fill(0xFF);
        
        // Mark all regions dirty
        self.dirty_regions.fill(true);
        
        // Flush to hardware
        self.flush_dirty_regions()?;
        
        log::info!("EEPROM erased completely");
        
        Ok(())
    }
    
    fn sync(&mut self) -> Result<(), HalError> {
        // With immediate writes, sync is a no-op but preserved for trait compatibility
        // All writes are already persisted to EEPROM immediately
        log::trace!("EEPROM sync called (no-op with immediate writes)");
        Ok(())
    }
    
    fn get_size(&self) -> usize {
        EEPROM_SIZE
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
    pub fn read(&self, storage: &mut Teensy41Storage, buffer: &mut [u8]) -> Result<usize, HalError> {
        if buffer.len() > self.size {
            return Err(HalError::invalid_parameter("Buffer larger than section"));
        }
        storage.read(self.offset, buffer)
    }
    
    /// Write to this storage section
    pub fn write(&self, storage: &mut Teensy41Storage, data: &[u8]) -> Result<(), HalError> {
        if data.len() > self.size {
            return Err(HalError::invalid_parameter("Data larger than section"));
        }
        storage.write(self.offset, data)
    }
}

impl Drop for Teensy41Storage {
    fn drop(&mut self) {
        // In automotive applications, Drop rarely executes due to abrupt power loss (key-off)
        // All writes are immediate, so no data loss occurs even without graceful shutdown
        log::debug!("Storage controller dropped (automotive: immediate writes ensure no data loss)");
    }
}