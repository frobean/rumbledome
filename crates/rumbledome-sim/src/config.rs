//! Configuration management for the simulator

use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

/// Simulator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorConfig {
    /// Default control loop frequency
    pub default_frequency: u16,
    
    /// Engine simulation parameters
    pub engine: EngineConfig,
    
    /// UI settings
    pub ui: UiConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Engine simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Engine displacement in liters
    pub displacement: f32,
    
    /// Turbo size factor
    pub turbo_size: f32,
    
    /// Wastegate spring pressure
    pub spring_pressure: f32,
    
    /// Maximum RPM
    pub max_rpm: u16,
    
    /// Idle RPM
    pub idle_rpm: u16,
    
    /// Torque curve parameters
    pub torque_curve: TorqueCurveConfig,
}

/// Torque curve configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorqueCurveConfig {
    /// Peak torque in Nm
    pub peak_torque: f32,
    
    /// RPM at peak torque
    pub peak_torque_rpm: u16,
    
    /// Maximum torque (at torque ceiling)
    pub max_torque: f32,
    
    /// Torque drop-off rate at high RPM
    pub high_rpm_dropoff: f32,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Update rate in Hz
    pub update_rate: u16,
    
    /// Chart history length
    pub chart_history_points: usize,
    
    /// Default gauge ranges
    pub gauge_ranges: GaugeRanges,
}

/// Gauge display ranges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeRanges {
    /// Maximum boost pressure display range
    pub max_boost_psi: f32,
    
    /// Maximum RPM display range
    pub max_rpm_display: u16,
    
    /// Maximum duty cycle (always 100%)
    pub max_duty_cycle: f32,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level filter
    pub level: String,
    
    /// Enable file logging
    pub enable_file_log: bool,
    
    /// Log file path
    pub log_file_path: String,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            default_frequency: 100,
            engine: EngineConfig {
                displacement: 5.0, // 5.0L Coyote
                turbo_size: 1.0,
                spring_pressure: 5.0,
                max_rpm: 7000,
                idle_rpm: 800,
                torque_curve: TorqueCurveConfig {
                    peak_torque: 400.0,    // Nm at peak
                    peak_torque_rpm: 4000,
                    max_torque: 600.0,     // Maximum possible
                    high_rpm_dropoff: 0.8,
                },
            },
            ui: UiConfig {
                update_rate: 30, // 30 FPS
                chart_history_points: 1000,
                gauge_ranges: GaugeRanges {
                    max_boost_psi: 20.0,
                    max_rpm_display: 8000,
                    max_duty_cycle: 100.0,
                },
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                enable_file_log: false,
                log_file_path: "rumbledome-sim.log".to_string(),
            },
        }
    }
}

impl SimulatorConfig {
    /// Load configuration from file
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: SimulatorConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Load configuration from file, or create default if file doesn't exist
    pub fn load_or_default(path: &str) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => {
                log::info!("Loaded configuration from {}", path);
                config
            },
            Err(e) => {
                log::warn!("Failed to load config from {}: {}. Using defaults.", path, e);
                let default_config = Self::default();
                
                // Try to save default config
                if let Err(save_err) = default_config.save_to_file(path) {
                    log::warn!("Failed to save default config: {}", save_err);
                } else {
                    log::info!("Saved default configuration to {}", path);
                }
                
                default_config
            }
        }
    }
}