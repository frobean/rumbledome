# RumbleDome Hardware Specifications

## Hardware Abstraction Layer (HAL) Interface

The HAL provides platform-independent interfaces for all hardware components, enabling support for multiple MCU platforms and hardware configurations.

## Core Hardware Interfaces

### Time Management
```rust
trait Time {
    fn now_ms() -> u32;
    fn sleep_ms(duration: u32);
    fn schedule_callback(delay_ms: u32, callback: fn());
}
```

**Implementation Requirements**:
- Monotonic timestamp source for control loop timing
- Precise delays for calibration sequences  
- Non-blocking sleep implementation for real-time operation

### PWM Control (Solenoid Drive)
```rust
trait Pwm {
    fn set_frequency(freq_hz: u32) -> Result<(), PwmError>;
    fn set_duty_cycle(duty_percent: f32) -> Result<(), PwmError>;
    fn get_current_duty() -> f32;
    fn enable() -> Result<(), PwmError>;
    fn disable() -> Result<(), PwmError>;
}
```

**Solenoid Drive Requirements**:
- **Frequency**: 30 Hz nominal (20-50 Hz acceptable range)
- **Resolution**: 0.1% duty cycle minimum resolution  
- **Response Time**: <10ms for duty cycle changes
- **Drive Current**: Sufficient for 4-port MAC solenoid (typically 2-3A @ 12V)
- **Protection**: Over-current and thermal protection required

### Analog Input (Pressure Sensors)
```rust
trait Analog {
    fn read_channel_mv(channel: u8) -> Result<i32, AnalogError>;
    fn read_multiple_channels(channels: &[u8]) -> Result<Vec<i32>, AnalogError>;
    fn calibrate_channel(channel: u8, ref_voltage: i32) -> Result<(), AnalogError>;
}
```

**Pressure Sensor Requirements**:
- **Input Voltage Range**: 0.5V - 4.5V (ratiometric to 5V supply)
- **Resolution**: 12-bit minimum (4096 steps across range)
- **Accuracy**: ±1% full scale
- **Sample Rate**: 1000 Hz minimum per channel
- **Input Impedance**: >10MΩ to avoid sensor loading
- **Filtering**: Hardware low-pass filtering recommended (100 Hz cutoff)

### Storage (Non-Volatile Memory)
```rust
trait Storage {
    fn read(key: &str) -> Result<Vec<u8>, StorageError>;
    fn write(key: &str, data: &[u8]) -> Result<(), StorageError>;
    fn delete(key: &str) -> Result<(), StorageError>;
    fn list_keys() -> Result<Vec<String>, StorageError>;
    fn get_wear_info() -> WearLevelInfo;
}
```

**Storage Requirements**:
- **Capacity**: 64KB minimum for configuration and learned data
- **Wear Leveling**: Automatic wear leveling to support >10,000 write cycles
- **Retention**: 10 years minimum data retention
- **Write Endurance**: 100,000 write cycles minimum per sector
- **Atomic Operations**: Write operations must be atomic to prevent corruption
- **Backup**: Redundant storage recommended for safety-critical data

### CAN Bus Interface
```rust
trait Can {
    fn init(bitrate: u32) -> Result<(), CanError>;
    fn send_frame(frame: CanFrame) -> Result<(), CanError>;
    fn receive_frame() -> Result<Option<CanFrame>, CanError>;
    fn set_filter(filter: CanFilter) -> Result<(), CanError>;
    fn get_stats() -> CanStats;
}
```

**CAN Bus Requirements**:
- **Bitrate**: 500 kbps (Ford Gen2 Coyote standard)
- **Transceiver**: 3.3V compatible (SN65HVD230 or equivalent)
- **Isolation**: Galvanic isolation recommended for automotive environment
- **Protection**: ESD protection and over-voltage protection required
- **Termination**: Software-configurable 120Ω termination
- **Error Handling**: Automatic error recovery and fault reporting

### Display Interface
```rust
trait Display {
    fn init() -> Result<(), DisplayError>;
    fn clear() -> Result<(), DisplayError>;
    fn draw_pixel(x: u16, y: u16, color: u16) -> Result<(), DisplayError>;
    fn draw_circle(x: u16, y: u16, radius: u16, color: u16) -> Result<(), DisplayError>;
    fn draw_text(x: u16, y: u16, text: &str, font: Font, color: u16) -> Result<(), DisplayError>;
    fn update() -> Result<(), DisplayError>;
}
```

**Display Requirements (ST7735R TFT)**:
- **Resolution**: 128×160 pixels minimum
- **Color Depth**: 16-bit RGB565
- **Interface**: SPI (10 MHz minimum)
- **Update Rate**: 30 Hz minimum for smooth gauge animation
- **Viewing Angle**: Suitable for automotive dashboard mounting
- **Temperature Range**: -20°C to +70°C operating temperature

### GPIO (Digital Inputs)
```rust
trait Gpio {
    fn read_pin(pin: u8) -> Result<bool, GpioError>;
    fn set_pin_mode(pin: u8, mode: PinMode) -> Result<(), GpioError>;
    fn enable_interrupt(pin: u8, trigger: InterruptTrigger, callback: fn()) -> Result<(), GpioError>;
}
```

**Digital Input Requirements**:
- **Input Voltage**: 3.3V/5V tolerant
- **Pull-up/Pull-down**: Configurable internal pull-up/pull-down resistors
- **Debouncing**: Hardware or software debouncing for switch inputs
- **Interrupt Support**: Edge-triggered interrupts for responsive button handling

## System Support Interfaces

### Logging System
```rust
trait Logger {
    fn log(level: LogLevel, message: &str);
    fn set_level(level: LogLevel);
    fn get_logs(count: usize) -> Vec<LogEntry>;
}
```

### Fault Reporting
```rust
trait FaultReporter {
    fn raise_fault(code: FaultCode, message: &str);
    fn clear_fault(code: FaultCode);
    fn get_active_faults() -> Vec<Fault>;
    fn get_fault_history() -> Vec<Fault>;
}
```

### Watchdog Timer
```rust
trait Watchdog {
    fn init(timeout_ms: u32) -> Result<(), WatchdogError>;
    fn feed() -> Result<(), WatchdogError>;
    fn disable() -> Result<(), WatchdogError>;
}
```

## Hardware Platform Specifications

### Primary Platform: Teensy 4.1

**MCU**: NXP iMXRT1062 (ARM Cortex-M7 @ 600 MHz)
- **Flash**: 8MB external flash
- **RAM**: 1MB total (512KB main, 512KB secondary)
- **GPIO**: 55 digital I/O pins
- **ADC**: 2x 12-bit SAR ADCs, up to 3.3V input
- **PWM**: FlexPWM modules with high resolution
- **CAN**: 3x FlexCAN controllers  
- **SPI**: 3x SPI controllers
- **I2C**: 3x I2C controllers
- **UART**: 8x UART controllers

**Pin Assignments**:
```
PWM Output (Solenoid):     Pin 2  (PWM1_A2)
Analog Input 0 (Dome In):  Pin 14 (ADC1_CH0) 
Analog Input 1 (Dome Up):  Pin 15 (ADC1_CH1)
Analog Input 2 (MAP):      Pin 16 (ADC1_CH2)
CAN TX:                    Pin 22 (CAN1_TX)
CAN RX:                    Pin 23 (CAN1_RX)
SPI Display CS:           Pin 10 (CS0)
SPI Display DC:           Pin 9  (GPIO)
SPI Display RST:          Pin 8  (GPIO)
Profile Switch:           Pin 4  (GPIO + Interrupt)
Scramble Button:          Pin 5  (GPIO + Interrupt)
Status LED:               Pin 13 (GPIO)
```

**Power Requirements**:
- **Input Voltage**: 12V automotive (9V-16V range)
- **Regulation**: 5V and 3.3V rails with adequate current capacity
- **Power Consumption**: <5W typical, <10W maximum
- **Protection**: Reverse polarity, over-voltage, and over-current protection

### Future Platform Support

**STM32F4 Series**:
- ARM Cortex-M4 with FPU
- CAN bus support
- Sufficient GPIO and analog inputs
- Real-time performance capability

**ESP32 Series**:
- Dual-core processor with WiFi/Bluetooth
- CAN bus via external transceiver
- Suitable for wireless connectivity features

## Hardware Validation Requirements

### Electrical Testing
- **Power Supply Validation**: Verify regulation and ripple under load
- **Analog Input Accuracy**: Calibrate and validate pressure sensor readings
- **PWM Output Verification**: Confirm duty cycle accuracy and frequency
- **CAN Bus Communication**: Validate message transmission and reception
- **EMI/RFI Testing**: Ensure automotive EMC compliance

### Environmental Testing  
- **Temperature Range**: -20°C to +70°C operation
- **Vibration Resistance**: Automotive vibration standards
- **Moisture Protection**: IP65 rating for enclosure
- **Salt Spray Resistance**: Automotive corrosion standards

### Safety Validation
- **Overboost Response Time**: Measure actual pneumatic response times
- **Failsafe Verification**: Confirm safe operation with power loss
- **Sensor Fault Detection**: Validate sensor failure detection and response
- **CAN Bus Fault Handling**: Test behavior with CAN bus failures

## Sensor Specifications

### Pressure Sensors (0-30 PSI)
- **Output**: 0.5V-4.5V ratiometric to 5V supply
- **Accuracy**: ±0.25% full scale
- **Response Time**: <1ms
- **Thread**: 1/8" NPT male
- **Electrical**: 3-wire configuration (Power, Ground, Signal)
- **Temperature Compensation**: Built-in compensation recommended

**Scaling Formula**:
```
PSI = ((Voltage - 0.5V) / 4.0V) × 30.0 PSI
```

### CAN Transceiver (SN65HVD230)
- **Logic Supply**: 3.3V
- **Bus Voltage**: ±12V
- **Data Rate**: Up to 1 Mbps
- **Standby Mode**: Low-power standby capability  
- **Protection**: ESD protection to ±8kV
- **Package**: SOIC-8 or DIP-8

### Display Module (ST7735R)
- **Controller**: ST7735R
- **Resolution**: 128×160 pixels
- **Interface**: 4-wire SPI
- **Supply Voltage**: 3.3V
- **Backlight**: LED backlight with PWM control
- **Viewing Angle**: 160° typical