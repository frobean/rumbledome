# RumbleDome Requirements

📖 **Related Documentation:**
- [Context.md](Context.md) - Design goals and philosophy behind these requirements
- [Physics.md](Physics.md) - Physical principles that constrain these specifications  
- [Architecture.md](Architecture.md) - System design implementing these requirements
- [Safety.md](Safety.md) - Safety requirements that take precedence over functional requirements
- [Definitions.md](Definitions.md) - Terminology and technical concepts used in these requirements

## Control Philosophy Foundation

**🏗️ TIER 1 SECTION: Foundational Philosophy**

**Foundational Philosophy**: T1-PHILOSOPHY-001 (3-Tier Priority Hierarchy) - See [Context.md](Context.md) for complete specification  
**AI Traceability**: This philosophy drives ALL functional requirements below

RumbleDome implements a **priority hierarchy with aggression-mediated balance** that governs all system behavior:

### **Priority 1: "Don't Kill My Car"** 🚨
**Overboost is a fault condition** requiring immediate hard correction and learning integration for future prevention. The system will use maximum authority to prevent manifold pressure from exceeding the user-configured `overboost_limit`. **Always takes precedence.**

### **Priority 2 & 3: Performance ⚖️ Comfort Balance**
The `aggression` setting determines which sibling priority leads:

**Priority 2: Optimize Max Boost Targeting** 🎯  
- **High Aggression (0.8-1.0)**: Performance leads - forceful targeting of `max_boost_psi`, sharp responses
- **Best effort to hit user's target** - Brief pressure spikes acceptable during transients
- **Sustained elevation triggers learning adjustments** for improved control

**Priority 3: Smooth Aggression-Informed Operation** ✨
- **Low Aggression (0.0-0.3)**: Comfort leads - gentle smooth delivery, gradual responses  
- **Medium Aggression (0.4-0.7)**: Balanced approach between performance and comfort
- **Control character shaped by learned operational patterns** and user preference

---

## 🏗️ TIER 2 SECTION: Derived Functional Requirements

**All specifications below are 🔗 Direct Derivations from Tier 1 concepts above**

## Functional Requirements

### FR-1: ECU Integration & Cooperation

**🔗 T2-ECU-001**: **Torque Production Assistant**  
**Derived From**: T1-TORQUE-001 (ECU Cooperation Philosophy)  
**Decision Type**: 🔗 **Direct Derivation** - Logical implementation of cooperation concept  
**AI Traceability**: Drives CAN interface requirements, control algorithms

- **Torque Production Assistant**: System monitors ECU torque requests and delivery, modulating boost to help ECU achieve torque targets
- **Final Torque Respect**: Respond to ECU's final desired_torque (after all safety systems have applied modifications) to avoid conflicts
- **Automatic Safety Integration**: Automatically cooperate with traction control, ABS, clutch protection, and other ECU safety systems without specific programming
- **Torque Target Strategy**: Target actual_torque at configurable percentage below desired_torque ceiling (default ~95%) to prevent harsh ECU interventions
- **Predictable Response**: Provide consistent, repeatable boost response so ECU's driver demand tables remain valid across all operating conditions

### FR-2: Single-Knob Control System

**🔗 T2-CONFIG-001**: **Pressure-Based Configuration**  
**Derived From**: T1-UI-001 (Single Parameter Philosophy)  
**Decision Type**: 🔗 **Direct Derivation** - Simplification principle applied to configuration  
**AI Traceability**: Drives user interface specifications, configuration storage

- **Boost Pressure Configuration**: All user configuration in pressure units (PSI/kPa), never raw duty cycles or power targets
- **Single Knob Control**: One control knob (0.0-1.0) replaces all profile complexity:
  - **0.0% (OFF Requirement)**: System OFF - as close to naturally aspirated operation as physically possible
    - 0% duty cycle (wastegate fully open)
    - Zero boost assistance to ECU torque requests
    - Infinite torque error deadband (ignore all ECU torque demands)
    - **Full Dome Control Requirement**: The "OFF" behavior only applies to full dome control systems where 0% duty sends full pressure to lower dome. Half dome systems maintain spring pressure as lower bound.
    - **Physical Limitation Acknowledgment**: Even with wastegate fully open, high exhaust energy may spin the turbos hard enough to force some boost into the intake manifold under certain conditions. This is unavoidable in absence of the ability to open the blowoff valves. Which is, as said, absent.
  - **1-99%**: Variable aggression scaling of torque-following assistance
  - **100%**: Maximum system aggression (instant ECU torque request assistance)
- **User Configuration Responsibility**: Users have full responsibility for setting appropriate limits for their engine/turbo configuration
- **System Guidance Role**: Provides intelligent warnings and learned suggestions, never enforcement of limits beyond overboost protection
- **Five Simple Configuration Values**:
  - `aggression` (0.0-1.0) - Single aggression control integrated with system operation
  - `spring_pressure` (PSI) - wastegate spring pressure
  - `max_boost_psi` (PSI) - safety ceiling for boost pressure (not knob-scaled target)
  - `overboost_limit` (PSI) - hard safety fault threshold (never exceed)
  - `scramble_enabled` (bool) - temporary higher performance mode toggle
- **Scramble Override**: Instant 100% aggression override via momentary button (non-latching)
- **Live Adjustment**: Safe aggression adjustment during operation via rotary encoder
- **Aggression Persistence**: Debounced NVRAM storage - aggression changes persist only after 5-10 seconds of stability to prevent wear during adjustment sessions

### Safety Margin Configuration Philosophy

**🔗 T2-CONFIG-006**: **User-Configurable Safety Margins with Dual Learning Architecture**  
**Derived From**: T1-PHILOSOPHY-001 (Priority 1: Don't Kill My Car) + realistic embedded system constraints  
**Decision Type**: ⚠️ **Engineering Decision** - User responsibility with simple system support  
**Engineering Rationale**: Users set risk tolerance; system learns safe operation within bounds using simple, reliable algorithms  
**AI Traceability**: Drives approach curve learning, conservative exploration, simple overboost response

**Core Principle**: System learns to predictably undershoot overboost limits through safe exploration, with simple overboost response.

**User Responsibility**: Set boost limits (`max_boost_psi`, `overboost_limit`) based on engine capability and risk tolerance  
**System Responsibility**: Learn boost approach curves that reliably stay within user-configured bounds

**Dual Learning Architecture**:

**Primary Learning Path: Conservative Exploration**
- Learn safe approach curves by observing successful approaches within `max_boost_psi`
- Start extremely conservative, gradually optimize through safe undershoot analysis
- Rate-of-rise limiting based on distance from boost ceiling
- Example: "Approached 18 PSI target, peaked at 17.8 PSI → can be slightly more aggressive"

**Emergency Learning Path: Simple Overboost Response**  
- Learn from overboost events with immediate conservative adjustment
- Immediate safety response (duty cycle = 0%) + event logging
- Make approach curves more conservative across all conditions
- Simple rule: "That approach rate caused overboost → use slower rates everywhere"
- No complex pattern analysis or multi-variable correlation

**Relearning Triggers**: 
- User changes boost limits → reset to conservative baseline
- Overboost event → immediate global conservative adjustment
- User-initiated reset → return to conservative defaults

### FR-3: Auto-Calibration System
- **Progressive Learning**: System learns duty cycle mappings for boost targets through safe, progressive calibration runs
- **Safety Progression**: Start calibration at spring+1 psi overboost limit, gradually increase as system proves safety response
- **Multi-Pass Validation**: Require consistent results across multiple calibration runs before accepting learned values
- **Environmental Adaptation**: Compensate for temperature, altitude, and dome supply pressure variations
- **Calibration Reset**: Provide capability to reset all learned calibration data

### FR-4: Pneumatic System Optimization
- **Input Pressure Analysis**: Calculate optimal air supply pressure for best duty cycle resolution and safety response
- **Real-time Health Monitoring**: Monitor pneumatic system performance and detect suboptimal configurations
- **Optimization Recommendations**: Provide specific recommendations for air supply pressure adjustments
- **Response Time Validation**: Verify that overboost pressure dumps can occur within required timeframes
- **Closed-Bias Wastegate Control**: Optional efficiency mode that keeps wastegate closed until boost limiting is needed
  - Reduces compressed air consumption from dome leakage during normal operation
  - Eliminates atmospheric wastegate dump noise at times when the wastegates can be closed without impacting delivered boost (such as when cruising or idling at vacuum)
  - Uses predictive threshold to maintain system responsiveness
  - **Safety Override**: Closed-bias mode is overridden by safety systems - any fault condition forces 0% duty cycle (wastegate open) regardless of closed-bias setting
  - Always respects 0% OFF setting and safety cuts
  - Emulates ultra-low virtual spring pressure without physical limitations during normal operation

### FR-5: Safety & Fault Management
- **Overboost Protection**: Immediate duty cut to 0% when manifold pressure exceeds configured limits
- **Progressive Overboost Limits**: Only increase overboost limits as system proves adequate safety response capability  
- **Fault Response**: Any sensor, CAN, or storage fault results in duty=0% with visible fault reason and logged timestamp
- **CAN Dependency**: Lost CAN torque signals treated as fault condition (no fallback to pressure-only mode)
- **High-Authority Recognition**: System recognizes that small duty cycle changes can produce large boost changes

### FR-6: Learning & Adaptation
- **Bounded Learning**: All learned duty cycle adjustments have slew rate limits and absolute bounds
- **Environmental Compensation**: Learn compensation factors for temperature, altitude, and supply pressure variations
- **Confidence Tracking**: Track calibration confidence and data quality metrics
- **Separate Storage**: Learned data stored separately from user configuration

📋 **Complete learning specification**: See **[LearnedData.md](LearnedData.md)** for detailed requirements on all learned parameters

### FR-7: User Interface & Monitoring
- **Real-time Display**: TFT display showing boost gauge, targets, calibration progress, pneumatic health
- **System Status**: Display current aggression level, system state, fault conditions, calibration progress
- **Configuration Interface**: JSON/CLI protocol for configuration, calibration control, system recommendations
- **Diagnostic Information**: Provide torque signals, pressure readings, learned duty cycles, system health data

## Performance Requirements

### PF-1: Real-time Operation
- **Control Loop Frequency**: 100 Hz minimum control loop execution
- **Response Time**: Overboost detection and response within 100ms maximum
- **CAN Processing**: Process CAN messages with minimal latency

### PF-2: Reliability & Availability
- **Fault Detection**: Detect sensor and communication faults within 200ms
- **Recovery Time**: System ready for operation within 5 seconds of power-on
- **Data Persistence**: Learned calibration data preserved across power cycles

## Platform Requirements

### PL-1: Hardware Abstraction
- **Multi-platform Support**: Support different MCU platforms through HAL abstraction
- **Sensor Flexibility**: Support different pressure sensor types and ranges
- **CAN Adaptability**: Support different vehicle CAN protocols through HAL implementations
- **Component Upgrades**: Enable hardware component upgrades without core logic changes

### PL-2: Development & Testing
- **Desktop Simulation**: All control logic must be testable on desktop without hardware dependencies
- **Comprehensive Testing**: Unit and integration test coverage for all safety-critical functionality
- **Mock Hardware**: Complete mock HAL implementation for testing scenarios

## Constraints & Limitations

### Phase 1 Scope
- Fixed torque target strategy (configurable percentage, not adaptive)
- Manual aggression adjustment (user must set desired power level via single parameter)
- Ford Gen2 Coyote CAN protocol only
- Basic environmental compensation
- Boost pressure configuration only (power output varies by engine setup)

### Future Phase Scope (Phase 2+: "Beyond RumbleDome")
- **Multi-variable Environmental Compensation**: Advanced algorithms for temperature, altitude, humidity
- **Platform Expansion**: Support for additional vehicle CAN protocols and engine platforms