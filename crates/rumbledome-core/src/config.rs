//! Configuration management for RumbleDome

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Wastegate spring pressure in PSI
    pub spring_pressure: f32,
    
    /// Available boost profiles
    pub profiles: HashMap<String, BoostProfile>,
    
    /// Currently active profile name
    pub active_profile: String,
    
    /// Scramble boost profile name
    pub scramble_profile: String,
    
    /// Torque target strategy (percentage of ECU desired torque to target)
    pub torque_target_percentage: f32,
    
    /// Boost change slew rate limit in PSI/second
    pub boost_slew_rate: f32,
    
    /// Control loop frequency in Hz
    pub control_frequency: u16,
    
    /// Safety limits and parameters
    pub safety: SafetyConfig,
    
    /// Platform-specific CAN configuration
    pub can: CanConfig,
    
    /// Hardware-specific calibration
    pub hardware: HardwareConfig,
}

/// Boost profile definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoostProfile {
    /// Profile display name
    pub name: String,
    
    /// Description of profile purpose
    pub description: String,
    
    /// Boost targets as RPM -> PSI mapping
    /// Points are linearly interpolated between defined RPM values
    pub boost_targets: Vec<BoostPoint>,
    
    /// Maximum boost pressure limit for this profile
    pub max_boost: f32,
    
    /// Overboost cut threshold (above max_boost)
    pub overboost_limit: f32,
    
    /// Overboost recovery hysteresis in PSI
    pub overboost_hysteresis: f32,
    
    /// Profile-specific PID tuning
    pub pid_tuning: PidConfig,
}

/// RPM to boost pressure mapping point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoostPoint {
    /// Engine RPM
    pub rpm: u16,
    /// Target boost pressure in PSI gauge
    pub boost_psi: f32,
}

/// PID controller configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidConfig {
    /// Proportional gain
    pub kp: f32,
    /// Integral gain
    pub ki: f32,
    /// Derivative gain
    pub kd: f32,
    /// Integral windup limit
    pub integral_limit: f32,
}

/// Safety system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Global overboost limit regardless of profile
    pub global_overboost_limit: f32,
    
    /// Maximum duty cycle change per control cycle (%)
    pub max_duty_change_per_cycle: f32,
    
    /// Maximum safe RPM for boost control
    pub max_rpm: u16,
    
    /// Minimum engine RPM to arm system
    pub min_rpm_for_arming: u16,
    
    /// Sensor validation ranges
    pub sensor_ranges: SensorRanges,
    
    /// CAN timeout before fault condition (ms)
    pub can_timeout_ms: u64,
    
    /// Maximum control loop execution time before warning (ms)
    pub max_loop_time_ms: u64,
}

/// Sensor validation ranges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorRanges {
    /// Dome input pressure valid range (PSI)
    pub dome_input_pressure: (f32, f32),
    
    /// Upper dome pressure valid range (PSI)
    pub upper_dome_pressure: (f32, f32),
    
    /// Manifold pressure valid range (PSI gauge)
    pub manifold_pressure: (f32, f32),
}

/// CAN bus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanConfig {
    /// CAN bus bit rate
    pub bitrate: u32,
    
    /// Platform-specific signal definitions
    pub signals: CanSignalConfig,
}

/// CAN signal configuration for specific vehicle platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanSignalConfig {
    /// Engine RPM signal
    pub rpm: CanSignal,
    
    /// Manifold absolute pressure signal
    pub map: CanSignal,
    
    /// ECU desired torque signal
    pub desired_torque: CanSignal,
    
    /// ECU actual torque signal
    pub actual_torque: CanSignal,
    
    /// Throttle position signal (optional, for Phase 2+)
    pub throttle_position: Option<CanSignal>,
    
    /// Drive mode signal (optional, for Phase 2+)
    pub drive_mode: Option<CanSignal>,
}

/// Individual CAN signal definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanSignal {
    /// CAN message ID
    pub id: u32,
    
    /// Byte offset within message
    pub byte_offset: u8,
    
    /// Number of bytes
    pub byte_length: u8,
    
    /// Little-endian byte order
    pub little_endian: bool,
    
    /// Scale factor (raw_value * scale = engineering_units)
    pub scale: f32,
    
    /// Offset (engineering_units = raw_value * scale + offset)
    pub offset: f32,
    
    /// Engineering units
    pub units: String,
}

/// Hardware-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    /// PWM frequency for solenoid control (Hz)
    pub pwm_frequency: u16,
    
    /// ADC reference voltage
    pub adc_vref: f32,
    
    /// ADC resolution (bits)
    pub adc_resolution: u8,
    
    /// Pressure sensor calibration
    pub pressure_sensors: PressureSensorConfig,
    
    /// Display configuration
    pub display: DisplayConfig,
}

/// Pressure sensor calibration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureSensorConfig {
    /// Voltage at 0 PSI
    pub zero_pressure_voltage: f32,
    
    /// Voltage at full scale (30 PSI)
    pub full_scale_voltage: f32,
    
    /// Full scale pressure in PSI
    pub full_scale_pressure: f32,
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Screen width in pixels
    pub width: u16,
    
    /// Screen height in pixels
    pub height: u16,
    
    /// Default brightness (0-100%)
    pub brightness: u8,
    
    /// Update rate in Hz
    pub update_rate: u16,
}

impl Default for SystemConfig {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        
        // Valet profile - minimal boost for safety
        profiles.insert("valet".to_string(), BoostProfile {
            name: "Valet".to_string(),
            description: "Minimal boost for inexperienced drivers".to_string(),
            boost_targets: vec![
                BoostPoint { rpm: 1000, boost_psi: 0.0 },
                BoostPoint { rpm: 7000, boost_psi: 2.0 },
            ],
            max_boost: 2.0,
            overboost_limit: 3.0,
            overboost_hysteresis: 0.5,
            pid_tuning: PidConfig {
                kp: 0.3,
                ki: 0.1,
                kd: 0.0,
                integral_limit: 5.0,
            },
        });
        
        // Daily profile - conservative for daily driving
        profiles.insert("daily".to_string(), BoostProfile {
            name: "Daily".to_string(),
            description: "Conservative boost for daily driving".to_string(),
            boost_targets: vec![
                BoostPoint { rpm: 1500, boost_psi: 0.0 },
                BoostPoint { rpm: 2500, boost_psi: 3.0 },
                BoostPoint { rpm: 3500, boost_psi: 7.0 },
                BoostPoint { rpm: 4000, boost_psi: 8.0 },
                BoostPoint { rpm: 7000, boost_psi: 8.0 },
            ],
            max_boost: 8.0,
            overboost_limit: 9.5,
            overboost_hysteresis: 0.3,
            pid_tuning: PidConfig {
                kp: 0.45,
                ki: 0.30,
                kd: 0.0,
                integral_limit: 10.0,
            },
        });
        
        // Aggressive profile - spirited driving
        profiles.insert("aggressive".to_string(), BoostProfile {
            name: "Aggressive".to_string(),
            description: "Higher boost for spirited driving".to_string(),
            boost_targets: vec![
                BoostPoint { rpm: 1500, boost_psi: 0.0 },
                BoostPoint { rpm: 2500, boost_psi: 4.0 },
                BoostPoint { rpm: 3500, boost_psi: 9.0 },
                BoostPoint { rpm: 4000, boost_psi: 10.0 },
                BoostPoint { rpm: 7000, boost_psi: 10.0 },
            ],
            max_boost: 10.0,
            overboost_limit: 11.5,
            overboost_hysteresis: 0.3,
            pid_tuning: PidConfig {
                kp: 0.6,
                ki: 0.35,
                kd: 0.0,
                integral_limit: 10.0,
            },
        });
        
        // Track profile - maximum performance
        profiles.insert("track".to_string(), BoostProfile {
            name: "Track".to_string(),
            description: "Maximum safe boost for track use".to_string(),
            boost_targets: vec![
                BoostPoint { rpm: 1500, boost_psi: 0.0 },
                BoostPoint { rpm: 2500, boost_psi: 5.0 },
                BoostPoint { rpm: 3500, boost_psi: 11.0 },
                BoostPoint { rpm: 4000, boost_psi: 12.0 },
                BoostPoint { rpm: 7000, boost_psi: 12.0 },
            ],
            max_boost: 12.0,
            overboost_limit: 13.5,
            overboost_hysteresis: 0.3,
            pid_tuning: PidConfig {
                kp: 0.7,
                ki: 0.4,
                kd: 0.0,
                integral_limit: 10.0,
            },
        });
        
        Self {
            spring_pressure: 5.0,
            profiles,
            active_profile: "daily".to_string(),
            scramble_profile: "track".to_string(),
            torque_target_percentage: 95.0,
            boost_slew_rate: 5.0,
            control_frequency: 100,
            
            safety: SafetyConfig {
                global_overboost_limit: 15.0,
                max_duty_change_per_cycle: 2.0,
                max_rpm: 7500,
                min_rpm_for_arming: 800,
                sensor_ranges: SensorRanges {
                    dome_input_pressure: (0.0, 25.0),
                    upper_dome_pressure: (0.0, 25.0),
                    manifold_pressure: (-15.0, 25.0), // Vacuum to boost
                },
                can_timeout_ms: 500,
                max_loop_time_ms: 15,
            },
            
            can: CanConfig {
                bitrate: 500_000,
                signals: CanSignalConfig {
                    rpm: CanSignal {
                        id: 0x201, // ⚠ SPECULATIVE - needs verification
                        byte_offset: 0,
                        byte_length: 2,
                        little_endian: true,
                        scale: 0.25,
                        offset: 0.0,
                        units: "RPM".to_string(),
                    },
                    map: CanSignal {
                        id: 0x202, // ⚠ SPECULATIVE - needs verification
                        byte_offset: 0,
                        byte_length: 2,
                        little_endian: true,
                        scale: 0.1,
                        offset: 0.0,
                        units: "kPa".to_string(),
                    },
                    desired_torque: CanSignal {
                        id: 0x203, // ⚠ SPECULATIVE - needs verification
                        byte_offset: 0,
                        byte_length: 2,
                        little_endian: true,
                        scale: 0.1,
                        offset: -1000.0,
                        units: "Nm".to_string(),
                    },
                    actual_torque: CanSignal {
                        id: 0x204, // ⚠ SPECULATIVE - needs verification
                        byte_offset: 0,
                        byte_length: 2,
                        little_endian: true,
                        scale: 0.1,
                        offset: -1000.0,
                        units: "Nm".to_string(),
                    },
                    throttle_position: None, // Phase 2+
                    drive_mode: None,       // Phase 2+
                },
            },
            
            hardware: HardwareConfig {
                pwm_frequency: 30,
                adc_vref: 3.3,      // Teensy 4.1 ADC reference
                adc_resolution: 12,
                pressure_sensors: PressureSensorConfig {
                    zero_pressure_voltage: 0.167, // 0.5V * 0.333 (10kΩ+20kΩ divider)
                    full_scale_voltage: 1.5,      // 4.5V * 0.333 (10kΩ+20kΩ divider)
                    full_scale_pressure: 30.0,
                },
                display: DisplayConfig {
                    width: 128,
                    height: 160,
                    brightness: 80,
                    update_rate: 10,
                },
            },
        }
    }
}

impl BoostProfile {
    /// Get boost target for specified RPM via linear interpolation
    pub fn get_boost_target(&self, rpm: u16) -> f32 {
        if self.boost_targets.is_empty() {
            return 0.0;
        }
        
        // Find surrounding points for interpolation
        if rpm <= self.boost_targets[0].rpm {
            return self.boost_targets[0].boost_psi;
        }
        
        if rpm >= self.boost_targets.last().unwrap().rpm {
            return self.boost_targets.last().unwrap().boost_psi;
        }
        
        // Linear interpolation between points
        for window in self.boost_targets.windows(2) {
            if rpm >= window[0].rpm && rpm <= window[1].rpm {
                let rpm_range = window[1].rpm - window[0].rpm;
                let boost_range = window[1].boost_psi - window[0].boost_psi;
                let rpm_offset = rpm - window[0].rpm;
                
                if rpm_range == 0 {
                    return window[0].boost_psi;
                }
                
                return window[0].boost_psi + (boost_range * rpm_offset as f32) / rpm_range as f32;
            }
        }
        
        0.0 // Should not reach here
    }
}

impl SystemConfig {
    /// Get currently active boost profile
    pub fn get_active_profile(&self) -> Option<&BoostProfile> {
        self.profiles.get(&self.active_profile)
    }
    
    /// Get scramble boost profile
    pub fn get_scramble_profile(&self) -> Option<&BoostProfile> {
        self.profiles.get(&self.scramble_profile)
    }
    
    /// Validate configuration for consistency and safety
    pub fn validate(&self) -> Result<(), String> {
        // Validate spring pressure
        if self.spring_pressure < 0.0 || self.spring_pressure > 20.0 {
            return Err(format!("Invalid spring pressure: {} PSI", self.spring_pressure));
        }
        
        // Validate active profile exists
        if !self.profiles.contains_key(&self.active_profile) {
            return Err(format!("Active profile '{}' not found", self.active_profile));
        }
        
        // Validate scramble profile exists
        if !self.profiles.contains_key(&self.scramble_profile) {
            return Err(format!("Scramble profile '{}' not found", self.scramble_profile));
        }
        
        // Validate torque target percentage
        if self.torque_target_percentage < 50.0 || self.torque_target_percentage > 100.0 {
            return Err(format!("Invalid torque target percentage: {}%", self.torque_target_percentage));
        }
        
        // Validate each profile
        for (name, profile) in &self.profiles {
            if let Err(e) = self.validate_profile(profile) {
                return Err(format!("Profile '{}' invalid: {}", name, e));
            }
        }
        
        Ok(())
    }
    
    fn validate_profile(&self, profile: &BoostProfile) -> Result<(), String> {
        // Check boost targets are sorted by RPM
        for window in profile.boost_targets.windows(2) {
            if window[1].rpm <= window[0].rpm {
                return Err("Boost targets must be sorted by increasing RPM".to_string());
            }
        }
        
        // Check boost values are reasonable
        for point in &profile.boost_targets {
            if point.boost_psi < 0.0 || point.boost_psi > self.safety.global_overboost_limit {
                return Err(format!("Boost target {} PSI exceeds safety limit", point.boost_psi));
            }
        }
        
        // Check overboost limit is above max boost
        if profile.overboost_limit <= profile.max_boost {
            return Err("Overboost limit must be above max boost".to_string());
        }
        
        Ok(())
    }
}