//! Learning system for RumbleDome
//! 
//! Implements learned duty cycle mappings and environmental compensation

use crate::error::CoreError;
use rumbledome_hal::{SystemInputs, SensorReadings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Learned calibration data storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedData {
    /// 2D interpolation table for duty cycle mappings
    pub calibration_map: CalibrationMap,
    
    /// Environmental compensation factors
    pub environmental_factors: EnvironmentalFactors,
    
    /// Confidence and quality metrics
    pub confidence_metrics: ConfidenceMetrics,
    
    /// Learning system statistics
    pub statistics: LearningStatistics,
    
    /// Last update timestamp
    pub last_updated: u64,
}

/// 2D interpolation table for (RPM, Boost) -> Duty Cycle mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationMap {
    /// Calibration data points
    pub points: Vec<CalibrationPoint>,
    
    /// Bounds for valid extrapolation
    pub bounds: CalibrationBounds,
    
    /// Default fallback duty cycle when no data available
    pub default_duty: f32,
}

/// Individual calibration data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationPoint {
    /// Engine RPM
    pub rpm: u16,
    
    /// Target boost pressure (PSI)
    pub boost_psi: f32,
    
    /// Learned duty cycle (%)
    pub duty_cycle: f32,
    
    /// Confidence in this data point (0-1)
    pub confidence: f32,
    
    /// Number of samples averaged for this point
    pub sample_count: u32,
    
    /// Last update timestamp
    pub last_updated: u64,
}

/// Calibration bounds for safe extrapolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationBounds {
    /// Minimum RPM with calibration data
    pub min_rpm: u16,
    
    /// Maximum RPM with calibration data
    pub max_rpm: u16,
    
    /// Minimum boost with calibration data
    pub min_boost: f32,
    
    /// Maximum boost with calibration data
    pub max_boost: f32,
}

/// Environmental compensation factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalFactors {
    /// Temperature compensation (duty cycle adjustment per °C)
    pub temperature_compensation: f32,
    
    /// Altitude compensation (duty cycle adjustment per 1000ft)
    pub altitude_compensation: f32,
    
    /// Dome supply pressure baseline (PSI)
    pub supply_pressure_baseline: f32,
    
    /// Supply pressure compensation (duty cycle adjustment per PSI deviation)
    pub supply_pressure_compensation: f32,
    
    /// Learning confidence for environmental factors
    pub environmental_confidence: f32,
}

/// Confidence and quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceMetrics {
    /// Overall calibration confidence (0-1)
    pub overall_confidence: f32,
    
    /// Number of calibrated RPM points
    pub calibrated_rpm_points: u32,
    
    /// Number of calibrated boost levels
    pub calibrated_boost_levels: u32,
    
    /// Average confidence across all points
    pub average_point_confidence: f32,
    
    /// Minimum confidence threshold for use
    pub confidence_threshold: f32,
}

/// Learning system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStatistics {
    /// Total learning updates performed
    pub total_updates: u64,
    
    /// Last learning session timestamp
    pub last_learning_session: Option<u64>,
    
    /// Number of successful calibration runs
    pub successful_runs: u32,
    
    /// Number of failed calibration attempts
    pub failed_runs: u32,
    
    /// Average duty cycle accuracy (PSI error)
    pub average_accuracy: f32,
}

impl LearnedData {
    /// Lookup duty cycle for given RPM and boost target
    pub fn lookup_duty_cycle(
        &self,
        rpm: u16,
        target_boost: f32,
        sensors: &SensorReadings,
    ) -> Result<f32, CoreError> {
        
        // Find the closest calibration points for interpolation
        let interpolated_duty = self.calibration_map.interpolate(rpm, target_boost)?;
        
        // Apply environmental compensation
        let compensated_duty = self.apply_environmental_compensation(
            interpolated_duty,
            sensors,
        )?;
        
        // Ensure duty cycle is within reasonable bounds
        Ok(compensated_duty.clamp(0.0, 100.0))
    }
    
    /// Apply environmental compensation to duty cycle
    fn apply_environmental_compensation(
        &self,
        base_duty: f32,
        sensors: &SensorReadings,
    ) -> Result<f32, CoreError> {
        
        let mut compensated_duty = base_duty;
        
        // Supply pressure compensation
        let supply_pressure_error = sensors.dome_input_pressure - 
            self.environmental_factors.supply_pressure_baseline;
        let supply_compensation = supply_pressure_error * 
            self.environmental_factors.supply_pressure_compensation;
        
        compensated_duty += supply_compensation;
        
        // TODO: Temperature and altitude compensation (requires additional sensors)
        // For now, these are placeholders for future implementation
        
        Ok(compensated_duty)
    }
    
    /// Update calibration data with new learned point
    pub fn update_calibration_point(
        &mut self,
        rpm: u16,
        target_boost: f32,
        achieved_boost: f32,
        duty_cycle: f32,
        timestamp: u64,
    ) -> Result<(), CoreError> {
        
        // Calculate accuracy of this data point
        let accuracy = (target_boost - achieved_boost).abs();
        
        // Only accept reasonably accurate data points
        if accuracy > 1.0 {
            return Err(CoreError::learning(
                format!("Inaccurate calibration point rejected: {:.2} PSI error", accuracy)
            ));
        }
        
        // Find existing point or create new one
        let point_index = self.calibration_map.points.iter().position(|p| {
            (p.rpm as i32 - rpm as i32).abs() < 200 && // Within 200 RPM
            (p.boost_psi - target_boost).abs() < 0.5   // Within 0.5 PSI
        });
        
        if let Some(index) = point_index {
            // Update existing point with exponential moving average
            let existing = &mut self.calibration_map.points[index];
            let alpha = 0.3; // Learning rate
            
            existing.duty_cycle = existing.duty_cycle * (1.0 - alpha) + duty_cycle * alpha;
            existing.sample_count += 1;
            existing.last_updated = timestamp;
            
            // Increase confidence with more samples (up to 1.0)
            existing.confidence = (existing.confidence + 0.1).min(1.0);
            
        } else {
            // Create new calibration point
            let new_point = CalibrationPoint {
                rpm,
                boost_psi: target_boost,
                duty_cycle,
                confidence: 0.5, // Start with medium confidence
                sample_count: 1,
                last_updated: timestamp,
            };
            
            self.calibration_map.points.push(new_point);
        }
        
        // Update bounds
        self.update_calibration_bounds();
        
        // Update statistics
        self.statistics.total_updates += 1;
        self.statistics.last_learning_session = Some(timestamp);
        self.statistics.average_accuracy = 
            (self.statistics.average_accuracy * 0.9) + (accuracy * 0.1);
        
        self.last_updated = timestamp;
        
        Ok(())
    }
    
    /// Update calibration bounds based on current data
    fn update_calibration_bounds(&mut self) {
        if self.calibration_map.points.is_empty() {
            return;
        }
        
        let mut min_rpm = u16::MAX;
        let mut max_rpm = u16::MIN;
        let mut min_boost = f32::MAX;
        let mut max_boost = f32::MIN;
        
        for point in &self.calibration_map.points {
            min_rpm = min_rpm.min(point.rpm);
            max_rpm = max_rpm.max(point.rpm);
            min_boost = min_boost.min(point.boost_psi);
            max_boost = max_boost.max(point.boost_psi);
        }
        
        self.calibration_map.bounds = CalibrationBounds {
            min_rpm,
            max_rpm,
            min_boost,
            max_boost,
        };
    }
    
    /// Calculate overall system confidence
    pub fn calculate_confidence(&mut self) {
        let mut total_confidence = 0.0;
        let mut confident_points = 0;
        
        for point in &self.calibration_map.points {
            total_confidence += point.confidence;
            if point.confidence >= self.confidence_metrics.confidence_threshold {
                confident_points += 1;
            }
        }
        
        if !self.calibration_map.points.is_empty() {
            self.confidence_metrics.average_point_confidence = 
                total_confidence / self.calibration_map.points.len() as f32;
            
            self.confidence_metrics.overall_confidence = 
                (confident_points as f32) / (self.calibration_map.points.len() as f32);
        }
        
        self.confidence_metrics.calibrated_rpm_points = self.calibration_map.points
            .iter()
            .map(|p| p.rpm)
            .collect::<std::collections::HashSet<_>>()
            .len() as u32;
            
        self.confidence_metrics.calibrated_boost_levels = self.calibration_map.points
            .iter()
            .map(|p| (p.boost_psi * 2.0) as u16) // 0.5 PSI buckets
            .collect::<std::collections::HashSet<_>>()
            .len() as u32;
    }
    
    /// Reset all learned data
    pub fn reset(&mut self) {
        self.calibration_map.points.clear();
        self.confidence_metrics.overall_confidence = 0.0;
        self.confidence_metrics.average_point_confidence = 0.0;
        self.confidence_metrics.calibrated_rpm_points = 0;
        self.confidence_metrics.calibrated_boost_levels = 0;
        self.statistics.total_updates = 0;
        self.statistics.last_learning_session = None;
    }
}

impl CalibrationMap {
    /// Interpolate duty cycle for given RPM and boost target
    pub fn interpolate(&self, rpm: u16, target_boost: f32) -> Result<f32, CoreError> {
        
        if self.points.is_empty() {
            return Ok(self.default_duty);
        }
        
        // Find points for bilinear interpolation
        let nearby_points = self.find_interpolation_points(rpm, target_boost);
        
        if nearby_points.is_empty() {
            return Ok(self.default_duty);
        }
        
        if nearby_points.len() == 1 {
            return Ok(nearby_points[0].duty_cycle);
        }
        
        // Weighted average based on distance and confidence
        let mut total_weight = 0.0;
        let mut weighted_duty = 0.0;
        
        for point in &nearby_points {
            let rpm_distance = ((point.rpm as f32) - (rpm as f32)).abs();
            let boost_distance = (point.boost_psi - target_boost).abs();
            
            // Distance-based weight (closer points have more influence)
            let distance_weight = 1.0 / (1.0 + rpm_distance / 1000.0 + boost_distance * 2.0);
            
            // Combine with confidence weight
            let total_point_weight = distance_weight * point.confidence;
            
            weighted_duty += point.duty_cycle * total_point_weight;
            total_weight += total_point_weight;
        }
        
        if total_weight > 0.0 {
            Ok(weighted_duty / total_weight)
        } else {
            Ok(self.default_duty)
        }
    }
    
    /// Find nearby points for interpolation
    fn find_interpolation_points(&self, rpm: u16, target_boost: f32) -> Vec<&CalibrationPoint> {
        let mut nearby: Vec<&CalibrationPoint> = self.points
            .iter()
            .filter(|p| {
                // Only use points with reasonable confidence
                p.confidence >= 0.3 &&
                // Within reasonable RPM range
                ((p.rpm as i32) - (rpm as i32)).abs() < 2000 &&
                // Within reasonable boost range  
                (p.boost_psi - target_boost).abs() < 5.0
            })
            .collect();
        
        // Sort by distance (RPM distance weighted more heavily)
        nearby.sort_by(|a, b| {
            let a_distance = ((a.rpm as f32) - (rpm as f32)).abs() / 100.0 + 
                           (a.boost_psi - target_boost).abs();
            let b_distance = ((b.rpm as f32) - (rpm as f32)).abs() / 100.0 + 
                           (b.boost_psi - target_boost).abs();
            
            a_distance.partial_cmp(&b_distance).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Use up to 4 closest points for interpolation
        nearby.truncate(4);
        nearby
    }
}

impl Default for LearnedData {
    fn default() -> Self {
        Self {
            calibration_map: CalibrationMap {
                points: Vec::new(),
                bounds: CalibrationBounds {
                    min_rpm: 0,
                    max_rpm: 0,
                    min_boost: 0.0,
                    max_boost: 0.0,
                },
                default_duty: 0.0, // Conservative default
            },
            environmental_factors: EnvironmentalFactors {
                temperature_compensation: 0.0,
                altitude_compensation: 0.0,
                supply_pressure_baseline: 15.0, // PSI
                supply_pressure_compensation: 1.0, // 1% duty per PSI
                environmental_confidence: 0.0,
            },
            confidence_metrics: ConfidenceMetrics {
                overall_confidence: 0.0,
                calibrated_rpm_points: 0,
                calibrated_boost_levels: 0,
                average_point_confidence: 0.0,
                confidence_threshold: 0.5,
            },
            statistics: LearningStatistics {
                total_updates: 0,
                last_learning_session: None,
                successful_runs: 0,
                failed_runs: 0,
                average_accuracy: 0.0,
            },
            last_updated: 0,
        }
    }
}