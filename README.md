# RumbleDome: Torque-Following Electronic Boost Controller

Welcome to **Mad Hacks: RumbleDome** — a revolutionary electronic boost controller that replaces complex configuration with a **single control knob** from "puppy dog" to "brimstone."

Built on Teensy 4.1 and written in Rust, RumbleDome implements intelligent **torque-following control** that automatically provides the boost your ECU is asking for, scaled by your preference for responsiveness and aggression.  

---

## ⚠ NOTE and WARNING

This project is experimental on basically every level. I am working through it to teach myself a number of things:  

- **Rust as a programming language**  
- **AI collaboration** — how to design and structure a development process that makes use of AI to produce consistent and usable results without ending up with a complete pile of trash at the end. The AI can work and reason somewhat autonomously, but I reserve the control to override any decision anywhere in the code.  
- **Microcontroller programming** — this is my first from-scratch firmware.  
- **Electronic boost control theory and physics** — I have a very specific goal I am aiming for in the level of integration and control between my aftermarket turbo system and the OEM systems.  
- **Ford CAN bus** — specifically for the stock (non-FRPP) Gen 2 Coyote engine management.  
- **Basic electronics** — because why bother learning with blinky LEDs and elementary exercises when I have a fun idea that no one else has ever built, along with the potential to blow up an expensive engine if things go south...  

I'm making this open source and available on the off chance that someone else might find it interesting or educational.  

**Legal / liability disclaimers:**  
- See adjacent LICENSE.md

Consider yourself warned.

**🎯 What Makes RumbleDome Different:**

RumbleDome is **not a traditional boost controller** - it's a **torque request amplifier** that works in harmony with your ECU to drive the turbos. Instead of fighting the ECU with predefined boost curves, RumbleDome reads the ECU's torque requests via CAN bus and intelligently provides exactly the boost needed to help achieve those targets. 

**Key Innovations**: 
- **Torque management awareness**: RumbleDome interfaces with the CAN bus, reads the ECU's desured vs actual torque in realtime, and uses the delta to decide how much boost to add or remove to meet the ECU's torque requests.
- **Single control knob**: Sets how aggressively RumbleDome manages the turbos to achieve the ECU's desired torque.
- **Simplified Configuration**: User sets the mechanical spring pressure, overboost PSI, and nominal maximum operating PSI during initial setup.  The aggression level is set via a simple dial encoder and can be changed on the fly.  
- **Self-learning**: Fully closed-loop operation. The rest of the operating parameters are auto-learned by watching the appropriate sensors and engine telemetry (similar to how the ECU learns to adjust fuel trims) so the system improves over time and adjusts to transient environmental changes without user intervention.

--

## 📖 Project Overview
📋 **Complete technical specifications**: [TechnicalSpecs.md](docs/TechnicalSpecs.md)

- **Firmware**: Rust (no unsafe where possible), modular design 
- **Hardware**: Teensy 4.1 MCU, 4-port MAC solenoid, 4 pressure sensors, ST7735R display, CAN interface, Bluetooth, MicroSD, custom PCB 'mainboard' to host the teensy and additional circuitry needed for power management, CAN integration, and other sensors.
- **Concept**: Revolutionary single-knob torque-following control that eliminates configuration complexity while providing comprehensive operational envelope scaling. Includes closed-bias wastegate control for efficiency and 4-sensor dome diagnostics.  

---

## 🗂 Repo Structure
- `/crates` → Rust workspace with modular crates for hardware abstraction, control logic, and testing.  
- `/docs` → Design documents and specifications.

### 📚 Documentation Reading Order

**Essential Foundation (read in order):**
1. `Context.md` → High-level design context and goals
2. `Physics.md` → Turbo system physics and control theory fundamentals  
3. `Requirements.md` → Functional and performance requirements
4. `Safety.md` → Safety requirements and critical constraints
5. `AI_Philosophy.md` → Human-AI collaboration methodology and boundaries

**Implementation Details:**
6. `Architecture.md` → System design and component architecture
7. `Hardware.md` → Hardware abstraction layer and platform specifications
8. `CAN_Signals.md` → Ford Gen2 Coyote CAN bus signal specifications
9. `Protocols.md` → JSON/CLI communication protocol specifications

**Development & Reference:**
10. `Implementation.md` → Code structure, build process, and development workflow
11. `TestPlan.md` → Testing strategy and validation procedures
12. `Definitions.md` → Acronyms, jargon, and domain-specific terminology
13. `BeyondRumbleDome.md` → Future enhancement concepts  

---

---

## 🚀 Quick Start

**👨‍🔬 For Researchers/Students**: Want to understand the technical approach?  
→ Read [Context.md](docs/Context.md) and [Physics.md](docs/Physics.md)

**👩‍💻 For Developers**: Ready to contribute or build?  
→ Follow the [documentation reading order](docs/README.md) starting with Context → Physics → Requirements

**🔧 For Users**: Want to build and install?  
→ Hardware assembly and installation guides coming in Phase 8

**📊 For Data/Tuning**: Want to understand the protocols?  
→ See [Protocols.md](docs/Protocols.md) for JSON/CLI interface specification

**🔧 For Systematic Engineering**: Want to use our AI-Traceable Engineering tools?  
→ See [Quick Start Guide](SYSTEMATIC_ENGINEERING_QUICKSTART.md) or [Full Tool Documentation](docs/SystematicEngineeringTool.md)

## ✨ Technical Highlights

**🎛️ Single-Knob Revolution**: Eliminates complex boost curve configuration with one intuitive control  
**🤝 ECU Cooperation**: Works with your ECU's torque management, never fights it  
**🔒 Safety-First**: Multiple layers of protection with fail-safe defaults (0% duty = minimal boost)  
**📡 CAN Integration**: Real-time torque data from Ford Gen2 Coyote ECU  
**⚡ Real-Time Control**: 100Hz control loop with 30Hz PWM synchronization

*For detailed technical achievements, see [Architecture.md](docs/Architecture.md)*

---

## 🚀 Current Status

**Design Complete - Ready for Fresh Implementation**

The comprehensive design and specification work has been completed, providing a solid foundation for implementation. All documentation has been refined and aligned around the torque-following architecture.

*For detailed development phases, progress tracking, and current task status, see [DEVELOPMENT_STATUS.md](DEVELOPMENT_STATUS.md)*

## 🤖 AI Working Agreements

**📋 Complete AI collaboration methodology**: See **[AI_Philosophy.md](docs/AI_Philosophy.md)** for comprehensive human-AI partnership principles

**Key principles for AI assistance on this project:**
1. **Work as staff engineer**: Request clarification for engineering decisions outside AI expertise rather than guessing
2. **Preserve architect authority**: Never override human engineering specifications or safety requirements
3. **Maintain systematic traceability**: Every implementation must trace to T1→T2→T3→T4 specifications
4. **Implement exactly as specified**: Don't "improve" algorithms without explicit architect direction
5. **Flag insufficient specification**: Stop and request architect input rather than making engineering assumptions
6. **Respect domain boundaries**: Don't make decisions requiring automotive/turbo/control theory expertise
7. **Professional communication**: Request clarification like a staff engineer implementing architectural vision

---

## 🛡️ Development Principles
- Documentation is the **single source of truth**  
- Code must be **verbose, modular, and testable**  
- Failure paths **always fail safe** (drop to zero boost)
- Safety requirements take precedence over performance  

---

## 📋 Documentation Management: AI-Traceable Engineering

**RumbleDome uses systematic traceability** - every technical decision links back to foundational requirements through T1→T2→T3→T4 IDs. This creates a "journaled filesystem" for engineering decisions.

### **Maintenance Strategy**

**✅ Safe Editing Approach:**
1. **Edit freely** - focus on content quality and clarity
2. **Commit incremental changes** - frequent git commits capture stable states
3. **Run periodic validation** - use AI to check traceability integrity
4. **Batch repairs** - fix linkage issues in dedicated sessions

**🔧 Validation Checklist:**
- All `T1-xxx → T2-xxx → T3-xxx → T4-xxx` links point to existing content
- Cross-references (`docs/File.md`) resolve correctly  
- Decision classifications remain consistent
- No orphaned traceability IDs

**⚠️ Recovery Process:**
If documentation becomes inconsistent:
1. **Git rollback** to last known-good state
2. **Selective restoration** of working sections
3. **AI-assisted repair** of broken linkages
4. **Incremental validation** before continuing

**🤖 AI Role in Maintenance:**
- **Traceability validation** - verify all links remain valid
- **Cross-reference repair** - fix broken documentation links  
- **Impact analysis** - identify changes that affect multiple tiers
- **Consistency enforcement** - ensure decision classifications align

This approach preserves the systematic traceability benefits while keeping documentation maintainable. The journaling creates recovery points, and AI assistance scales the complexity management.

---

## 🤝 Contributing

**Want to help?** 
- 📖 **Documentation**: Improvements to clarity and accuracy always welcome
- 🧪 **Testing**: Help validate control algorithms and safety systems  
- 🔧 **Hardware**: Real-world sensor calibration and CAN signal verification
- 💻 **Code**: See [Implementation.md](docs/Implementation.md) for development setup

**Process**: Fork → Branch → Pull Request  
**Questions?** Open an issue for discussion before major changes

**📋 Documentation Contributors**: When editing docs with T1-T4 traceability IDs, feel free to focus on content quality. Traceability consistency can be validated and repaired in separate maintenance sessions.

---

*Mad Hacks: RumbleDome — because sometimes boost control needs a little chaos, carefully engineered.*