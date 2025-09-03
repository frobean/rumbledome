# RumbleDome Technical Specifications

This document serves as the **single source of truth** for all technical specifications in the RumbleDome project. All other documentation references these specifications.

## 🏗️ Tier 2: Implementation Design Document

**🔗 Dependencies:** 
- **Tier 1**: Context.md (design goals), Requirements.md (functional specs), Safety.md (constraints)
- **Constraints**: Physics.md (turbo physics), CAN_Signals.md (vehicle integration)

**📤 Impacts:** Changes to hardware specs here require review of:
- **Tier 2**: Hardware.md (HAL interfaces), Architecture.md (system design)
- **Tier 3**: Implementation.md (build process), TestPlan.md (hardware validation)

## 🔄 Change Impact Checklist
Before modifying this document:
- [ ] **⚠️ TIER 2 CHANGE**: This affects hardware implementation specifications
- [ ] Verify consistency with Tier 1 dependencies: Context.md, Requirements.md, Safety.md
- [ ] Check constraints: Physics.md, CAN_Signals.md
- [ ] Review impacted Tier 2: Hardware.md, Architecture.md  
- [ ] Review impacted Tier 3: Implementation.md, TestPlan.md
- [ ] Update pin assignments and electrical specifications consistently
- [ ] Add new hardware concepts to Definitions.md if applicable

📖 **For terminology**: See **[Definitions.md](Definitions.md)** for technical acronyms and hardware concepts used in these specifications

---

## Hardware Platform

### Microcontroller

**🔗 T2-MCU-001**: **Teensy 4.1 Platform Selection**  
**Derived From**: T1-BEHAVIOR-001 (Universal Behavioral Scaling) + T1-SAFETY-002 (Defense in Depth)  
**Decision Type**: ⚠️ **Engineering Decision** - Platform choice from requirements analysis  
**Engineering Rationale**: 600MHz ARM Cortex-M7 provides sufficient headroom for 100Hz control loops with complex behavioral scaling algorithms, plus robust peripheral set for automotive integration  
**AI Traceability**: Drives all hardware interface implementations, timing constraints, memory architecture

- **Model**: Teensy 4.1 (NXP i.MX RT1062)
- **Clock Speed**: 600 MHz ARM Cortex-M7
- **Flash Memory**: 8MB
- **RAM**: 1024KB (FlexRAM: high-performance tightly coupled memory)

**🔗 T2-STORAGE-001**: **SD Card Primary Storage**  
**Derived From**: LearnedData.md capacity requirements (4.5KB learned data > 4KB EEPROM limit)  
**Decision Type**: ⚠️ **Engineering Decision** - Storage architecture choice  
**Engineering Rationale**: Learned data requirements exceeded EEPROM capacity, SD card provides unlimited expansion with wear management  
**AI Traceability**: Drives storage HAL design, wear management algorithms, backup strategies

- **Primary Storage**: MicroSD card (FAT32 filesystem) for all configuration and learned data
- **Write Debouncing**: All data writes debounced 5-10 seconds to optimize SD card wear characteristics

### Pressure Sensors (4 Total)
- **Type**: 0-30 PSI absolute pressure sensors
- **Output**: 5V (0.5V @ 0 PSI, 4.5V @ 30 PSI)
- **Interface**: Analog input with voltage divider
- **Voltage Divider**: 2kΩ + 1kΩ resistors (0.333 ratio)
- **Scaled Range**: 0.167V - 1.5V input to ADC
- **Resolution**: 0.007 PSI with 12-bit ADC
- **Sensor Locations**:
  1. **Manifold Pressure**: Intake manifold pressure measurement
  2. **Dome Input**: Air supply pressure to solenoid system  
  3. **Upper Dome**: Pressure in wastegate upper chamber
  4. **Lower Dome**: Pressure in wastegate lower chamber

### Solenoid Control

**🔗 T2-SOLENOID-001**: **4-Port MAC Solenoid Selection**  
**Derived From**: T1-SAFETY-001 (Overboost as Fault Condition) + Physics.md full-dome requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Hardware selection from pneumatic requirements  
**Engineering Rationale**: 4-port MAC valve enables full-dome control necessary for 0% duty = wastegate open failsafe behavior  
**AI Traceability**: Drives pneumatic control algorithms, pressure regulation logic, failsafe implementations

**🔗 T2-PWM-001**: **30 Hz PWM Frequency**  
**Derived From**: Hardware compatibility constraints (MAC valve 20-50Hz operating range)  
**Decision Type**: ⚠️ **Engineering Decision** - Frequency selection within acceptable range  
**Engineering Rationale**: 30Hz chosen for optimal response balance - fast enough for control loop requirements, slow enough to avoid valve resonance  
**Change Impact**: Tier 3 implementation detail - changes require hardware validation only  
**AI Traceability**: Use PWM_FREQUENCY_HZ constant with hardware compatibility validation

- **Type**: 4-port MAC solenoid (pneumatic boost control)
- **PWM Frequency**: 30 Hz
- **Control Range**: 0-100% duty cycle
- **Failsafe**: 0% duty cycle = minimal boost operation
- **Driver**: MOSFET-based load switch

### Display
- **Model**: ST7735R TFT LCD
- **Size**: 1.8" diagonal
- **Resolution**: 128×160 pixels
- **Interface**: SPI
- **Colors**: 65K color depth

### CAN Interface
- **Transceiver**: SN65HVD230 or equivalent
- **Bus Speed**: 500 kbps (Ford Gen2 Coyote standard)
- **Connection**: OBD-II port integration
- **Protocol**: ISO 11898-2 (High-speed CAN)

### Storage System
- **Primary Storage**: MicroSD card (Class 10 or better, 8-32GB recommended)
- **Filesystem**: FAT32 for maximum compatibility
- **File Structure**:
  ```
  /config/user_config.json         (User configuration - 5 parameters)
  /learned/calibration_maps.bin     (Duty cycle calibration tables)
  /learned/environmental.json       (Environmental compensation factors)  
  /learned/sensor_fusion.json       (Sensor cross-calibration data)
  /learned/safety_params.json       (Learned safety characteristics)
  /backups/[timestamp]/             (Automatic rolling backups)
  /logs/[date]/                     (Diagnostic and safety logs)
  ```
- **Write Strategy**: Debounced writes (5-10 seconds) with atomic file operations (crash-safe)
- **Wear Management**: Built-in SD card wear leveling + application-level write optimization
- **Expected Lifespan**: 10+ years with proper write management
- **Failure Recovery**: System continues with default parameters if SD card fails

### GPIO Pin Assignments (Teensy 4.1)
```
Pressure Sensors:
- Manifold Pressure:      Pin A0  (ADC)
- Dome Input Pressure:    Pin A1  (ADC) 
- Upper Dome Pressure:    Pin A2  (ADC)
- Lower Dome Pressure:    Pin A3  (ADC)

PWM Output:
- Solenoid Control:       Pin 2   (FlexPWM)

CAN Bus:
- CAN TX:                 Pin 22  (CAN1_TX)
- CAN RX:                 Pin 23  (CAN1_RX)

SPI Display:
- SPI Display CS:         Pin 10  (CS0)
- SPI Display DC:         Pin 9   (GPIO)
- SPI Display RST:        Pin 8   (GPIO)

User Controls:
- Control Knob Adjust:    Pin 4   (GPIO + Interrupt)
- Scramble Button:        Pin 5   (GPIO + Interrupt)
- Status LED:             Pin 13  (GPIO)
```

---

## Control System Specifications

### Timing and Frequencies
- **Main Control Loop**: 100 Hz execution frequency
- **Solenoid PWM**: 30 Hz output frequency  
- **PWM Synchronization**: Coordinated timing to prevent beat frequencies
- **CAN Polling**: As available (Ford ECU dependent)
- **Display Update**: 10 Hz refresh rate
- **ADC Sampling**: 4x averaging per reading

### Control Knob System
- **Range**: 0.0 - 1.0 (floating point)
- **Resolution**: 0.1% steps (0.001 increments)
- **Default**: 0.3 (30% - conservative daily driving)
- **Mapping**: Linear scaling of all torque amplification parameters
- **Scramble Mode**: Temporary boost to configurable higher level

### Safety Limits
- **Overboost Limit**: 15.0 PSI absolute maximum
- **Overboost Hysteresis**: 0.5 PSI deadband
- **Response Time**: <100ms to failsafe state
- **Torque Target**: 95% of ECU desired torque (configurable)
- **Maximum Boost Slew Rate**: 2.0 PSI/second default

### Learning System
- **Calibration Points**: 2D interpolation table (RPM × Boost Pressure)
- **Learning Rate**: Bounded, slew-rate limited adjustments
- **Confidence Tracking**: Multi-pass validation required
- **Environmental Compensation**: Temperature, altitude, supply pressure
- **Data Retention**: Persistent storage across power cycles

---

## Communication Protocols

### JSON/CLI Protocol
- **Transport**: Serial UART (115200 baud, 8N1) or Bluetooth SPP
- **Format**: JSON messages with `"ok": true/false` responses
- **Line Ending**: `\n` (newline)
- **Encoding**: UTF-8
- **Maximum Message Size**: 1KB
- **Timeout**: 5 seconds per request

### Bluetooth Interface  
- **Standard**: Bluetooth Classic 2.1+
- **Profile**: Serial Port Profile (SPP)
- **Range**: ~10 meters typical
- **Pairing**: Required for security
- **Same Protocol**: Identical JSON commands as serial interface

### CAN Bus Signals (Ford Gen2 Coyote)
⚠️ **SPECULATIVE - Requires Real Vehicle Verification**
- **RPM**: Signal ID TBD, 16-bit encoding expected
- **Manifold Pressure**: Signal ID TBD, kPa units → convert to PSI
- **Desired Torque**: Signal ID TBD, Nm units
- **Actual Torque**: Signal ID TBD, Nm units  

---

## Environmental Operating Conditions

### Temperature Range
- **Operating**: -40°C to +85°C (automotive grade)
- **Storage**: -40°C to +100°C
- **Thermal Management**: Passive cooling, no active thermal control

### Power Requirements
- **Input Voltage**: 12V automotive (9V-16V operating range)
- **Current Draw**: <500mA typical, <1A maximum
- **Standby Current**: <50mA (CAN monitoring active)
- **Voltage Regulation**: Internal buck converters (5V, 3.3V rails)

### Mechanical
- **Mounting**: Standard automotive electronics enclosure
- **Vibration**: Automotive specification compliance
- **IP Rating**: IP54 minimum (dust/splash protection)
- **Connections**: Automotive-grade connectors required

---

## Performance Specifications

### Response Characteristics
- **Control Loop Latency**: <10ms typical
- **Boost Response Time**: 200-500ms (turbo system dependent)
- **Overboost Detection**: <50ms detection, <100ms response
- **CAN Message Latency**: <20ms processing time
- **User Input Response**: <100ms acknowledgment

### Accuracy Specifications
- **Pressure Measurement**: ±0.1 PSI accuracy
- **PWM Output**: ±1% duty cycle accuracy
- **Control Knob Resolution**: 0.1% steps (0.001)
- **Timing Accuracy**: ±1% of specified frequencies
- **Temperature Compensation**: ±2% over operating range

---

## Safety and Compliance

### Fail-Safe Operation
- **Default State**: 0% PWM duty cycle (minimal boost)
- **Fault Response**: <100ms to safe state
- **Watchdog**: Hardware watchdog with 500ms timeout
- **CAN Timeout**: 500ms maximum before failsafe
- **Sensor Validation**: Cross-checking and plausibility limits

### Automotive Standards
- **EMC**: Automotive EMC compliance required
- **ESD Protection**: IEC 61000-4-2 Level 3 minimum
- **Transient Protection**: ISO 7637-2 pulse testing
- **CAN Compliance**: ISO 11898-2 standard

---

## Version Information

- **Specification Version**: 1.0
- **Last Updated**: January 2025
- **Applicable Firmware**: v1.0.0+
- **Hardware Revision**: Teensy 4.1 based design

**Note**: This specification is maintained as the authoritative reference. All implementation documents must reference these values rather than duplicating them.

---

*🔗 Referenced by: Context.md, Hardware.md, Architecture.md, Implementation.md, Safety.md, Protocols.md*