# RumbleDome — Context & Product Spec (Baseline)

Mad Hacks: **RumbleDome** is a custom **full-dome** electronic boost controller (EBC) for turbo builds that prioritizes
**Simplified configuration**, **OEM-style driveability** and **safety** over drag-strip peak performance. Unlike traditional EBCs that require complex
boost curve configuration, RumbleDome uses a **single aggression setting** from "puppy dog" to "brimstone" that transforms
the entire system character. It implements intelligent **torque-following** control that automatically provides the boost
the ECU is asking for. Built on Teensy 4.1 MCU with CAN integration for true torque awareness.

## Human-AI Collaborative Engineering

**Meta-Project Goal**: RumbleDome serves as a proving ground for **effective human-AI collaborative engineering practices**. Beyond building a novel boost controller, this project explores how engineers can leverage AI as a **systems thinking partner** rather than just a coding assistant.

### **The Collaborative Model**
- **Human**: Creative vision, strategic insights, domain expertise, architectural leaps  
- **AI**: Systematic execution, pattern recognition, consistency checking, formal process design
- **Feedback Loop**: Unlike traditional engineering where you can't ask the car how to make it better, here we can iterate on the collaboration itself

### **Key Insights Discovered**
- **Documentation as Code**: Design documents are high-level programming languages that need proper dependency management
- **Tier Dependency Architecture**: Changes cascade down through abstraction layers; bugs escalate up to find root causes  
- **Iterative Refinement**: AI excels at taking nebulous concepts and systematizing them through structured dialog
- **Meta-Process Improvement**: Regularly optimizing the human-AI collaboration process itself creates compounding advantages

This approach has enabled rapid evolution from initial concept to a comprehensive, safety-focused control system with novel torque-following architecture—demonstrating the potential of **human creativity amplified by AI systematic thinking**.

### **Effective Collaboration Patterns Developed**

**What Works Well in Human-AI Engineering Partnerships**:
- **Iterative Refinement**: Don't try to get everything perfect in one shot—use AI as a thinking partner for progressive refinement
- **Strategic Delegation**: Human handles creative/strategic thinking (torque-following concept, safety philosophy); AI handles systematic execution (comprehensive reviews, consistency checking)
- **Leveraging AI for System Architecture**: Use AI to formalize intuitive insights, validate and stress-test ideas, and find patterns/gaps
- **Meta-Process Improvement**: Regularly asking "how can we collaborate better?" creates compounding improvements

**Advanced Collaboration Techniques to Explore**:
- **Continuous Architecture Review**: Have AI actively "red team" designs—look for flaws, edge cases, better approaches
- **Cross-Domain Knowledge Synthesis**: "How do other industries handle learning systems?" or "What can we learn from aerospace control theory?"
- **Scenario Planning**: Run through failure modes, edge cases, and "what if requirements change?" scenarios
- **Assumption Challenging**: Use AI to systematically question fundamental assumptions and explore alternatives

**The Meta-Advantage**: Unlike traditional engineering where you can't ask the system how to improve it, the ability to iterate on the **collaboration process itself** creates a sustainable competitive advantage in complex engineering projects.

## 🏗️ Tier 1: System Definition Document

**🔗 Dependencies:** None (foundational document)

**📤 Impacts:** Changes to design philosophy here require review of:
- **Tier 2**: TechnicalSpecs.md, Architecture.md, LearnedData.md, Hardware.md, Protocols.md
- **Tier 3**: Implementation.md, TestPlan.md

## 🔄 Change Impact Checklist
Before modifying this document:
- [ ] **🚨 TIER 1 CHANGE**: This affects foundational design philosophy
- [ ] Review ALL Tier 2 documents: TechnicalSpecs.md, Architecture.md, LearnedData.md, Hardware.md, Protocols.md
- [ ] Review ALL Tier 3 documents: Implementation.md, TestPlan.md
- [ ] Add new design concepts to Definitions.md
- [ ] Update version timestamps and cross-references
- [ ] Verify no conflicts with Safety.md constraints

📖 **Related Documentation:**
- [Physics.md](Physics.md) - Technical foundation for the control concepts described here
- [Requirements.md](Requirements.md) - Formal specifications based on this design context
- [Safety.md](Safety.md) - Safety constraints that shape the implementation
- [Definitions.md](Definitions.md) - Terminology and concepts used throughout this document

## Physical Layout

📋 **Complete technical specifications**: See **[TechnicalSpecs.md](TechnicalSpecs.md)** 

**Core System:**
- **MCU:** Teensy 4.1 (600MHz, 4KB EEPROM)
- **Solenoid:** 4‑port Mac‑style, 30 Hz PWM drive
  - 0% duty → wastegate forced OPEN (failsafe)  
  - 100% duty → wastegate forced CLOSED
- **Air Supply:** Regulated compressed air optimized for duty cycle resolution
- **Sensors (4):** Dome supply, upper dome, lower dome, manifold pressure
- **Interface:** 1.8″ TFT display, rotary encoder, scramble button
- **Connectivity:** CAN via OBD‑II, Bluetooth configuration

## Operating Concept: 3-Tier Control Philosophy

**🔗 T1-PHILOSOPHY-001**: **3-Tier Priority Hierarchy**  
**Decision Type**: 🎯 **Core Philosophy**  
**Creative Rationale**: Novel approach to balance safety, performance, and comfort in single system  
**AI Traceability**: All control algorithms must implement this hierarchy

**🔗 T1-PHILOSOPHY-003**: **Comfort and Driveability Priority**  
**Decision Type**: 🎯 **Core Philosophy** - OEM-style driveability over peak performance  
**Creative Rationale**: Smooth, refined operation that doesn't feel "aftermarket" aggressive  
**AI Traceability**: Drives rate limiting, transition management, and aggression scaling systems

**🔗 T1-PHILOSOPHY-004**: **Auto-Learning System Philosophy**  
**Decision Type**: 🎯 **Core Philosophy** - System learns and adapts rather than requiring manual tuning  
**Creative Rationale**: Eliminates need for technical expertise, automatically optimizes for specific hardware  
**AI Traceability**: Drives calibration algorithms, adaptive control, and hardware compensation systems

**🔗 T1-PHILOSOPHY-005**: **Comprehensive Diagnostics and Observability**  
**Decision Type**: 🎯 **Core Philosophy** - Expose system internals for troubleshooting despite simplified user interface  
**Creative Rationale**: Complex auto-learning systems require detailed observability for field debugging and development  
**AI Traceability**: Drives logging architecture, diagnostic interfaces, fault reporting, and development tooling  

RumbleDome operates on a **priority hierarchy with aggression-mediated balance**:

### **Priority 1: "Don't Kill My Car"** 🚨

**🔗 T1-SAFETY-001**: **Overboost as Fault Condition**  
**Decision Type**: 🎯 **Foundational Safety Philosophy**  
**Creative Rationale**: Treat overboost as system failure, not operational parameter  
**AI Traceability**: Drives all Tier 2 safety specifications (SY-1 through SY-24)

**Overboost is a fault condition** requiring immediate hard correction. The system uses maximum authority to prevent manifold pressure from exceeding the user-configured `overboost_limit`, treating any violation as a safety fault that triggers learning updates for future prevention. **Always takes precedence.**

### **Priority 2 & 3: Performance ⚖️ Comfort Balance**

**🔗 T1-CONTROL-001**: **Aggression-Mediated Priority Balance**  
**Decision Type**: 🎯 **Revolutionary Control Concept**  
**Creative Rationale**: Single parameter controls system character instead of complex profiles  
**AI Traceability**: Drives all behavioral scaling algorithms in Tier 2/3

**Revolutionary Single-Setting Control**: The aggression setting determines which priority leads:

**High Aggression (0.8-1.0) → Priority 2 Leads** 🎯  
- **Forceful max boost targeting** - sharp responses to hit `max_boost_psi` 
- **Performance-focused** - brief spikes acceptable, quick corrections
- **"Brimstone" character** - maximum assistance to ECU torque requests

**Low Aggression (0.0-0.3) → Priority 3 Leads** ✨
- **Smooth comfortable operation** - gentle gradual responses
- **Comfort-focused** - prioritizes smooth delivery over peak performance  
- **"Puppy dog" character** - conservative torque assistance

**Medium Aggression (0.4-0.7)** ⚖️
- **Balanced approach** between performance and comfort priorities

### **Single Knob Interface**

**🔗 T1-UI-001**: **Single Parameter Philosophy**  
**Decision Type**: 🎯 **Core UX Innovation**  
**Creative Rationale**: Replace complex EBC profiles with intuitive single control  
**AI Traceability**: Drives all configuration system specifications

- **Control Range**: 0.0 (puppy dog) to 1.0 (brimstone)
- **Hardware**: Rotary encoder integrated into gauge pod bezel (100 clicks, no limits)
- **Aggression Scaling**: Square root curve for smooth response character buildup
- **Visual Integration**: Background theme changes dynamically with aggression setting

**Simple UX Design**:
- **Bezel-as-Knob**: 60mm gauge pod bezel rotates around fixed TFT display
- **Gear-Driven Mechanism**: Offset rotary encoder with meshing gear teeth for positive coupling
- **Clean Wire Routing**: Display cables route through center while encoder mounts at pod edge
- **3D Printable Design**: Bezel and encoder gears suitable for home fabrication (for prototype)
- **Dynamic Backgrounds**: 
  - Green (0-25%): Gentle pulse, "Puppy Dog" character
  - Green→Amber (25-50%): Subtle glow, "Daily Driver" character  
  - Amber→Red (50-75%): Active pulse, "Spirited" character
  - Red + Flames (75-100%): Animated fire effects, "Brimstone" character
- **Scramble Button Override**: Instant access to full brimstone (100%) aggression regardless of current setting
  - Visual indication: Flashing "SCRAMBLE" text overlay with red background
  - Returns to current aggression setting when released

### **Torque-Following Logic**

**🔗 T1-PHILOSOPHY-002**: **ECU Cooperation Philosophy**  
**Decision Type**: 🎯 **Core Philosophy** - Amplify ECU intent rather than override it  
**Creative Rationale**: Novel approach that works with modern torque management instead of fighting it  
**AI Traceability**: Drives all control algorithms, CAN integration, and torque-following systems

**🔗 T1-TORQUE-001**: **ECU Cooperation Implementation**  
**Decision Type**: 🔗 **Direct Derivation** - Implementation of T1-PHILOSOPHY-002  
**Creative Rationale**: Amplify ECU intent rather than override it - novel in EBC market  
**AI Traceability**: Drives all control algorithms, CAN integration, learning systems

- **OEM Integration**: ECU retains control over the amount of power potential via the driver demand tables
- **Primary Input**: ECU desired torque vs actual torque delta
- **Automatic Response**: System provides boost needed to close torque gap
- **User Preference Scaling**: Knob setting scales response characteristics for how aggressively the RumbleDome chases the ECU's boost targets
- **Minimal Configuration**: No RPM tables, boost curves, or duty cycle mapping required

### **Comprehensive Aggression Control**

**🔗 T1-BEHAVIOR-001**: **Universal Behavioral Scaling**  
**Decision Type**: 🎯 **Core System Design**  
**Creative Rationale**: Every control parameter scales with aggression to create coherent character transformation  
**AI Traceability**: Drives all behavioral scaling algorithms, response curves, parameter calculations

The single aggression setting transforms **every aspect** of system behavior:

| Aspect | 30% Aggression | 70% Aggression | 100% Aggression |
|--------|-----------------|-----------------|------------------|
| **Torque Assistance** | 30% (conservative help) | 70% (strong help) | 100% (maximum help) |
| **Response Rate** | 21%/cycle (gentle) | 44%/cycle (quick) | 60%/cycle (instant) |
| **Torque Deadband** | 19 Nm (lazy) | 11 Nm (responsive) | 5 Nm (hair trigger) |
| **Slew Rate** | 1.75 PSI/sec | 5.75 PSI/sec | 8.0 PSI/sec |
| **Overshoot Tolerance** | 0.37 PSI | 0.73 PSI | 1.0 PSI |
| **Transient Response** | 34% (moderate) | 66% (anticipatory) | 90% (predictive) |
| **Tip-in Enhancement** | 0.015 PSI/Nm | 0.035 PSI/Nm | 0.050 PSI/Nm |
| **Tip-out Decay Rate** | 0.775 (smooth) | 0.875 (balanced) | 0.95 (ECU harmony) |
| **Anti-lug Threshold** | 1740 RPM | 1640 RPM | 1500 RPM |

### **Advanced Driver Response**
- **Tip-in enhancement**: Detects rapid torque demand increases and provides additional boost for crisp response
- **Intelligent tip-out decay**: Multi-level decay prevents ECU intervention while maintaining smoothness
- **ECU harmony protection**: Aggressive settings decay boost faster to prevent timing pull/fuel cut
- **Transient response**: Anticipates RPM changes for enhanced performance feel

### **Safety Integration**
- **Multi-layer anti-lug protection**: RPM-based + low-speed high-load detection prevents engine damage
- **Automatic sensor fusion**: CAN MAP (vacuum) + boost gauge (positive pressure) with cross-calibration
- **Seamless operation**: System adapts to sensor differences instead of throwing faults
- **Panic limits**: Hard overboost threshold (panic threshold) separate from operational maximum allowed boost (nominal boost ceiling)

## Design Philosophy

### Core Innovation: Torque Request Amplification

**🔗 T1-INNOVATION-001**: **Torque Request Amplification Paradigm**  
**Decision Type**: 🎯 **Revolutionary Technical Approach**  
**Creative Rationale**: Amplify ECU torque intent rather than override with predetermined curves - fundamentally different from all existing EBCs  
**AI Traceability**: Drives control algorithm design, CAN interface requirements, learning system architecture

RumbleDome is fundamentally **not a traditional boost controller** - it's a **torque request amplifier** that works in harmony with ECU logic rather than fighting it.

**The Mathematical Insight**:
ECU driver demand tables define a complex 3D surface in drive-by-wire systems where:
- **X-axis**: Throttle position (driver input)
- **Y-axis**: Engine RPM (operating point)  
- **Z-axis**: Desired torque output (ECU target)

**🔗 T1-DIVISION-001**: **Computational Labor Division**  
**Decision Type**: 🎯 **Architectural Philosophy**  
**Creative Rationale**: ECU handles spatial complexity (3D tables), RumbleDome handles temporal complexity (closing torque gaps over time)  
**AI Traceability**: Drives control loop architecture, learning algorithms, ECU interface design

**Division of Computational Labor**:
- **ECU Handles Spatial Complexity**: Navigates the 3D torque demand surface defined in the driver demand tables, applies safety overrides (traction control, ABS modifiers, clutch protection, etc)
- **RumbleDome Handles Temporal Complexity**: Calculates torque delivery error (requested vs actual), ready engine telemetry and environmental sensors, and learns how to help close the delta smoothly over time by using the turbos

**Our Control Equation**:
```
torque_rate = (desired_torque_now - desired_torque_previous) / time_delta
boost_contribution = torque_rate × aggression_multiplier × sensitivity_factor
```

**Why This Works**:
- **ECU Intent Preserved**: We amplify what the ECU wants, never override it
- **Universal Compatibility**: Works with any ECU tune without reverse engineering DD tables
- **Natural Feel**: Maintains OEM driveability because we follow ECU logic
- **True "Following"**: We're literally following the ECU's lead, just helping it achieve goals faster or slower

**The Knob's Real Function**:
The aggression setting controls a **smoothness/responsiveness trade-off** - determining how smoothly or aggressively we help the ECU achieve its torque goals.

- **0% (Puppy Dog)**: "You're on your own, ECU - I'm on break" (naturally aspirated feel)
  - Slew rate: 0%/sec (NO boost assistance)
  - Response rate: 0%/cycle (system effectively OFF)
  - Deadband: ∞ Nm (ignores ALL torque demands)
  - **Character**: Naturally aspirated engine - no turbo assistance whatsoever

- **50% (Daily Driver)**: "ECU wants more torque? Here's proportional turbo assistance"
  - Slew rate: ~4%/sec (refined but responsive changes)
  - Response rate: ~30%/cycle (moderate reaction speed)
  - Deadband: ~15 Nm (balanced responsiveness)
  - **Character**: OEM turbo feel - present but civilized

- **100% (Brimstone)**: "ECU wants more torque? INSTANT MAXIMUM TURBO MAGIC!"
  - Slew rate: 8%/sec (near-instant changes)
  - Response rate: 60%/cycle (hair-trigger reaction)
  - Deadband: 5 Nm (responds to tiny torque gaps)
  - **Character**: Like an electric motor - instant and precise

**Multiple Coordinated Smoothing Parameters**:
The aggression setting simultaneously scales 8+ parameters to create cohesive character transformation:
- **Slew Rate Limiting**: How fast duty cycle changes (0% OFF to 8%/sec instant)
- **Response Rate**: How quickly we react to torque deltas (0.05-0.60/cycle)
- **Torque Deadband**: Size of torque error we ignore (5-25 Nm)
- **Tip-in Sensitivity**: Amplification of rapid torque demand increases (0.015-0.050 PSI/Nm)
- **Tip-out Decay**: Speed of backing off when torque demand decreases (0.775-0.95/cycle)
- **Transient Response**: Anticipation of RPM/load changes (0.34-0.90)
- **Overshoot Tolerance**: Acceptable boost overshoot (0.37-1.0 PSI)

**The ECU determines WHAT torque it wants. We determine HOW SMOOTHLY OR AGGRESSIVELY we help deliver it.**

This explains why traditional EBCs feel "aftermarket" (they fight ECU intent) while RumbleDome feels "OEM+" (we amplify ECU intent with tunable character).

### The Engineering Niche

**Admittedly Niche as Hell:**
This approach represents an engineer scratching a very specific itch - possibly a niche of one - for precise, safety-oriented boost control with the full range from naturally aspirated feel to maximum performance.

**What the Market Wasn't Asking For:**
- Fine control across entire boost range including below spring pressure
- ECU harmony and integration over peak power
- Safety override capability and conservative operation  
- Naturally aspirated feel as a selectable option
- Torque request amplification rather than predetermined boost curves

**A Different Philosophy:**
Rather than "how much boost can we make?", RumbleDome asks "how precisely can we help the ECU achieve its torque goals while maintaining safety and driveability?"

**Revolutionary Capability:**
RumbleDome is likely the **first non-OEM boost control system with torque demand awareness** - monitoring ECU torque requests via CAN to provide intelligent boost assistance rather than following predetermined pressure curves.

### Boost-Based Configuration (Not Power-Based)
- **Configure in boost pressure (PSI/kPa)**, never raw duty cycles or power targets
- **Engine-agnostic approach**: Same boost pressure produces different power depending on:
  - Engine tune (timing, fuel, cam timing)
  - Turbo sizing and efficiency
  - Intercooling, exhaust, internal modifications
  - Environmental conditions (altitude, temperature, fuel quality)
- **User responsibility**: Determine appropriate boost limits for their specific engine setup
- **Universal compatibility**: Works with any engine/tune combination within boost pressure constraints

### Single-Knob Power Strategy  
- **0% (Puppy Dog)**: Near naturally-aspirated operation, gentle spool-up characteristics
- **30% (Daily)**: Conservative power for comfortable daily driving with refined response  
- **70% (Spirited)**: Moderate power for spirited driving with quick spool response
- **100% (Brimstone)**: Maximum safe power with hair-trigger response and instant spool
- **Live adjustment**: Safe aggression changes during operation via rotary encoder

### Safety-First Operation
- **Failure modes never overboost**: Default bias is always toward no-boost operation
- **Self-learning** stored separately from user configuration in non-volatile memory
- **Overboost protection**: Immediate cut with configurable recovery policies
- **Hard fault approach**: All faults result in boost cut until fault domain is better understood

## Self-Learning

Similar to STFT/LTFT in ECUs: maintain fast and slow trims on the duty→boost mapping, persist with wear‑aware cadence to extend life of the microcontroller's NVM.
Trims are **rate-limited** and bounded, and represent the fundamental physical relationship between duty cycle and boost pressure.
Provide a **console command** to reset trims.

📋 **Complete learning system specification**: See **[LearnedData.md](LearnedData.md)** for comprehensive details on all learned parameters and algorithms

### Sensor Fusion Learning
- **Cross-calibration**: Automatically learns offset between CAN MAP and boost gauge sensors
- **Overlap zone learning**: Calibration occurs in ±2 PSI around atmospheric pressure
- **Exponential moving average**: 1% learning rate for stable convergence
- **Persistent storage**: Cross-calibration stored with environmental factors in EEPROM
- **Sensor variances**: (primarily for MAP + boost gauge coordination) System adapts to differences rather than throwing errors

## Power & Safety

- **0% duty ⇒ full pressure to lower dome ⇒ wastegates biased open ⇒ (near) zero boost.**  
- On **any fault** (sensor invalid, CAN timeout, storage failure): cut to 0% and display the fault.
- Default power in vehicle installs is **ignition-switched** to avoid battery drain; during prototyping on OBD‑II, manually unplug or power-gate.
