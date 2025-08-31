//! Analog input implementation for Teensy 4.1
//! 
//! Provides ADC functionality for reading pressure sensors using the
//! i.MX RT1062 ADC modules.

use crate::traits::AnalogReader;
use crate::types::AnalogChannel;
use crate::error::HalError;

use teensy4_bsp::hal;
use hal::adc;

/// Teensy 4.1 analog input implementation
pub struct Teensy41Analog {
    /// ADC1 instance for pressure sensor readings
    adc1: adc::Adc<adc::module::_1>,
    
    /// ADC2 instance for additional channels
    adc2: adc::Adc<adc::module::_2>,
    
    /// ADC reference voltage
    vref: f32,
    
    /// ADC resolution (12-bit = 4096 counts)
    resolution: u16,
    
    /// Calibration offsets for each channel
    calibration_offsets: [f32; 3],
    
    /// Calibration scales for each channel  
    calibration_scales: [f32; 3],
}

impl Teensy41Analog {
    /// Create new analog input controller
    pub fn new() -> Result<Self, HalError> {
        
        // Initialize ADC1 
        let mut adc1 = hal::adc::Adc::new(
            unsafe { hal::adc::module::_1::new() },
            adc::ClockSelect::default(),
            adc::ClockDivision::default(),
        );
        
        // Initialize ADC2
        let mut adc2 = hal::adc::Adc::new(
            unsafe { hal::adc::module::_2::new() },
            adc::ClockSelect::default(), 
            adc::ClockDivision::default(),
        );
        
        // Configure ADC settings
        adc1.set_resolution(adc::Resolution::Bits12);
        adc1.set_sample_time(adc::SampleTime::Clocks25);
        adc1.set_average_count(adc::AverageCount::Samples4); // 4x averaging for noise reduction
        
        adc2.set_resolution(adc::Resolution::Bits12);
        adc2.set_sample_time(adc::SampleTime::Clocks25);
        adc2.set_average_count(adc::AverageCount::Samples4);
        
        // Calibrate ADCs
        adc1.calibrate()
            .map_err(|e| HalError::adc_error(format!("ADC1 calibration failed: {:?}", e)))?;
        
        adc2.calibrate()
            .map_err(|e| HalError::adc_error(format!("ADC2 calibration failed: {:?}", e)))?;
        
        // Default sensor calibration (will be overridden by configuration)
        let calibration_offsets = [0.5, 0.5, 0.5]; // 0.5V offset for 0-30 PSI sensors
        let calibration_scales = [7.5, 7.5, 7.5];  // 7.5 PSI/V for 0-30 PSI sensors
        
        log::info!("ADC initialized with 12-bit resolution, 4x averaging");
        
        Ok(Self {
            adc1,
            adc2,
            vref: 3.3, // 3.3V reference
            resolution: 4096,
            calibration_offsets,
            calibration_scales,
        })
    }
    
    /// Update sensor calibration parameters
    pub fn set_sensor_calibration(
        &mut self,
        channel: AnalogChannel,
        offset_volts: f32,
        scale_psi_per_volt: f32,
    ) {
        let channel_idx = match channel {
            AnalogChannel::DomeInputPressure => 0,
            AnalogChannel::UpperDomePressure => 1,
            AnalogChannel::ManifoldPressure => 2,
        };
        
        self.calibration_offsets[channel_idx] = offset_volts;
        self.calibration_scales[channel_idx] = scale_psi_per_volt;
        
        log::info!("Updated calibration for {:?}: offset={:.3}V, scale={:.1} PSI/V", 
            channel, offset_volts, scale_psi_per_volt);
    }
    
    /// Get ADC channel number for analog channel
    fn get_adc_channel(&self, channel: AnalogChannel) -> u8 {
        match channel {
            AnalogChannel::DomeInputPressure => 0,  // A0 (Pin 14)
            AnalogChannel::UpperDomePressure => 1,  // A1 (Pin 15)  
            AnalogChannel::ManifoldPressure => 2,   // A2 (Pin 16)
        }
    }
    
    /// Select appropriate ADC for channel
    fn select_adc(&mut self, channel: AnalogChannel) -> &mut adc::Adc<impl adc::Module> {
        match channel {
            AnalogChannel::DomeInputPressure | 
            AnalogChannel::UpperDomePressure => &mut self.adc1,
            AnalogChannel::ManifoldPressure => &mut self.adc2,
        }
    }
}

impl AnalogReader for Teensy41Analog {
    fn read_raw(&mut self, channel: AnalogChannel) -> Result<u16, HalError> {
        let adc_channel = self.get_adc_channel(channel);
        
        // Read from appropriate ADC
        let raw_value = match channel {
            AnalogChannel::DomeInputPressure | 
            AnalogChannel::UpperDomePressure => {
                self.adc1.read_blocking(adc_channel)
                    .map_err(|e| HalError::adc_error(format!("ADC1 read failed: {:?}", e)))?
            },
            AnalogChannel::ManifoldPressure => {
                self.adc2.read_blocking(adc_channel)
                    .map_err(|e| HalError::adc_error(format!("ADC2 read failed: {:?}", e)))?
            }
        };
        
        log::trace!("ADC raw reading for {:?}: {}", channel, raw_value);
        
        Ok(raw_value)
    }
    
    fn read_voltage(&mut self, channel: AnalogChannel) -> Result<f32, HalError> {
        let raw = self.read_raw(channel)?;
        
        // Convert ADC counts to voltage
        let voltage = (raw as f32 / self.resolution as f32) * self.vref;
        
        log::trace!("ADC voltage for {:?}: {:.3}V", channel, voltage);
        
        Ok(voltage)
    }
    
    fn read_pressure_psi(&mut self, channel: AnalogChannel) -> Result<f32, HalError> {
        let voltage = self.read_voltage(channel)?;
        
        // Get calibration parameters for this channel
        let channel_idx = match channel {
            AnalogChannel::DomeInputPressure => 0,
            AnalogChannel::UpperDomePressure => 1,
            AnalogChannel::ManifoldPressure => 2,
        };
        
        let offset = self.calibration_offsets[channel_idx];
        let scale = self.calibration_scales[channel_idx];
        
        // Apply calibration: PSI = (V - offset) * scale  
        let pressure = (voltage - offset) * scale;
        
        // Validate pressure reading is reasonable
        match channel {
            AnalogChannel::DomeInputPressure | AnalogChannel::UpperDomePressure => {
                if pressure < -5.0 || pressure > 50.0 {
                    log::warn!("Pressure reading out of expected range: {:.1} PSI", pressure);
                }
            },
            AnalogChannel::ManifoldPressure => {
                if pressure < -20.0 || pressure > 30.0 {
                    log::warn!("Manifold pressure out of expected range: {:.1} PSI", pressure);
                }
            }
        }
        
        log::trace!("Pressure reading for {:?}: {:.2} PSI", channel, pressure);
        
        Ok(pressure)
    }
}

/// ADC channel configuration for pressure sensors
pub struct PressureSensorConfig {
    /// Zero pressure voltage (V)
    pub zero_voltage: f32,
    
    /// Full scale voltage (V) 
    pub full_scale_voltage: f32,
    
    /// Full scale pressure (PSI)
    pub full_scale_pressure: f32,
}

impl Default for PressureSensorConfig {
    fn default() -> Self {
        // Default for typical 0-30 PSI, 0.5-4.5V sensors
        Self {
            zero_voltage: 0.5,
            full_scale_voltage: 4.5,
            full_scale_pressure: 30.0,
        }
    }
}

impl PressureSensorConfig {
    /// Calculate calibration parameters
    pub fn get_calibration(&self) -> (f32, f32) {
        let offset = self.zero_voltage;
        let scale = self.full_scale_pressure / (self.full_scale_voltage - self.zero_voltage);
        (offset, scale)
    }
}