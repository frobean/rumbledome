# RumbleDome Architecture

📋 **Technical specifications**: See **[TechnicalSpecs.md](TechnicalSpecs.md)** for hardware details

📖 **For terminology**: See **[Definitions.md](Definitions.md)** for technical concepts and acronyms used throughout this document

## System Overview

RumbleDome is a torque-aware electronic boost controller that cooperates with modern ECU torque management systems rather than fighting them. The system prioritizes predictable, configurable boost response to maintain ECU driver demand table validity while providing safety-critical overboost protection.

## High-Level Architecture

### Control Philosophy: 3-Tier Priority Hierarchy

**🔗 T2-CONTROL-001**: **Priority Hierarchy Implementation**  
**Derived From**: T1-PHILOSOPHY-001 (3-Tier Priority Hierarchy)  
**Decision Type**: 🔗 **Direct Derivation** - Implementation of foundational control philosophy  
**AI Traceability**: Drives all control algorithms, safety systems, aggression scaling behavior

RumbleDome implements a **priority hierarchy with aggression-mediated balance** that organizes all system architecture:

**Priority 1: "Don't Kill My Car"** 🚨 - Overboost protection with maximum authority (always overrides)

**Priority 2 & 3: Performance ⚖️ Comfort Balance** - Aggression knob determines which leads:
- **High Aggression**: Priority 2 leads (forceful max boost targeting) 🎯
- **Low Aggression**: Priority 3 leads (smooth comfortable operation) ✨  
- **Medium Aggression**: Balanced approach between performance and comfort  

**ECU Cooperation Strategy**: RumbleDome monitors ECU torque requests and delivery, then modulates boost to help the ECU achieve its torque targets smoothly and safely. The system works with the ECU's torque management (including all safety system overrides) rather than operating independently.

**🔗 T2-CONTROL-002**: **PWM-Synchronized Control Architecture**  
**Derived From**: T2-PWM-001 (30 Hz PWM Frequency) + Performance timing requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Advanced timing coordination strategy  
**Engineering Rationale**: 100Hz/30Hz coordination prevents beat frequency interference, eliminates phase noise  
**AI Traceability**: Drives timing validation, cycle synchronization, jitter reduction algorithms

**PWM-Synchronized Control**: Advanced timing coordination prevents phase noise and jitter in pneumatic control through beat frequency elimination and cycle-synchronized updates. 100Hz control loop timing coordinates with 30Hz PWM cycles for optimal solenoid response.

**Automatic Safety System Integration**: By responding to the final desired torque (after all ECU safety systems have applied their modifications), RumbleDome automatically cooperates with traction control, ABS, stability control, and other safety systems without requiring specific knowledge of each system.

### System Components

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   ECU (CAN)     │────│  RumbleDome      │────│  Pneumatic      │
│                 │    │  Controller      │    │  System         │
│ • Torque Demand │    │                  │    │                 │
│ • Actual Torque │    │ • Control Logic  │    │ • 4-port MAC    │
│ • RPM, MAP      │    │ • Safety Monitor │    │   Solenoid      │
│                 │    │ • Learning Sys   │    │ • Dome Control  │
└─────────────────┘    │ • Calibration    │    │ • Wastegates    │
                       └──────────────────┘    └─────────────────┘
                                │
                       ┌──────────────────┐
                       │   User Interface │
                       │                  │
                       │ • TFT Display    │
                       │ • JSON Protocol  │
                       │ • Calibration UI │
                       └──────────────────┘
```

## Physical System Architecture

### Pneumatic Control System
- **Air Supply**: Compressed air regulated from ~150 psi to configurable input pressure (typically 10-20 psi)
- **4-Port MAC Solenoid**: Controls dome pressure distribution
  - 0% duty → Lower dome pressurized, upper dome vented → Wastegate forced OPEN
  - 100% duty → Upper dome pressurized, lower dome vented → Wastegate forced CLOSED
- **Wastegate Spring**: 5 psi (configurable) provides mechanical failsafe and baseline control authority
- **Dome Control**: Full-dome system with constant pressure feed enables boost control both above and below spring pressure

### Sensor Configuration
1. **Dome Input Pressure**: Monitors air supply pressure for feedforward compensation
2. **Upper Dome Pressure**: Monitors wastegate actuation effectiveness  
3. **Lower Dome Pressure**: Monitors wastegate opening force and pneumatic system health
4. **Manifold Pressure**: Primary safety monitor and boost measurement (post-throttle)

### Control Authority Analysis
```
Max Theoretical Boost ≈ Spring Pressure + Input Air Pressure
Control Resolution ∝ 1 / Input Air Pressure
Safety Response Time ∝ Input Air Pressure × Dome Volume / Solenoid Flow Rate
```

## Software Architecture

### Layered Design

**Layer 1: Hardware Abstraction (HAL)**
- Platform-independent traits for all hardware interfaces
- Teensy 4.1 implementation for production hardware
- Mock implementations for desktop testing
- Future platform support through additional HAL implementations

**Layer 2: Core Control Logic**
- Zero hardware dependencies - pure business logic
- All safety-critical algorithms and decision making
- Learning and adaptation algorithms
- State management and fault handling

**Layer 3: Protocol & Interface**
- JSON/CLI protocol definitions and parsing
- User interface abstractions
- Configuration management
- Diagnostic and telemetry interfaces

**Layer 4: Application Integration**
- Firmware binary for target hardware
- Desktop simulator for testing and validation
- Configuration tools and utilities

## PWM Synchronization Architecture

### Timing Coordination System
RumbleDome implements advanced PWM-synchronized control timing to eliminate phase noise and jitter in pneumatic control:

```rust
pub enum ControlUpdateStrategy {
    Asynchronous,           // Standard async timing
    CycleStart,            // Update at PWM cycle start
    CycleMidpoint,         // Update at cycle midpoint (optimal)
    SubCycle { updates_per_cycle: u8 }, // Multiple updates per cycle
}
```

**Key Benefits**:
- **Beat Frequency Prevention**: 100Hz control updates coordinate with 30Hz PWM cycles
- **Jitter Reduction**: Deadband filtering using 0.003% FlexPWM resolution  
- **Phase Noise Elimination**: Synchronized updates prevent control/PWM interference
- **Timing Windows**: ±10% update windows around optimal cycle points

### PWM Timing Integration
```rust
// Control loop validates PWM timing before updates
fn execute_control_cycle(&mut self, pwm_timing: &PwmTimingInfo) -> Result<(), CoreError> {
    // Check for optimal update window
    if !pwm_timing.is_optimal_update_time(current_time_us) {
        let wait_time = pwm_timing.time_to_next_update_window_us(current_time_us);
        if wait_time < MAX_ACCEPTABLE_DELAY_US {
            // Wait for optimal timing window
            delay_us(wait_time);
        }
    }
    
    // Execute synchronized control update
    self.apply_duty_cycle_synchronized(duty_cycle, current_time_us)?;
}
```

### Control System Architecture

#### Primary Control Loop (100 Hz) - 3-Level Hierarchy

**🔗 T2-CONTROL-003**: **3-Level Control Hierarchy Implementation**  
**Derived From**: T1-PHILOSOPHY-001 (3-Tier Priority) + T2-ECU-001 (Torque Production Assistant)  
**Decision Type**: 🔗 **Direct Derivation** - Implementation of torque-based control strategy  
**AI Traceability**: Drives torque analysis, boost targeting, safety override algorithms

**🔗 T2-CONTROL-004**: **Torque-Based Boost Target Adjustment**  
**Derived From**: T2-ECU-001 (Torque Production Assistant) + FR-1 (ECU Integration)  
**Decision Type**: 🔗 **Direct Derivation** - Core torque-following implementation  
**AI Traceability**: Drives CAN signal processing, torque gap analysis, ECU cooperation logic

**Level 1: Torque-Based Boost Target Adjustment**
1. **Input Processing**: Read CAN torque signals (desired_torque, actual_torque), RPM
2. **Torque Gap Analysis**: Calculate `torque_error = desired_torque - actual_torque`
3. **Torque Assistance Decision**: Determine boost assistance based on ECU torque achievement and aggression setting
   - Large torque gap + high aggression → provide strong boost assistance to help ECU
   - Small/no gap → maintain current assistance level
   - Approaching torque ceiling → reduce assistance to prevent ECU intervention  
4. **Safety Limit Enforcement**: Clamp boost assistance to configured safety ceilings (max_boost_psi, overboost_limit)

**🔗 T2-CONTROL-005**: **Precise Boost Delivery (PID + Learned Calibration)**  
**Derived From**: FR-3 (Auto-Calibration System) + FR-6 (Learning & Adaptation)  
**Decision Type**: ⚠️ **Engineering Decision** - Hybrid control approach combining PID with learned baselines  
**Engineering Rationale**: PID alone insufficient for turbo dynamics, learned baseline provides optimal starting point  
**AI Traceability**: Drives calibration lookup, PID controller, environmental compensation

**Level 2: Precise Boost Delivery (PID + Learned Calibration)**
5. **Learned Duty Baseline**: Look up base duty cycle from calibration data for current boost target
6. **Real-time PID Correction**: Apply PID control using `(target_boost - actual_boost)` from manifold pressure
7. **Environmental Compensation**: Apply learned compensation for temperature, altitude, supply pressure

**🔗 T2-CONTROL-006**: **Safety and Output Control Layer**  
**Derived From**: T1-PHILOSOPHY-001 (Priority 1: Don't Kill My Car) + FR-5 (Safety & Fault Management)  
**Decision Type**: 🔗 **Direct Derivation** - Safety-first output control implementation  
**AI Traceability**: Drives overboost protection, slew rate limiting, PWM output validation

**Level 3: Safety and Output**
8. **Safety Override**: Apply overboost protection and pneumatic system constraints
9. **Slew Rate Limiting**: Prevent rapid duty cycle changes that could cause unsafe response
10. **PWM Output**: Update solenoid duty cycle
11. **Learning Refinement**: Update calibration data and environmental compensation factors

#### Auto-Calibration System

**🔗 T2-CONTROL-007**: **Progressive Safety Auto-Calibration**  
**Derived From**: FR-3 (Auto-Calibration System) + T1-SAFETY-001 (Overboost as Fault)  
**Decision Type**: ⚠️ **Engineering Decision** - 3-phase progressive calibration strategy  
**Engineering Rationale**: Gradual expansion from ultra-conservative prevents unsafe calibration runs  
**AI Traceability**: Drives calibration progression logic, safety validation, confidence tracking

**Progressive Safety Approach**:
- Phase 1: Ultra-conservative limits (spring + 1 psi)
- Phase 2: Gradual expansion based on proven safety response
- Phase 3: Target achievement with full safety validation

**Learning Process**:
```
For each (RPM, Boost_Target) pair:
  1. Start with conservative duty cycle
  2. Gradually increase duty in small steps
  3. Monitor boost response and safety metrics
  4. Record successful duty cycle when target achieved
  5. Validate with multiple runs for consistency
  6. Apply environmental compensation factors
```

#### Safety System Architecture

**🔗 T2-CONTROL-008**: **Defense in Depth Safety Architecture**  
**Derived From**: T1-PHILOSOPHY-001 (Priority 1: Don't Kill My Car) + T1-SAFETY-002 (Defense in Depth)  
**Decision Type**: 🔗 **Direct Derivation** - Multi-layer safety implementation  
**AI Traceability**: Drives electronic safety, pneumatic failsafe, mechanical backup systems

**Defense in Depth**:
1. **Electronic Safety**: Software overboost detection and response
2. **Pneumatic Safety**: 0% duty forces wastegate open via dome pressure
3. **Mechanical Safety**: Spring provides baseline pressure relief

**Fault Response Hierarchy**:
- **Critical Faults**: Immediate duty=0%, system halt, display fault
- **Sensor Faults**: Invalid readings, CAN timeouts, storage errors
- **Calibration Faults**: Inconsistent learning data, safety response failures

## Data Architecture

### Configuration Data
- **Aggression-Based**: Single aggression setting (0.0-1.0) scales torque amplification parameters, not predefined boost curves
- **System Parameters**: Spring pressure, hardware configuration, safety limits
- **Real-time Aggression Scaling**: Aggression setting dynamically adjusts torque-following response characteristics

#### Control Strategy - Boost vs Power Independence  
- **Configure via torque amplification**: Aggression setting scales ECU torque request assistance, never absolute power targets (power target is owned by the ECU)
- **Engine-agnostic approach**: Same boost pressure produces different power depending on:
  - Engine tune (timing, fuel, cam timing)
  - Turbo sizing and efficiency  
  - Intercooling, exhaust, internal modifications
  - Environmental conditions (altitude, temperature, fuel quality)
- **User responsibility**: Determine appropriate boost limits for their specific engine setup
- **Universal compatibility**: Works with any engine/tune combination within boost pressure constraints

### Learned Data  
- **Duty Cycle Mappings**: Boost target → required duty cycle relationships
- **Environmental Compensation**: Temperature, altitude, supply pressure factors
- **Response Characteristics**: Boost rise rates, system response timing
- **Calibration Confidence**: Data quality metrics and validation status

📋 **Complete learned data specification**: See **[LearnedData.md](LearnedData.md)** for comprehensive details on all learning algorithms and storage structures

### Separation Strategy
- User configuration and learned data stored in separate SD card files
- Learning reset preserves user preferences (only affects /learned/ directory)
- Configuration changes don't affect learned calibration data
- Independent backup and restore capabilities via file system operations
- Human-readable config files enable direct editing and version control

## Performance Architecture

### Real-time Constraints
- **Control Loop**: 100 Hz minimum execution frequency
- **Safety Response**: <100ms from overboost detection to wastegate opening
- **CAN Processing**: Minimal latency message handling
- **Display Updates**: Smooth gauge animation and status updates

### Storage Management
- **Static Allocation**: Predictable memory usage for embedded reliability
- **SD Card Storage**: FAT32 filesystem for all configuration and learned data
- **Debounced Persistence**: All data writes debounced 5-10 seconds to optimize SD card wear
- **Atomic Operations**: Crash-safe writes using temp files and atomic renames
- **Automatic Backups**: Rolling backups preserve data integrity across system updates
- **Graceful Degradation**: System continues with defaults if SD card fails

## Extensibility Architecture

### Hardware Platform Support
- **HAL Abstraction**: Clean interfaces for different MCU platforms
- **Sensor Flexibility**: Support for different pressure sensor types and ranges
- **Display Options**: Abstracted display interface for different screen types
- **CAN Protocol Support**: Vehicle-specific protocol implementations

### Vehicle Platform Support
- **Ford Gen2 Coyote**: Initial target platform with known CAN signals
- **Future Platforms**: GM, Mopar, others through CAN protocol HAL implementations
- **Signal Mapping**: Configurable CAN ID and scaling parameters
- **Torque Model Variations**: Platform-specific torque interpretation logic

### Feature Evolution
- **Phase 1 (RumbleDome MVP)**: Core torque-based control, auto-calibration, manual profile selection
- **Phase 2 ("Beyond RumbleDome")**: TBD
