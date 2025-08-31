//! Sensor fusion for manifold pressure readings
//! 
//! Combines CAN MAP sensor (vacuum range) with added boost gauge (boost range)
//! to provide full-spectrum manifold pressure from deep vacuum to high boost.

use crate::types::{
    CombinedManifoldPressure, ManifoldPressureSource, SensorAgreement,
    SystemInputs, SensorReadings, CanData
};
use crate::error::HalError;

/// Sensor fusion configuration parameters
#[derive(Debug, Clone)]
pub struct SensorFusionConfig {
    /// Transition zone around atmospheric pressure (PSI gauge)
    pub transition_zone_psi: f32,
    
    /// Sensor agreement tolerance (PSI)
    pub agreement_tolerance_psi: f32,
    
    /// Major disagreement threshold (PSI)
    pub major_disagreement_threshold_psi: f32,
    
    /// Boost gauge offset calibration (PSI)
    pub boost_gauge_offset: f32,
    
    /// CAN MAP sensor calibration offset (kPa)
    pub can_map_offset: f32,
}

impl Default for SensorFusionConfig {
    fn default() -> Self {
        Self {
            transition_zone_psi: 1.0,     // ±1 PSI around atmospheric
            agreement_tolerance_psi: 0.5,  // Sensors should agree within 0.5 PSI
            major_disagreement_threshold_psi: 2.0, // Major fault if >2 PSI difference
            boost_gauge_offset: 0.0,       // Calibrated offset for boost gauge
            can_map_offset: 0.0,          // Calibrated offset for CAN MAP
        }
    }
}

/// Manifold pressure sensor fusion implementation
pub struct ManifoldPressureFusion {
    config: SensorFusionConfig,
    atmospheric_baseline: f32, // PSI absolute
}

impl ManifoldPressureFusion {
    /// Create new sensor fusion instance
    pub fn new(config: SensorFusionConfig) -> Self {
        Self {
            config,
            atmospheric_baseline: 14.7, // Sea level default
        }
    }
    
    /// Update atmospheric pressure baseline
    pub fn update_atmospheric_baseline(&mut self, baseline_psi: f32) {
        self.atmospheric_baseline = baseline_psi;
    }
    
    /// Combine CAN MAP and boost gauge readings into unified manifold pressure
    pub fn combine_manifold_readings(
        &mut self,
        can_data: &CanData,
        sensors: &SensorReadings,
    ) -> Result<CombinedManifoldPressure, HalError> {
        
        // Convert CAN MAP from kPa absolute to PSI gauge
        let can_map_psi_absolute = can_data.map_kpa / 6.895; // kPa to PSI
        let can_map_psi_gauge = can_map_psi_absolute - self.atmospheric_baseline;
        
        // Get boost gauge reading (already in PSI gauge)
        let boost_gauge_psi = sensors.manifold_pressure_gauge + self.config.boost_gauge_offset;
        
        // Learn cross-calibration in overlap zone
        let _learned_offset = self.update_cross_calibration(can_map_psi_gauge, boost_gauge_psi);
        
        // Determine optimal sensor source based on operating range
        let (primary_source, pressure_gauge_psi, in_transition_zone) = 
            self.select_primary_sensor(can_map_psi_gauge, boost_gauge_psi)?;
        
        // Check sensor agreement (for diagnostics only, not faults)
        let sensor_agreement = self.check_sensor_agreement(
            can_map_psi_gauge, 
            boost_gauge_psi, 
            primary_source
        );
        
        Ok(CombinedManifoldPressure {
            pressure_gauge_psi,
            primary_source,
            sensor_agreement,
            in_transition_zone,
        })
    }
    
    /// Select optimal sensor based on operating conditions
    fn select_primary_sensor(
        &self,
        can_map_gauge: f32,
        boost_gauge: f32,
    ) -> Result<(ManifoldPressureSource, f32, bool), HalError> {
        
        let transition_zone = self.config.transition_zone_psi;
        
        // Deep vacuum: CAN MAP sensor is more accurate
        if can_map_gauge < -transition_zone {
            return Ok((ManifoldPressureSource::CanMapSensor, can_map_gauge, false));
        }
        
        // Significant boost: Boost gauge sensor is more accurate
        if boost_gauge > transition_zone {
            return Ok((ManifoldPressureSource::BoostGaugeSensor, boost_gauge, false));
        }
        
        // Transition zone around atmospheric pressure: blend sensors
        let in_transition = can_map_gauge.abs() <= transition_zone || boost_gauge.abs() <= transition_zone;
        
        if in_transition {
            // Weighted blend favoring the sensor closer to its optimal range
            let vacuum_weight = if can_map_gauge < 0.0 { 
                (-can_map_gauge / transition_zone).min(1.0) 
            } else { 
                0.0 
            };
            
            let boost_weight = if boost_gauge > 0.0 { 
                (boost_gauge / transition_zone).min(1.0) 
            } else { 
                0.0 
            };
            
            let total_weight = vacuum_weight + boost_weight;
            
            let blended_pressure = if total_weight > 0.0 {
                (can_map_gauge * vacuum_weight + boost_gauge * boost_weight) / total_weight
            } else {
                // No clear preference - use simple average
                (can_map_gauge + boost_gauge) / 2.0
            };
            
            Ok((ManifoldPressureSource::BlendedSensors, blended_pressure, true))
        } else {
            // Default to boost gauge for positive readings, CAN MAP for negative
            if boost_gauge >= 0.0 {
                Ok((ManifoldPressureSource::BoostGaugeSensor, boost_gauge, false))
            } else {
                Ok((ManifoldPressureSource::CanMapSensor, can_map_gauge, false))
            }
        }
    }
    
    /// Check agreement between sensors in their overlap range
    fn check_sensor_agreement(
        &self,
        can_map_gauge: f32,
        boost_gauge: f32,
        primary_source: ManifoldPressureSource,
    ) -> SensorAgreement {
        
        // Only check agreement near atmospheric pressure where both sensors are valid
        if can_map_gauge.abs() > 5.0 || boost_gauge.abs() > 5.0 {
            return SensorAgreement::OutOfRange;
        }
        
        let pressure_diff = (can_map_gauge - boost_gauge).abs();
        
        if pressure_diff <= self.config.agreement_tolerance_psi {
            SensorAgreement::Good
        } else if pressure_diff <= self.config.major_disagreement_threshold_psi {
            SensorAgreement::MinorDisagreement
        } else {
            SensorAgreement::MajorDisagreement
        }
    }
    
    /// Update sensor cross-calibration based on overlap readings
    /// Returns the learned offset for storage in persistent learning system
    pub fn update_cross_calibration(
        &mut self,
        can_map_gauge: f32,
        boost_gauge: f32,
    ) -> Option<f32> {
        // Only learn in the overlap zone where both sensors are reasonably accurate
        if can_map_gauge.abs() <= 2.0 && boost_gauge.abs() <= 2.0 {
            let sensor_offset = can_map_gauge - boost_gauge;
            
            // Simple exponential moving average for offset learning
            // This will converge to the systematic difference between sensors
            let learning_rate = 0.01; // 1% learning rate for stability
            self.config.boost_gauge_offset += sensor_offset * learning_rate;
            
            Some(self.config.boost_gauge_offset)
        } else {
            None
        }
    }
    
    /// Get sensor fusion diagnostics
    pub fn get_diagnostics(&self, combined: &CombinedManifoldPressure) -> Vec<String> {
        let mut diagnostics = Vec::new();
        
        diagnostics.push(format!(
            "Manifold pressure: {:.1} PSI ({:?})",
            combined.pressure_gauge_psi,
            combined.primary_source
        ));
        
        match combined.sensor_agreement {
            SensorAgreement::Good => {},
            SensorAgreement::MinorDisagreement => {
                diagnostics.push("Minor MAP sensor disagreement".to_string());
            },
            SensorAgreement::MajorDisagreement => {
                diagnostics.push("⚠️ Major MAP sensor disagreement".to_string());
            },
            SensorAgreement::OutOfRange => {
                diagnostics.push("MAP sensors outside agreement range".to_string());
            },
        }
        
        if combined.in_transition_zone {
            diagnostics.push("MAP sensor blending active".to_string());
        }
        
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn setup_fusion() -> ManifoldPressureFusion {
        ManifoldPressureFusion::new(SensorFusionConfig::default())
    }
    
    #[test]
    fn test_vacuum_range_selection() {
        let fusion = setup_fusion();
        
        // Deep vacuum - should use CAN MAP
        let can_data = CanData {
            map_kpa: 50.0, // ~7.25 PSI absolute = -7.45 PSI gauge
            ..Default::default()
        };
        let sensors = SensorReadings {
            manifold_pressure_gauge: 0.1, // Boost gauge near zero
            ..Default::default()
        };
        
        let result = fusion.combine_manifold_readings(&can_data, &sensors).unwrap();
        
        assert_eq!(result.primary_source, ManifoldPressureSource::CanMapSensor);
        assert!(result.pressure_gauge_psi < -2.0); // Should be vacuum
        assert!(!result.in_transition_zone);
    }
    
    #[test]
    fn test_boost_range_selection() {
        let fusion = setup_fusion();
        
        // Boost conditions - should use boost gauge
        let can_data = CanData {
            map_kpa: 101.3, // Atmospheric
            ..Default::default()
        };
        let sensors = SensorReadings {
            manifold_pressure_gauge: 8.0, // 8 PSI boost
            ..Default::default()
        };
        
        let result = fusion.combine_manifold_readings(&can_data, &sensors).unwrap();
        
        assert_eq!(result.primary_source, ManifoldPressureSource::BoostGaugeSensor);
        assert!((result.pressure_gauge_psi - 8.0).abs() < 0.1);
        assert!(!result.in_transition_zone);
    }
    
    #[test]
    fn test_transition_zone_blending() {
        let fusion = setup_fusion();
        
        // Near atmospheric - should blend sensors
        let can_data = CanData {
            map_kpa: 100.0, // Slightly below atmospheric
            ..Default::default()
        };
        let sensors = SensorReadings {
            manifold_pressure_gauge: 0.5, // Slight boost
            ..Default::default()
        };
        
        let result = fusion.combine_manifold_readings(&can_data, &sensors).unwrap();
        
        assert_eq!(result.primary_source, ManifoldPressureSource::BlendedSensors);
        assert!(result.in_transition_zone);
    }
    
    #[test]
    fn test_sensor_disagreement_detection() {
        let fusion = setup_fusion();
        
        // Both sensors near atmospheric but disagreeing
        let can_data = CanData {
            map_kpa: 98.0, // -1.9 PSI gauge
            ..Default::default()
        };
        let sensors = SensorReadings {
            manifold_pressure_gauge: 1.5, // +1.5 PSI boost
            ..Default::default()
        };
        
        let result = fusion.combine_manifold_readings(&can_data, &sensors).unwrap();
        
        // 3.4 PSI difference should trigger major disagreement
        assert_eq!(result.sensor_agreement, SensorAgreement::MajorDisagreement);
    }
}