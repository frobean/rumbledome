# RumbleDome Hardware Specifications

📋 **Hardware specifications and pin assignments**: See **[TechnicalSpecs.md](TechnicalSpecs.md)** for complete technical details

## Hardware Abstraction Layer (HAL) Interface

**🔗 T2-HAL-001**: **Platform-Independent Hardware Abstraction Design**  
**Decision Type**: ⚠️ **Engineering Decision** - Multi-platform support architecture  
**Derived From**: T1-INNOVATION-001 (Torque Request Amplification Paradigm) - system must work across different hardware platforms  
**Engineering Rationale**: HAL abstraction enables desktop simulation, testing, and future platform expansion without core logic changes  
**AI Traceability**: Drives interface definitions, mock implementations, platform-specific adapters

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

**4-Port MAC Solenoid Specifications**:

**⚙️ Solenoid Characteristics**:
- **Type**: 4-port MAC valve (pneumatic boost control)
- **Configuration**: Air supply → solenoid → wastegate actuator domes
- **Control Method**: PWM duty cycle modulates air flow distribution
- **Fail-Safe Design**: 0% duty = wastegate open (minimal boost)
- **Operating Principle**: Proportional control of wastegate dome pressure balance

**🔗 T2-HAL-004**: **4-Port MAC Solenoid Drive Requirements**  
**Derived From**: T2-SOLENOID-001 (4-Port MAC Solenoid Selection) + automotive operating conditions  
**Decision Type**: ⚠️ **Engineering Decision** - Driver circuit specification for automotive environment  
**Engineering Rationale**: 12V automotive supply with wide tolerance, high-current MOSFET drive for reliable solenoid operation  
**AI Traceability**: Drives PWM driver design, current limiting, thermal management

**🔌 Electrical Drive Requirements**:
- **Supply Voltage**: 12V nominal (9-16V operating range)
- **Current Draw**: 2-3A typical at 12V (varies by solenoid model)
- **PWM Frequency**: 30 Hz nominal (20-50 Hz acceptable range)
- **Duty Cycle Range**: 0-100% with 0.1% minimum resolution
- **Response Time**: <10ms for duty cycle changes
- **Drive Circuit**: MOSFET-based high-side switching with flyback diode protection

**🛡️ Protection and Safety**:
- **Over-Current Protection**: Electronic current limiting to prevent driver damage
- **Thermal Protection**: Temperature monitoring of driver circuit
- **Short Circuit Protection**: Automatic shutdown on solenoid coil short
- **Flyback Protection**: Diode clamping for inductive kickback suppression
- **Fault Detection**: Current monitoring for open coil or short circuit conditions

**⚡ Driver Circuit Specifications**:
- **Switch Type**: N-channel MOSFET (RDS(on) <10mΩ)
- **Current Rating**: 5A minimum continuous, 10A peak
- **Voltage Rating**: 40V minimum (automotive transient protection)
- **Gate Drive**: Logic-level compatible with 3.3V MCU outputs
- **Thermal Management**: Heat sink required for continuous operation >2A

**📊 Performance Characteristics**:
- **Linearity**: Proportional relationship between duty cycle and flow rate
- **Repeatability**: ±1% duty cycle accuracy over temperature range
- **Temperature Coefficient**: <0.1%/°C drift over -40°C to +85°C range
- **Mechanical Durability**: >10 million switching cycles at rated conditions
- **Environmental Rating**: IP67 sealed connector required for automotive use

### Analog Input (Pressure Sensors)
```rust
trait Analog {
    fn read_channel_mv(channel: u8) -> Result<i32, AnalogError>;
    fn read_multiple_channels(channels: &[u8]) -> Result<Vec<i32>, AnalogError>;
    fn calibrate_channel(channel: u8, ref_voltage: i32) -> Result<(), AnalogError>;
}
```

**Pressure Sensor Requirements**:
- **Sensor Output**: 0.5V - 4.5V (ratiometric to 5V supply)
- **Teensy Input Range**: 0.33V - 2.97V (after 5V→3.3V voltage divider)
- **Voltage Divider**: Precision resistors, 3.3V/5V = 0.66 ratio
- **Resolution**: 12-bit minimum (4096 steps across 2.64V span)
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

**💾 Storage Architecture (MicroSD Card Primary)**:

**🗂️ SD Card File Structure**:
```
/RUMBLEDOME/
├── config/
│   └── user_config.json        # Only 5 parameters: aggression, spring_pressure, max_boost_psi, overboost_limit, scramble_enabled
├── learned/
│   ├── calibration_maps.bin     # Duty cycle calibration tables (RPM × Boost → Duty)
│   ├── environmental.json       # Temperature/altitude compensation factors
│   ├── sensor_fusion.json       # CAN MAP vs boost gauge cross-calibration
│   └── safety_params.json       # Learned safety characteristics and response times
├── backups/
│   └── [timestamp]/             # Automatic rolling backups
└── logs/
    └── [date]/                  # Diagnostic and safety event logs
```

**📊 Write Optimization Strategy**:
- **Debounced Writes**: All data writes debounced 5-10 seconds to optimize SD card wear
- **Atomic Operations**: Crash-safe writes using temp file + rename operations
- **Change Detection**: Only persist actual configuration changes, not repeated identical writes
- **Batch Learning Updates**: Learning system accumulates changes before writing
- **Automatic Backups**: Rolling backup system prevents data loss from card failures

**🔍 SD Card Health Monitoring**:
- **Health Status Tracking**: Monitor for card errors, corruption, and write failures
- **User Warnings**: Display notifications for SD card errors or impending failures  
- **Graceful Degradation**: System operates safely with default parameters if SD card fails
- **Emergency Recovery**: System continues operation without SD card using firmware defaults

**Technical Specifications**:
- **Capacity**: 8-32GB MicroSD (Class 10 or better recommended)
- **Filesystem**: FAT32 for maximum compatibility and easy access
- **Write Strategy**: Debounced persistence with atomic file operations
- **Expected Lifespan**: 10+ years with proper write management and wear leveling
- **Failure Recovery**: System continues with defaults if SD card unavailable

### MicroSD Card Storage (Portable Configuration)

**🔗 T2-HAL-002**: **Portable Storage Interface Design**  
**Decision Type**: 🔗 **Direct Derivation** - Implementation of configuration portability  
**Derived From**: T2-STORAGE-001 (SD Card Primary Storage) + T1-UI-001 (Single Parameter Philosophy)  
**AI Traceability**: Drives storage management algorithms, configuration persistence, backup strategies

```rust
trait PortableStorage {
    fn mount(&mut self) -> Result<(), HalError>;
    fn unmount(&mut self) -> Result<(), HalError>;
    fn is_mounted(&self) -> bool;
    fn read_config_file(&mut self, filename: &str) -> Result<Vec<u8>, HalError>;
    fn write_config_file(&mut self, filename: &str, data: &[u8]) -> Result<(), HalError>;
    fn load_user_config(&mut self) -> Result<UserConfiguration, HalError>;
    fn save_user_config(&mut self, config: &UserConfiguration) -> Result<(), HalError>;
    fn get_card_info(&self) -> Result<SdCardInfo, HalError>;
}

**🔗 T2-HAL-003**: **5-Parameter Configuration Structure**  
**Decision Type**: 🔗 **Direct Derivation** - Software implementation of single-knob philosophy  
**Derived From**: T1-UI-001 (Single Parameter Philosophy)  
**AI Traceability**: Drives configuration data structures, parameter validation, user interface

// Simple 5-parameter configuration structure  
pub struct UserConfiguration {
    pub aggression: f32,              // 0.0-1.0 - scales all system behavior
    pub spring_pressure: f32,         // PSI - wastegate spring pressure  
    pub max_boost_psi: f32,          // PSI - performance ceiling
    pub overboost_limit: f32,        // PSI - hard safety limit
    pub scramble_enabled: bool,       // Enable scramble button feature
}
```

**Single-Tier SD Card Storage**:
- **User Configuration**: Simple 5-parameter settings stored in user_config.json
- **Learned Data**: Hardware-specific calibration maps and environmental factors  
- **Safety Logs**: Diagnostic and fault history for analysis
- **Automatic Backups**: Rolling backups for data protection

**Configuration Benefits**:
- **Portability**: Same SD card works across multiple controller units
- **Rapid Replacement**: Swap SD card to new micro → instant configuration access
- **Version Control**: Text-based JSON files compatible with git and external editing
- **Emergency Recovery**: Physical SD card survives controller failures
- **Development Flexibility**: Easy configuration management and testing

### CAN Bus Interface
```rust
trait Can {
    fn init(bitrate: u32) -> Result<(), CanError>;
    fn send_frame(frame: CanFrame) -> Result<(), CanError>;
    fn receive_frame() -> Result<Option<CanFrame>, CanError>;
    fn set_filter(filter: CanFilter) -> Result<(), CanError>;
    fn get_stats() -> CanStats;
    
    // Ford S550-specific signal decoding
    fn read_rpm(&mut self) -> Result<u16, CanError>;
    fn read_desired_torque(&mut self) -> Result<f32, CanError>;      // TBD - need to identify signal
    fn read_actual_torque(&mut self) -> Result<f32, CanError>;       // TBD - need to identify signal  
    fn read_manifold_pressure(&mut self) -> Result<f32, CanError>;
}
```

**CAN Bus Requirements**:
- **Bitrate**: 500 kbps (Ford Gen2 Coyote standard)
- **Transceiver**: 3.3V compatible (SN65HVD230 or equivalent)
- **Isolation**: Galvanic isolation recommended for automotive environment
- **Protection**: ESD protection and over-voltage protection required
- **Termination**: Software-configurable 120Ω termination
- **Error Handling**: Automatic error recovery and fault reporting

**🔗 T2-HAL-005**: **Ford S550 CAN Signal Integration**  
**Derived From**: FR-1 (ECU Integration & Cooperation) + vehicle platform requirements  
**Decision Type**: 🚧 **TBD** - CAN signal mapping requires vehicle testing for validation  
**Engineering Notes**: RPM confirmed, torque signals require identification through testing  
**AI Traceability**: Drives CAN protocol implementation, signal validation, torque extraction

**Ford S550 CAN Signal Integration**:
- **RPM (0x109)**: `(b0<<8 + b1) / 4` - Engine speed for learned calibration lookups
- **Manifold Pressure (0x167)**: `((b5-25)<<8 + b6 - 128) / 5` - CAN MAP sensor for safety monitoring
- **Torque Signals (TBD)**: Need to determine which of these represents desired vs actual torque:
  - **Signal A (0x167)**: `((b1-128)<<8 + b2) / 4` - "Engine load/torque" 
  - **Signal B (0x43E)**: `(b5<<8 + b6) / 72 - 140` - "Engine load percentage"

**Signal Validation Requirements**:
- **Torque Signal Identification**: Test 0x167 and 0x43E to determine desired vs actual torque
- **Update Rate Verification**: Measure actual CAN message frequencies for control loop timing
- **Signal Accuracy**: Cross-reference with HPTuners data if available for validation
- **Behavioral Analysis**: Desired torque should lead actual torque during acceleration events

**Control Loop Requirements**:
- **Minimum Update Rate**: 20-50Hz for smooth torque-following control
- **Maximum Latency**: <50ms from ECU torque change to boost response
- **Fault Detection**: CAN timeout detection with <500ms failsafe response

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

**🔗 T2-HAL-006**: **Wireless CLI Console Access**  
**Derived From**: T1-UI-001 (Single Parameter Philosophy) + mobile accessibility requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Bluetooth SPP abstraction layer for wireless console access  
**Engineering Rationale**: Same CLI commands over wireless eliminates duplicate interfaces, maintains consistency  
**AI Traceability**: Drives wireless protocol abstraction, mobile app design, command consistency

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
    ├─ Config Manager ──────────→ │ "rumbledome config --knob 0.7"  
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
- **Integration**: Mounted in 60mm gauge pod with rotary encoder bezel

### Single-Knob Control Interface
```rust
trait RotaryEncoder {
    fn read_position() -> Result<i32, EncoderError>;
    fn reset_position() -> Result<(), EncoderError>;
    fn get_click_count() -> u32;
    fn has_button() -> bool;
    fn is_button_pressed() -> bool;
}
```

**Revolutionary Control Design**:
- **Hardware**: Rotary encoder with tactile detents (no mechanical limits)
- **Integration**: Bezel around gauge pod rotates as the control knob
- **Mechanical Coupling**: Offset gear-driven design for clean wire routing
- **Resolution**: 100 clicks for 0-100% range (1% per click precision)
- **Feedback**: Mechanical detent click confirms each adjustment
- **Durability**: No potentiometer wear, automotive-grade encoder (IP67)
- **UX**: Knob position directly correlates with visual display background

**Offset Gear-Driven Encoder Mechanism**:
```
    ┌─────────────────┐ Outer Bezel
    │  ┌───────────┐  │ 
    │  │  DISPLAY  │  │ <- Wires route straight through center
    │  │           │  │
    │  └───────────┘  │
    │              ⚙  │ <- Encoder offset near pod edge
    └─────────────────┘
```

**Mechanical Implementation**:
- **Bezel Design**: Internal gear teeth around inner circumference of rotating bezel
- **Encoder Coupling**: Small pinion gear on standard rotary encoder shaft
- **Positioning**: Encoder mounted offset near gauge pod edge, avoids center wire routing
- **Gear Engagement**: Meshing teeth provide positive mechanical coupling (no slip/backlash)
- **Gear Ratio Options**: 1:1 direct drive or 2:1/3:1 reduction for finer control resolution
- **3D Printable**: Both bezel ring teeth and encoder pinion gear suitable for FDM printing

**Manufacturing Advantages**:
- **Clean center routing**: Display wires have completely unobstructed path to PCB
- **Standard components**: Uses off-the-shelf rotary encoder, no custom electronics
- **3D printer compatible**: Gear teeth print well with 45° chamfers, no supports needed
- **Positive engagement**: Gear teeth eliminate slip, provide precise position feedback
- **Balanced rotation**: Offset encoder mass provides better mechanical balance than center-mounted
- **Serviceable**: Encoder can be replaced without disturbing display wiring

**Scramble Button Override**:
- **Function**: Instant brimstone (100%) aggression override regardless of knob position
- **Type**: Momentary push button (normally open)
- **Mounting**: Separate button on gauge pod or easily accessible dashboard location
- **Visual Feedback**: Display shows "SCRAMBLE" overlay with flashing red background when active
- **Behavior**: Hold for override, release returns to current knob setting
- **Safety**: No latching - requires continuous pressure for activation

**Dynamic UI Integration**:
- **0-25%**: Green background with gentle pulse animation ("Puppy Dog")
- **25-50%**: Green→Amber gradient with subtle glow ("Daily Driver") 
- **50-75%**: Amber→Red gradient with active pulse ("Spirited")
- **75-100%**: Red background with animated flame effects ("Brimstone")

**Implementation Benefits**:
- **No mechanical limits**: Infinite rotation prevents breakage
- **Precise control**: Exact percentage setting via click counting
- **Visual cohesion**: Knob position matches screen theme seamlessly
- **Premium feel**: Industrial design integration vs aftermarket add-on
- **Immediate feedback**: Background changes in real-time as knob turns
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

**Pin Assignments**: See **[TechnicalSpecs.md](TechnicalSpecs.md)** for complete pin mapping

**Power and Environmental Specifications**: See **[TechnicalSpecs.md](TechnicalSpecs.md)**
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

### Manifold Pressure Sensor Strategy

**Dual-Sensor Approach for Full Vacuum→Boost Range**:

**CAN MAP Sensor (OEM)**:
- **Range**: 0-1 bar absolute (vacuum only: -14.7 to 0 PSI gauge)
- **Optimal**: Deep vacuum conditions (idle, deceleration)
- **Data Source**: CAN bus from ECU
- **Resolution**: Varies by ECU (typically 0.1-0.2 PSI)

**Added Boost Gauge Sensor**:
- **Range**: 0-30 PSI gauge (boost measurement)
- **Sensor Output**: 0.5V-4.5V ratiometric to 5V supply
- **Teensy ADC Input**: 0.167V-1.5V (2kΩ+1kΩ voltage divider, 0.333 ratio)
- **Scaling Formula**: `PSI = ((Vout - 0.167) / 1.33) * 30`
- **Optimal**: Boost conditions (positive manifold pressure)
- **Resolution**: 0.018 PSI with 12-bit ADC
- **Thread**: 1/8" NPT male

**Manifold Pressure Sensor Fusion Logic**:
> **Note**: Dual sensor fusion applies ONLY to manifold pressure measurement. Dome feed line and upper dome pressure sensors use single dedicated gauges.

- **Deep vacuum** (-5 PSI to -1 PSI): CAN MAP sensor primary
- **Transition zone** (±1 PSI around atmospheric): Blended reading  
- **Boost range** (+1 PSI to +30 PSI): Boost gauge sensor primary
- **Cross-calibration learning**: Automatically learns offset between sensors in overlap zone
- **Dynamic offset compensation**: Continuously adjusts for systematic sensor differences
- **Seamless operation**: No faults for sensor disagreement - system adapts and learns
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