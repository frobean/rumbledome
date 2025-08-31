//! Mock HAL implementation for testing without hardware

use crate::traits::*;
use crate::types::*;
use crate::error::HalError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock hardware implementation for testing
pub struct MockHal {
    time: MockTime,
    pwm: MockPwm,
    analog: MockAnalog,
    storage: MockStorage,
    can: MockCan,
    display: MockDisplay,
    gpio: MockGpio,
    watchdog: MockWatchdog,
}

impl MockHal {
    /// Create new mock HAL instance
    pub fn new() -> Self {
        Self {
            time: MockTime::new(),
            pwm: MockPwm::new(),
            analog: MockAnalog::new(),
            storage: MockStorage::new(),
            can: MockCan::new(),
            display: MockDisplay::new(),
            gpio: MockGpio::new(),
            watchdog: MockWatchdog::new(),
        }
    }
    
    /// Set simulated sensor values for testing
    pub fn set_sensor_readings(&mut self, readings: SensorReadings) {
        self.analog.set_readings(readings);
    }
    
    /// Set simulated CAN data for testing
    pub fn set_can_data(&mut self, data: CanData) {
        self.can.set_data(data);
    }
    
    /// Get current PWM duty cycle for verification
    pub fn get_pwm_duty(&self, channel: PwmChannel) -> f32 {
        self.pwm.get_duty_cycle(channel).unwrap_or(0.0)
    }
}

impl HalTrait for MockHal {
    type Time = MockTime;
    type Pwm = MockPwm;
    type Analog = MockAnalog;
    type Storage = MockStorage;
    type Can = MockCan;
    type Display = MockDisplay;
    type Gpio = MockGpio;
    type Watchdog = MockWatchdog;
}

/// Mock time provider
pub struct MockTime {
    current_time_ms: Arc<Mutex<u64>>,
}

impl MockTime {
    fn new() -> Self {
        Self {
            current_time_ms: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Advance mock time for testing
    pub fn advance_ms(&mut self, ms: u64) {
        let mut time = self.current_time_ms.lock().unwrap();
        *time += ms;
    }
}

impl TimeProvider for MockTime {
    fn now_ms(&self) -> u64 {
        *self.current_time_ms.lock().unwrap()
    }
    
    fn delay_ms(&mut self, ms: u32) {
        self.advance_ms(ms as u64);
    }
    
    fn timestamp_us(&self) -> u64 {
        self.now_ms() * 1000
    }
}

/// Mock PWM controller
pub struct MockPwm {
    duty_cycles: HashMap<PwmChannel, f32>,
    frequencies: HashMap<PwmChannel, u16>,
    enabled: HashMap<PwmChannel, bool>,
}

impl MockPwm {
    fn new() -> Self {
        Self {
            duty_cycles: HashMap::new(),
            frequencies: HashMap::new(),
            enabled: HashMap::new(),
        }
    }
}

impl PwmController for MockPwm {
    fn set_duty_cycle(&mut self, channel: PwmChannel, duty_percent: f32) -> Result<(), HalError> {
        if duty_percent < 0.0 || duty_percent > 100.0 {
            return Err(HalError::invalid_parameter(format!(
                "Invalid duty cycle: {}% (must be 0-100%)", duty_percent
            )));
        }
        
        self.duty_cycles.insert(channel, duty_percent);
        log::debug!("Mock PWM {:?}: {}% duty cycle", channel, duty_percent);
        Ok(())
    }
    
    fn set_frequency(&mut self, channel: PwmChannel, frequency_hz: u16) -> Result<(), HalError> {
        self.frequencies.insert(channel, frequency_hz);
        log::debug!("Mock PWM {:?}: {} Hz frequency", channel, frequency_hz);
        Ok(())
    }
    
    fn enable(&mut self, channel: PwmChannel) -> Result<(), HalError> {
        self.enabled.insert(channel, true);
        log::debug!("Mock PWM {:?}: enabled", channel);
        Ok(())
    }
    
    fn disable(&mut self, channel: PwmChannel) -> Result<(), HalError> {
        self.enabled.insert(channel, false);
        self.duty_cycles.insert(channel, 0.0); // Failsafe
        log::debug!("Mock PWM {:?}: disabled (failsafe)", channel);
        Ok(())
    }
    
    fn get_duty_cycle(&self, channel: PwmChannel) -> Result<f32, HalError> {
        Ok(self.duty_cycles.get(&channel).copied().unwrap_or(0.0))
    }
}

/// Mock analog reader
pub struct MockAnalog {
    readings: SensorReadings,
}

impl MockAnalog {
    fn new() -> Self {
        Self {
            readings: SensorReadings::default(),
        }
    }
    
    fn set_readings(&mut self, readings: SensorReadings) {
        self.readings = readings;
    }
}

impl AnalogReader for MockAnalog {
    fn read_raw(&mut self, channel: AnalogChannel) -> Result<u16, HalError> {
        // Simulate 12-bit ADC (0-4095)
        let voltage = self.read_voltage(channel)?;
        let raw = ((voltage / 5.0) * 4095.0) as u16;
        Ok(raw)
    }
    
    fn read_voltage(&mut self, channel: AnalogChannel) -> Result<f32, HalError> {
        let pressure = match channel {
            AnalogChannel::DomeInputPressure => self.readings.dome_input_pressure,
            AnalogChannel::UpperDomePressure => self.readings.upper_dome_pressure,
            AnalogChannel::ManifoldPressure => self.readings.manifold_pressure_gauge,
        };
        
        // Convert PSI back to voltage for 0-30 PSI sensor
        // PSI = ((Vout - 0.5) / 4.0) * 30
        // Vout = (PSI / 30) * 4.0 + 0.5
        let voltage = (pressure / 30.0) * 4.0 + 0.5;
        Ok(voltage.clamp(0.0, 5.0))
    }
    
    fn read_pressure_psi(&mut self, channel: AnalogChannel) -> Result<f32, HalError> {
        let pressure = match channel {
            AnalogChannel::DomeInputPressure => self.readings.dome_input_pressure,
            AnalogChannel::UpperDomePressure => self.readings.upper_dome_pressure,
            AnalogChannel::ManifoldPressure => self.readings.manifold_pressure_gauge,
        };
        Ok(pressure)
    }
}

/// Mock storage implementation
pub struct MockStorage {
    data: Vec<u8>,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            data: vec![0; 64 * 1024], // 64KB mock storage
        }
    }
}

impl NonVolatileStorage for MockStorage {
    fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), HalError> {
        let start = address as usize;
        let end = start + buffer.len();
        
        if end > self.data.len() {
            return Err(HalError::invalid_parameter("Address out of range"));
        }
        
        buffer.copy_from_slice(&self.data[start..end]);
        Ok(())
    }
    
    fn write(&mut self, address: u32, data: &[u8]) -> Result<(), HalError> {
        let start = address as usize;
        let end = start + data.len();
        
        if end > self.data.len() {
            return Err(HalError::invalid_parameter("Address out of range"));
        }
        
        self.data[start..end].copy_from_slice(data);
        Ok(())
    }
    
    fn erase_sector(&mut self, address: u32) -> Result<(), HalError> {
        // Mock 4KB sectors
        let sector_start = (address & !0xFFF) as usize;
        let sector_end = sector_start + 4096;
        
        if sector_end > self.data.len() {
            return Err(HalError::invalid_parameter("Sector out of range"));
        }
        
        self.data[sector_start..sector_end].fill(0xFF);
        Ok(())
    }
    
    fn capacity(&self) -> u32 {
        self.data.len() as u32
    }
    
    fn maintenance(&mut self) -> Result<(), HalError> {
        // No-op for mock
        Ok(())
    }
}

/// Mock CAN bus implementation
pub struct MockCan {
    data: CanData,
    connected: bool,
    error_stats: CanErrorStats,
}

impl MockCan {
    fn new() -> Self {
        Self {
            data: CanData::default(),
            connected: true,
            error_stats: CanErrorStats::default(),
        }
    }
    
    fn set_data(&mut self, data: CanData) {
        self.data = data;
    }
}

impl CanBus for MockCan {
    fn send(&mut self, _message: &CanMessage) -> Result<(), HalError> {
        if !self.connected {
            return Err(HalError::CanError("Bus not connected".into()));
        }
        Ok(())
    }
    
    fn receive(&mut self) -> Result<Option<CanMessage>, HalError> {
        if !self.connected {
            return Err(HalError::CanError("Bus not connected".into()));
        }
        
        // Return simulated torque data
        let message = CanMessage {
            id: 0x123, // Placeholder CAN ID
            data: vec![0; 8], // TODO: Encode actual torque data
            extended: false,
            rtr: false,
        };
        
        Ok(Some(message))
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
    
    fn get_error_stats(&self) -> CanErrorStats {
        self.error_stats.clone()
    }
    
    fn reset(&mut self) -> Result<(), HalError> {
        self.error_stats = CanErrorStats::default();
        Ok(())
    }
}

// Additional mock implementations for Display, GPIO, and Watchdog...
// (Abbreviated for brevity - would implement similar pattern)

/// Mock display controller
pub struct MockDisplay;
impl MockDisplay {
    fn new() -> Self { Self }
}
impl DisplayController for MockDisplay {
    fn clear(&mut self) -> Result<(), HalError> { Ok(()) }
    fn draw_text(&mut self, _x: u16, _y: u16, _text: &str, _font_size: FontSize) -> Result<(), HalError> { Ok(()) }
    fn fill_rect(&mut self, _x: u16, _y: u16, _width: u16, _height: u16, _color: Color) -> Result<(), HalError> { Ok(()) }
    fn draw_line(&mut self, _x1: u16, _y1: u16, _x2: u16, _y2: u16, _color: Color) -> Result<(), HalError> { Ok(()) }
    fn draw_circle(&mut self, _x: u16, _y: u16, _radius: u16, _color: Color) -> Result<(), HalError> { Ok(()) }
    fn update(&mut self) -> Result<(), HalError> { Ok(()) }
    fn set_brightness(&mut self, _percent: u8) -> Result<(), HalError> { Ok(()) }
}

/// Mock GPIO controller
pub struct MockGpio;
impl MockGpio {
    fn new() -> Self { Self }
}
impl GpioController for MockGpio {
    fn read_pin(&mut self, _pin: GpioPin) -> Result<bool, HalError> { Ok(false) }
    fn write_pin(&mut self, _pin: GpioPin, _state: bool) -> Result<(), HalError> { Ok(()) }
    fn set_input_pullup(&mut self, _pin: GpioPin) -> Result<(), HalError> { Ok(()) }
    fn set_output(&mut self, _pin: GpioPin) -> Result<(), HalError> { Ok(()) }
}

/// Mock watchdog timer
pub struct MockWatchdog;
impl MockWatchdog {
    fn new() -> Self { Self }
}
impl WatchdogTimer for MockWatchdog {
    fn start(&mut self, _timeout_ms: u32) -> Result<(), HalError> { Ok(()) }
    fn feed(&mut self) -> Result<(), HalError> { Ok(()) }
    fn stop(&mut self) -> Result<(), HalError> { Ok(()) }
    fn is_running(&self) -> bool { false }
}

impl Default for MockHal {
    fn default() -> Self {
        Self::new()
    }
}