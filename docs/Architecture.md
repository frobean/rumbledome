# RumbleDome Architecture

## System Overview

RumbleDome is a torque-aware electronic boost controller that cooperates with modern ECU torque management systems rather than fighting them. The system prioritizes predictable, repeatable boost response to maintain ECU driver demand table validity while providing safety-critical overboost protection.

## High-Level Architecture

### Control Philosophy
**ECU Torque Production Assistant**: RumbleDome monitors ECU torque requests and delivery, then modulates boost to help the ECU achieve its torque targets smoothly and safely. The system works with the ECU's torque management (including all safety system overrides like traction control, ABS, clutch protection) rather than operating independently.

**PWM-Synchronized Control**: Advanced timing coordination prevents phase noise and jitter in pneumatic control through beat frequency elimination and cycle-synchronized updates. Control loop timing aligns with 100Hz PWM cycles for optimal solenoid response.

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
  - 0% duty → Lower dome pressurized → Wastegate forced OPEN
  - 100% duty → Upper dome pressurized → Wastegate forced CLOSED
- **Wastegate Spring**: 5 psi (configurable) provides mechanical failsafe and baseline control authority
- **Dome Control**: Full-dome system enables boost control both above and below spring pressure

### Sensor Configuration
1. **Dome Input Pressure**: Monitors air supply pressure for feedforward compensation
2. **Upper Dome Pressure**: Monitors wastegate actuation effectiveness  
3. **Manifold Pressure**: Primary safety monitor and boost measurement (post-throttle)

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
- **Beat Frequency Prevention**: Control updates coordinate with 100Hz PWM cycles
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

**Level 1: Torque-Based Boost Target Adjustment**
1. **Input Processing**: Read CAN torque signals (desired_torque, actual_torque), RPM
2. **Torque Gap Analysis**: Calculate `torque_error = desired_torque - actual_torque`
3. **Boost Target Modulation**: Adjust base boost target based on whether ECU is achieving its torque goals
   - Large torque gap → increase boost target to help ECU
   - Small/no gap → maintain current boost target  
   - Approaching torque ceiling → reduce boost target to prevent ECU intervention
4. **Profile Limit Enforcement**: Clamp boost target to user-configured profile maximums

**Level 2: Precise Boost Delivery (PID + Learned Calibration)**
5. **Learned Duty Baseline**: Look up base duty cycle from calibration data for current boost target
6. **Real-time PID Correction**: Apply PID control using `(target_boost - actual_boost)` from manifold pressure
7. **Environmental Compensation**: Apply learned compensation for temperature, altitude, supply pressure

**Level 3: Safety and Output**
8. **Safety Override**: Apply overboost protection and pneumatic system constraints
9. **Slew Rate Limiting**: Prevent rapid duty cycle changes that could cause unsafe response
10. **PWM Output**: Update solenoid duty cycle
11. **Learning Refinement**: Update calibration data and environmental compensation factors

#### Auto-Calibration System
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
- **Boost-Based Profiles**: Boost pressure limits (PSI/kPa) configured by user, not power targets
- **System Parameters**: Spring pressure, hardware configuration, safety limits
- **Profile Switching**: Live switching capability with safety validation

#### Profile Strategy - Boost vs Power Independence
- **Configure in boost pressure**: Profiles define boost pressure curves, never power targets
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

### Separation Strategy
- User configuration and learned data stored separately
- Learning reset preserves user preferences
- Configuration changes don't affect learned calibration data
- Independent backup and restore capabilities

## Performance Architecture

### Real-time Constraints
- **Control Loop**: 100 Hz minimum execution frequency
- **Safety Response**: <100ms from overboost detection to wastegate opening
- **CAN Processing**: Minimal latency message handling
- **Display Updates**: Smooth gauge animation and status updates

### Memory Management
- **Static Allocation**: Predictable memory usage for embedded reliability
- **Wear-Aware Storage**: EEPROM wear leveling for configuration and learned data
- **Bounded Data Structures**: Fixed-size learning tables and calibration storage

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
- **Phase 2 ("Beyond RumbleDome")**: Separation of Power Level from Delivery Style
  - **Power Level (Profile)**: What boost/power you get (user-selected: Valet/Daily/Aggressive/Track)
  - **Delivery Style (Drive Mode)**: How that power is delivered (Normal/Sport+/Track aggressiveness)
  - **Safety Benefit**: Drive mode changes don't automatically increase power - prevents accidental power jumps

#### Phase 2 Design Philosophy
- **Explicit Power Selection**: User must intentionally select power level (profile)
- **Drive Mode Independence**: Selecting Sport+ or Track mode changes delivery characteristics, not boost limits
- **Safety Through Separation**: No accidental power increases from drive mode selections
- **Predictable Power Levels**: Boost targets remain consistent regardless of how aggressively they're delivered