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
trait NonVolatileStorage {
    fn read(&mut self, offset: usize, buffer: &mut [u8]) -> Result<usize, HalError>;
    fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), HalError>;
    fn erase_all(&mut self) -> Result<(), HalError>;
    fn sync(&mut self) -> Result<(), HalError>;
    fn get_size(&self) -> usize;
    fn get_health_report(&self, current_time: u64) -> StorageHealthReport;
}

// Comprehensive wear tracking for predictive maintenance
struct StorageHealthReport {
    overall_health: StorageHealth,
    section_health: SectionHealthReport,
    estimated_lifespan_years: f32,
    most_worn_region: RegionWearInfo,
    write_statistics: WriteStatistics,
    health_summary: String,
    recommendations: Vec<String>,
}
```

**Automotive Storage Requirements**:

**🚗 Power Loss Reality**:
- **No Graceful Shutdown**: Key-off events cause instant power loss without warning
- **Immediate Writes**: All write operations must persist immediately (no caching/deferred writes)
- **Write-Through Strategy**: Every storage write must complete before returning success
- **No Dependency on Drop/Destructors**: Cannot rely on cleanup routines that won't execute

**📊 Comprehensive Wear Tracking**:
- **Per-Region Monitoring**: 8 regions of 512 bytes each, individually tracked
- **Write Cycle Counting**: Track write cycles per region (0-100,000 limit)
- **Health Status Classification**: Excellent → Good → Warning → Critical → Failed
- **Predictive Analysis**: Estimate remaining lifespan based on usage patterns
- **Human-Readable Reporting**: Console and GUI health reports with clear recommendations

**💾 Storage Architecture (Teensy 4.1 FlexRAM EEPROM)**:
```
├── Configuration     [   0 -  512] →  512 bytes (system config, profiles)
├── Learned Data      [ 512 - 2560] → 2048 bytes (calibration maps, environmental factors) 
├── Calibration       [2560 - 3584] → 1024 bytes (sensor calibration, auto-cal state)
└── Safety Log        [3584 - 4096] →  512 bytes (fault history, safety events)
```

**⚠️ Wear Management Strategy**:
- **Write Rate Limiting**: Learning system must minimize write frequency 
- **Batch Updates**: Only persist significant changes, not every minor adjustment
- **Confidence Thresholds**: Only write calibration data above confidence thresholds
- **Time-Based Persistence**: Maximum write rate limits (e.g., 1 write per minute during learning)

**🔍 Health Monitoring Features**:
- **Real-Time Tracking**: Write counts, timestamps, average write sizes per region
- **Proactive Warnings**: Alerts at 80% wear (years before failure)
- **Critical Notifications**: Urgent alerts at 95% wear with replacement timeline
- **Usage Pattern Analysis**: Peak write rates, session statistics, uptime correlation
- **Lifespan Estimation**: Predictive modeling based on current usage patterns

**Technical Specifications**:
- **Capacity**: 4KB EEPROM emulation via FlexRAM
- **Write Endurance**: 100,000 cycles per 512-byte region (conservative estimate)
- **Retention**: 10+ years minimum data retention
- **Write Speed**: Immediate persistence (no caching delays)
- **Expected Lifespan**: 15-30 years with normal driving patterns
- **Failure Prediction**: 2-5 years advance warning before wear-out

### MicroSD Card Storage (Portable Configuration)
```rust
trait PortableStorage {
    fn mount(&mut self) -> Result<(), HalError>;
    fn unmount(&mut self) -> Result<(), HalError>;
    fn is_mounted(&self) -> bool;
    fn read_config_file(&mut self, filename: &str) -> Result<Vec<u8>, HalError>;
    fn write_config_file(&mut self, filename: &str, data: &[u8]) -> Result<(), HalError>;
    fn load_user_profiles(&mut self) -> Result<UserProfileSet, HalError>;
    fn save_user_profiles(&mut self, profiles: &UserProfileSet) -> Result<(), HalError>;
    fn get_card_info(&self) -> Result<SdCardInfo, HalError>;
}

// Portable configuration structure
pub struct UserProfileSet {
    pub metadata: ProfileSetMetadata,
    pub profiles: Vec<BoostProfile>,
    pub sensor_calibrations: SensorCalibrations,
    pub safety_limits: UserSafetyLimits,
    pub system_preferences: SystemPreferences,
}
```

**MicroSD Storage Architecture**:

**🎯 Two-Tier Storage Strategy**:
- **EEPROM (Instance-Specific)**: Learned data, auto-calibration progress, storage wear tracking
- **MicroSD (Portable)**: User profiles, sensor calibrations, safety limits, system preferences

**📁 SD Card File Structure**:
```
/RUMBLEDOME/
├── profiles/
│   ├── daily_driver.json        # User boost profiles
│   ├── sport_mode.json
│   ├── track_day.json
│   └── valet_mode.json
├── config/
│   ├── sensor_calibrations.json # Pressure sensor parameters
│   ├── safety_limits.json       # User safety boundaries  
│   └── system_preferences.json  # Display/CAN/UI settings
├── backups/
│   ├── 2025-01-15_baseline.bak  # Full system backups
│   └── 2025-01-20_tuned.bak
└── firmware/
    └── updates/                 # Future firmware updates
```

**Configuration Resolution Priority**:
1. **SD Card Profiles**: User-defined boost profiles and preferences
2. **EEPROM Learned Data**: Hardware-specific calibration maps and wear tracking
3. **Firmware Defaults**: Factory fallbacks if storage unavailable

**Benefits**:
- **Hardware Independence**: Same SD card works across multiple micros
- **Rapid Replacement**: Swap SD card to new micro → instant profile access
- **Version Control**: Text-based JSON files compatible with git
- **Emergency Backup**: Physical SD card survives micro failures
- **Development Flexibility**: Easy bulk configuration management

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

### Bluetooth Serial Interface (Wireless Console Access)
```rust
trait BluetoothSerial {
    fn init(&mut self, name: &str, pin: &str) -> Result<(), HalError>;
    fn is_connected(&self) -> bool;
    fn send(&mut self, data: &[u8]) -> Result<(), HalError>;
    fn receive(&mut self) -> Result<Vec<u8>, HalError>;
    fn get_connection_info(&self) -> Result<BluetoothConnectionInfo, HalError>;
}

pub struct BluetoothConnectionInfo {
    pub connected_device: Option<String>,
    pub signal_strength: i8,
    pub connection_duration: u32,
    pub bytes_transferred: u64,
}
```

**Bluetooth Architecture**:

**🎯 Wireless Serial Port Abstraction**:
- **Primary Purpose**: Wireless access to CLI console interface
- **Protocol**: Standard serial communication over Bluetooth Classic (SPP)
- **Transparency**: Bluetooth connection identical to USB-C serial connection
- **Mobile App**: GUI wrapper around existing CLI commands

**📱 Mobile App = Wireless CLI Client**:
```
Mobile App                    Teensy 4.1 Console
    │                             │
    ├─ GUI Button ──────────────→ │ "rumbledome backup"
    ├─ Profile Manager ─────────→ │ "rumbledome config --profile sport"  
    ├─ Live Telemetry ──────────→ │ "rumbledome status --live --format json"
    └─ Backup/Restore ──────────→ │ "rumbledome restore --backup-file tune.json"
                                  │
         Bluetooth SPP            │ Same exact command processing
         (Wireless Serial)        │ Same exact responses
                                  │ Same exact SD card access
```

**Command Translation Examples**:
| Mobile App Action | CLI Command Sent Over Bluetooth |
|------------------|----------------------------------|
| "Download Config" | `rumbledome backup --output mobile_backup.json` |
| "Upload New Tune" | `rumbledome restore --backup-file uploaded.json` |
| "Switch to Sport Mode" | `rumbledome config --profile sport` |
| "View Storage Health" | `rumbledome diagnostics --eeprom-report` |
| "Live Boost Reading" | `rumbledome status --live --format json` |

**Benefits**:
- **Single Interface**: CLI commands work identically over USB-C or Bluetooth
- **No Duplicate Code**: Same command parsing, same functionality
- **Development Consistency**: Debug with USB-C, deploy with Bluetooth
- **Robust Fallback**: Bluetooth failure → use USB-C cable
- **Security**: Standard Bluetooth pairing controls access
- **Emergency Access**: Physical USB-C always available

**Technical Specifications**:
- **Protocol**: Bluetooth Classic 2.1+ with SPP (Serial Port Profile)
- **Range**: 10+ meters typical indoor range
- **Security**: Bluetooth pairing + optional PIN authentication
- **Power**: Low power consumption in standby mode
- **Compatibility**: Standard Bluetooth serial - works with any terminal app
- **Fallback**: USB-C serial always available for emergency access

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