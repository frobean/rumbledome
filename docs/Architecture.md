# RumbleDome Architecture

📋 **Technical specifications**: See **[TechnicalSpecs.md](TechnicalSpecs.md)** for hardware details

📖 **For terminology**: See **[Definitions.md](Definitions.md)** for technical concepts and acronyms used throughout this document

**🔗 T1-PHILOSOPHY-005**: **Comprehensive Diagnostics and Observability**  
**Decision Type**: 🎯 **Core Philosophy** - Expose system internals for troubleshooting despite simplified user interface  
**Engineering Rationale**: Complex auto-learning systems require detailed observability for field debugging and development  
**AI Traceability**: Drives logging architecture, diagnostic interfaces, fault reporting, and development tooling

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
- **Air Supply**: Compressed air regulated from ~150 psi to calculated nominal feed pressure
- **Air-Efficient Control Strategy**: Bias toward 100% duty cycle (wastegate closed) for steady-state operation
  - **Steady-state closed operation** (vacuum conditions) = 100% duty = no air consumption
  - **Progressive easing function** for smooth transitions from closed to active boost control
  - **Compressor-friendly operation** - minimal air usage during majority of drive time
- **4-Port MAC Solenoid**: Controls dome pressure distribution
  - 0% duty → Lower dome pressurized, upper dome vented → Wastegate forced OPEN
  - 100% duty → Upper dome pressurized, lower dome vented → Wastegate forced CLOSED
- **Wastegate Spring**: Configurable spring pressure (minimum 5 psi to prevent flutter) provides mechanical failsafe and baseline control authority
- **Dome Control**: Full-dome system with constant pressure feed enables boost control both above and below spring pressure

**Spring Optimization Strategy:**
- **Standard Configuration**: 5+ PSI spring ensures smooth failure mode (predictable boost if pneumatics fail)
- **Minimum Spring Optimization**: 2-3 PSI spring for maximum control resolution, with active flutter prevention
  - **Flutter Prevention Logic**: Maintain `Upper Dome Pressure + Spring Pressure >= 5 PSI` when wastegate commanded closed
  - **Sensor-Based Control**: Use dome pressure sensors to actively prevent flutter through pressure management
  - **Failure Mode Trade-off**: Pneumatic failure results in low boost + potential flutter until repaired
  - **Use Case**: Optimal for track-only or closely monitored applications prioritizing control precision

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
- Phase 1: Ultra-conservative limits (spring pressure + 1 psi)
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

#### Auto-Learning Pneumatic System

**🔗 T2-CONTROL-009**: **Adaptive Pneumatic System with Feed Pressure Compensation**  
**Derived From**: T1-PHILOSOPHY-001 (Single-Knob Philosophy) + Physics.md pneumatic system dynamics  
**Decision Type**: ⚠️ **Engineering Decision** - Auto-learning system for pneumatic optimization  
**Engineering Rationale**: Feed pressure variations from low-cost regulators require real-time compensation for stable control  
**AI Traceability**: Drives T4-CORE-010+ auto-learning implementations, feed pressure monitoring, and adaptive control algorithms

**Pneumatic System Architecture:**

**Feed Pressure Management:**
- **Calculated nominal feed pressure** - optimizes control resolution vs. safety authority trade-off per build
- **Feed pressure calculation**: `Nominal Feed Pressure = Spring Pressure + Safety Margin + (Target Boost × Scaling Factor)`
  - **Safety Margin**: 3 PSI minimum (universal constraint for reliable opening authority)
  - **Scaling Factor**: 0.6 (empirical factor for boost scaling, adjustable per application)
- **Real-time feed pressure monitoring** - compensates for regulator imprecision and drift
- **Pressure-normalized learning** - all learned parameters adjusted for actual vs. nominal feed pressure
- **Dynamic compensation factor**: `compensated_duty_cycle = base_duty_cycle × (nominal_feed_pressure / actual_feed_pressure)`

**Multi-Layer Auto-Learning Protocol:**

**Layer 0: System Bootstrap and Validation (Key-On Procedure)**
- **Feed Pressure Wait** - monitor feed pressure until minimum threshold reached (Spring Pressure + Safety Margin)
- **Pressure Stabilization** - wait for feed pressure stable within ±0.5 PSI for 2-3 seconds
- **Session Baseline Capture** - record stabilized pressure as compensation baseline for current session
- **Feed Pressure Optimization Check** - validate pressure against calculated optimal range
  - **Max Useful Pressure**: `Overboost Limit + Spring Pressure + Fudge Factor`
  - **Scaled Fudge Factor**: `Base Margin (2-3 PSI) + (Overboost Limit × 0.15)`
  - **Control Range Validation**: Ensure minimum 20% usable duty cycle range for stable operation
  - **Warning Thresholds**: Yellow warning if above optimal, red warning if control range <20%
- **Dome Connectivity Test** - cycle solenoid 0%→100%→0% and verify both domes respond with expected pressure changes
- **Cross-Connection Detection** - test for reversed upper/lower dome lines through pressure response polarity
- **Safety Authority Verification** - confirm 0% duty cycle can achieve wastegate opening force (lower dome > spring + exhaust pressure)
- **System Health Gate** - require all Layer 0 tests to pass before enabling boost control or higher learning layers
- **Installation Diagnostics** - provide specific error codes and guidance for plumbing corrections if tests fail
- **User Feedback** - display "Initializing Pneumatics..." during wait, "RumbleDome Ready" when complete

**Layer 1: Pneumatic System Characterization**
- **Feed pressure baseline measurement** and deviation tracking from calculated nominal pressure
- **Dome response time constants** - measure upper/lower dome pressurization and evacuation rates
- **System bandwidth detection** - determine maximum useful control frequency for stable operation
- **Regulator health monitoring** - track pressure stability and recovery characteristics

**Layer 2: Boost Control Mapping**  
- **Critical range mapping** (0% to calculated safety threshold duty cycle) with 5% increment resolution testing
- **Pressure-normalized control tables** - store all learning data relative to calculated nominal feed pressure baseline
- **Safety threshold detection** - identify duty cycle limits for overboost protection
- **Transient response characterization** - measure boost response times and settling behavior

**Layer 3: Runtime Adaptive Optimization**
- **Continuous feed pressure compensation** - real-time adjustment for regulator variations
- **Performance degradation detection** - monitor response times for maintenance indication
- **Environmental adaptation** - compensate for temperature effects on pneumatic components
- **Torque-following integration** - apply learned parameters to ECU torque gap responses

**Auto-Learning Safety Integration:**

**Startup System Validation:**
- **Feed pressure adequacy check** - verify minimum (Spring Pressure + Safety Margin) for adequate opening authority
- **Dome response validation** - confirm pneumatic system can achieve required response rates
- **Safety authority confirmation** - validate overboost protection capability before operation
- **Fallback to defaults** - use conservative pre-programmed parameters if learning fails

**Runtime Safety Monitoring:**
- **Feed pressure fault detection** - alert when pressure drops below safe operating threshold
- **Compensation limit enforcement** - prevent compensation from exceeding safe duty cycle ranges  
- **Learning data validation** - verify new learning data is consistent with safety requirements
- **Emergency bypass** - disable auto-learning and use defaults during fault conditions

**System Health Diagnostics:**
- **Regulator performance trending** - track pressure stability degradation over time
- **Pneumatic component health** - monitor dome response time changes indicating wear/damage
- **Learning convergence analysis** - ensure auto-learning produces stable, repeatable results
- **Predictive maintenance alerts** - warn when system performance indicates service needs

**Integration with Torque-Following Control:**
- **Pressure-compensated boost targeting** - apply learned compensation to torque gap responses  
- **Adaptive rate limiting** - use learned system bandwidth to prevent control instability
- **Performance optimization** - continuously refine torque response based on pneumatic system capability
- **Aggression scaling integration** - apply learned parameters across full aggression range (0.0-1.0)

This auto-learning pneumatic system transforms RumbleDome from a static boost controller into an **adaptive, self-optimizing system** that automatically compensates for hardware variations, component aging, and environmental changes while maintaining safety authority and control precision.

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

## Diagnostics and Logging Architecture

**🔗 T2-DIAGNOSTICS-001**: **Multi-Output Logging System**  
**Derived From**: T1-PHILOSOPHY-005 (Comprehensive Diagnostics)  
**Decision Type**: ⚠️ **Engineering Decision** - Structured logging with multiple output channels  
**Engineering Rationale**: Complex auto-learning system requires detailed observability without impacting real-time performance  
**AI Traceability**: Drives debugging interfaces, field troubleshooting tools, and development workflows

**Logging Output Channels:**
- **Console Output** - real-time debugging during development and bench testing
- **Serial/Bluetooth** - wireless diagnostic interface for field troubleshooting  
- **SD Card Filesystem** - persistent logging for trend analysis and post-incident review
- **Display Integration** - critical alerts and status on main user interface

**Log Level Hierarchy:**
- **CRITICAL** - system faults requiring immediate user attention (overboost, sensor failures)
- **WARNING** - degraded performance or maintenance indicators (pressure drift, learning convergence issues)
- **INFO** - operational state changes (boost events, learning progress, configuration changes)
- **DEBUG** - detailed system internals (sensor readings, control loop calculations, CAN messages)
- **TRACE** - high-frequency data streams (pneumatic response timing, duty cycle adjustments)

**Structured Logging Categories:**
- **PNEUMATIC** - feed pressure, dome pressures, solenoid response, leak detection, regulator health
- **CONTROL** - PID calculations, duty cycle commands, boost targeting, torque gap analysis
- **LEARNING** - auto-learning progress, calibration data updates, convergence metrics
- **SAFETY** - overboost events, fault conditions, emergency responses, safety authority verification
- **CAN** - ECU communication, torque signals, message timing, protocol errors
- **SYSTEM** - startup/shutdown, configuration changes, SD card operations, memory usage

**Log Storage Strategy:**
- **Ring Buffer** - recent logs in RAM for immediate access via diagnostic interface (all log levels)
- **Rolling Files** - daily log files with automatic cleanup (keep last 30 days)
- **SD Card Write Protection** - default to logging only CRITICAL and WARNING events to minimize SD wear
- **Runtime Log Level Control** - users can increase logging verbosity via console interface for debugging
- **Structured Format** - JSON or CSV for machine parsing and analysis tools
- **Compression** - older logs compressed to minimize SD card usage
- **Export Capability** - logs can be extracted for external analysis or support requests

**Diagnostic Interface:**
- **Live Data Stream** - real-time sensor values and system state via Bluetooth
- **Historical Analysis** - trend plotting and pattern recognition from stored logs  
- **Fault Code System** - standardized error codes with troubleshooting guidance
- **Learning Visibility** - view auto-learning progress and calibration confidence levels
- **Performance Metrics** - system health scores and maintenance predictions

**🔗 T2-DIAGNOSTICS-002**: **Pneumatic Fault Detection Algorithms**  
**Derived From**: T2-DIAGNOSTICS-001 (Multi-Output Logging) + pneumatic system reliability requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Real-time pneumatic system health monitoring  
**Engineering Rationale**: Early fault detection prevents system damage and enables predictive maintenance  
**AI Traceability**: Drives fault response protocols, maintenance scheduling, and system reliability metrics

**Blown Hose Detection:**
- **Pressurization Test** - command dome pressurization and verify pressure response reaches >50% of feed pressure within 2 seconds
- **Upper Dome Test** - 100% duty cycle should pressurize upper dome to near feed pressure level
- **Lower Dome Test** - 0% duty cycle should pressurize lower dome to near feed pressure level  
- **Blown Hose Indication** - commanded dome fails to reach expected pressure (remains near atmospheric + small line segment pressure)
- **Bootstrap Integration** - active testing during Layer 0 system validation
- **Runtime Monitoring** - verify expected pressure response any time significant dome pressurization is commanded

**Feed Pressure Monitoring:**
- **Session Drift Detection** - feed pressure varies >1.5 PSI from established baseline during single drive
- **Historical Trending** - session baselines drift >0.5 PSI/week indicating regulator degradation
- **Stability Threshold** - feed pressure variance >±0.3 PSI over 30 second window during steady state
- **Adequacy Thresholds** - WARNING when <(Spring Pressure + 2 PSI), CRITICAL when <Spring Pressure

**Solenoid Valve Health:**
- **Response Timeout** - dome pressure change <1 PSI within 2 seconds of 50% duty cycle command
- **Sticking Detection** - duty cycle changes >15% produce <0.2 PSI dome response (when commands should produce response)
- **Recovery Protocol** - attempt 0%→100%→0% cycle up to 3 times before declaring fault
- **Command Validation** - only monitor response when duty cycle change should produce measurable pressure change

**System Response Degradation:**
- **Slow Response Detection** - dome pressure change takes >3 seconds to reach 90% of expected change
- **Trend Tracking** - response times increase >50% compared to Layer 1 learning baseline
- **Maintenance Alerts** - WARNING when response times exceed 2x baseline, schedule predictive maintenance

**Fault Response Protocols:**
- **CRITICAL Faults** (blown hose, inadequate feed pressure) - immediate duty=0%, disable boost control, display fault code
- **WARNING Faults** (pressure drift, slow response) - continue operation with compensation, log for maintenance scheduling
- **Recovery Attempts** - automated recovery cycles for intermittent faults before escalating to manual intervention

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
