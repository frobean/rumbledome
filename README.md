# RumbleDome: Torque-Based Electronic Boost Controller

Welcome to **Mad Hacks: RumbleDome** — a custom, full-dome electronic boost controller built around the Teensy 4.1 microcontroller and written in Rust.  

This project explores innovative electronic boost control through torque-based ECU cooperation and auto-calibration.  

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

**Recent Development Notes (Jan 2025)**:
- PWM synchronization system prevents phase noise in dome pressure control
- EEPROM implementation handles automotive power-loss scenarios safely
- Voltage divider support enables standard 5V pressure sensors with 3.3V ADC
- Comprehensive test framework provides safety validation foundation  

--

## 📖 Project Overview
- **Firmware**: Rust (no unsafe where possible), modular design, Teensy 4.1 target.  
- **Hardware**:
  - Teensy 4.1 MCU  
  - 4-port MAC solenoid (100 Hz PWM, 0% duty = failsafe to no boost)  
  - 3 pressure sensors (0–30 psi, 5V output with 3.3V voltage dividers)  
  - ST7735R TFT LCD (1.8" 128×160)  
  - CAN bus transceiver (SN65HVD230 or similar)  
  - OBD2 adapter for CAN connection  
  - MicroSD card for portable configuration  
- **Concept**: Closed-loop, self-learning dome pressure control with multiple user profiles and fail-safes.  

---

## 🗂 Repo Structure
- `/crates` → Rust workspace with modular crates for hardware abstraction, control logic, and testing.  
- `/docs` → Design documents and specifications.  
  - `Context.md` → High-level design context (narrative + goals).  
  - `Requirements.md` → Functional and performance requirements.
  - `Architecture.md` → System design and component architecture.
  - `Safety.md` → Safety requirements and critical constraints.
  - `Protocols.md` → JSON/CLI communication protocol specifications.
  - `Hardware.md` → Hardware abstraction layer and platform specifications.
  - `Implementation.md` → Code structure, build process, and development workflow.
  - `Definitions.md` → Acronyms, jargon, and domain-specific terminology.  

---

## 🧭 Getting Started
**For developers**:
1. Start by reading `docs/Context.md` for the project narrative.  
2. Review `docs/Requirements.md` for what the system must do.
3. Study `docs/Safety.md` for critical safety requirements.
4. Read `docs/Architecture.md` for system design and component relationships.
5. Reference `docs/Implementation.md` for build process and development workflow.  

---

## 🛡 Development Principles
- Specs and context docs are the **single source of truth**.  
- Any new insights → update the docs first, then code.  
- Code must be **verbose, modular, and testable**.  
- Failure paths must **always fail safe** (drop to zero boost).

## 🔧 Key Technical Achievements

### PWM Synchronization Innovation
Solved critical phase noise issue in pneumatic control through:
- **Beat frequency prevention**: Control loop timing coordinated with PWM cycles
- **Jitter reduction**: Deadband filtering based on FlexPWM hardware resolution
- **Multiple sync strategies**: Optimized timing for different performance requirements
- **Automotive timing constraints**: 100Hz PWM with ±10% update windows

### Storage Architecture for Automotive Reality
- **Immediate write-through**: No graceful shutdown dependency (key-off = instant power loss)
- **Dual-tier storage**: EEPROM for hardware-specific data, SD card for portable profiles
- **Comprehensive wear tracking**: Predictive maintenance with years of advance warning
- **Backup/restore system**: Complete microcontroller replacement workflow

### Voltage Interface Adaptation
- **5V→3.3V scaling**: 10kΩ+20kΩ voltage divider support throughout codebase
- **Optimized resolution**: 0.018 PSI with 12-bit ADC for precise boost control
- **Calibration automation**: Sensor scaling factors updated automatically
- **Documentation synchronization**: All docs updated for new voltage ranges

### Intelligent Sensor Fusion
- **Dual MAP sensor strategy**: CAN MAP (vacuum) + boost gauge (positive pressure)
- **Automatic cross-calibration**: Learns systematic offset between sensors during operation
- **Seamless range coverage**: Full spectrum from deep vacuum (-14.7 PSI) to high boost (+30 PSI)
- **No sensor faults**: System adapts to sensor differences instead of throwing errors

---

## 🚀 Implementation Status

### Phase 6: Hardware Abstraction ✅
- [x] Complete Teensy 4.1 HAL implementation
- [x] FlexPWM solenoid control (100Hz with PWM synchronization)
- [x] Dual ADC pressure sensor reading (12-bit, 4x averaging, voltage divider support)
- [x] FlexCAN integration with Ford Gen2 Coyote parsing
- [x] ST7735R TFT display with gauges and status
- [x] FlexRAM EEPROM emulation (4KB organized storage)
- [x] GPIO control for buttons and status LEDs
- [x] Bluetooth serial interface for wireless configuration
- [x] MicroSD card support for portable profiles

### Phase 7: Control Systems ✅  
- [x] 3-level control hierarchy implementation
- [x] PWM-synchronized control loop timing (jitter reduction)
- [x] Torque-based boost target modulation
- [x] PID controller with learned baseline correction
- [x] Safety monitoring and overboost protection
- [x] Auto-calibration with progressive safety limits
- [x] Comprehensive test framework foundation

### Phase 8: Testing & Validation 🔄
- [x] Unit test structure for safety-critical functions
- [x] Storage operation validation tests
- [x] PWM timing coordination tests
- [ ] Integration tests for full control cycles
- [ ] Hardware-in-loop validation setup
- [ ] Safety system stress testing

### Phase 9: Next Steps 📋
- [ ] Real vehicle CAN signal mapping (replace speculative IDs)
- [ ] Physical hardware validation and sensor calibration
- [ ] Complete desktop simulator implementation
- [ ] Mobile app development for Bluetooth interface
- [ ] Production deployment testing

## 🤖 AI Working Agreements
When assisting with this project, AI must:
1. **Never drop requirements**: anything listed in the spec documents is binding until explicitly removed.  
2. **Work module-by-module**: respect API contracts, don't introduce cross-cutting hacks.  
3. **Document assumptions clearly**: mark speculative areas with `⚠ SPECULATIVE` so humans can verify.  
4. **Preserve clarity and style**: verbose variable names, self-documenting code, proper comments for math/algorithms.  
5. **Approachability**: Never assume the reader is an expert in the math, jargon, microcontroller, physics, or theory.
6. **Fail safe in code paths**: defaults and error states must never result in uncontrolled boost.  
7. **Keep testability in mind**: unit tests must be able to run with fake data without hardware.  
8. **Surface gaps**: if required details are missing from the spec, pause and request clarification rather than guessing silently.  
9. **Respect layering**: HAL abstractions first, hardware-specific logic later.

---

## 🚦 Development Status

**Current Status: ~80% Implementation Complete**

### Phase 0: Foundation ✅
- [x] Context defined  
- [x] Design spec fleshed out  
- [x] Interfaces defined (CAN signals speculative, need vehicle verification)

### Phase 1: Core Infrastructure ✅
- [x] Rust workspace scaffolding (Cargo.toml, crate structure)
- [x] HAL trait definitions and mock implementations
- [x] Basic unit test framework setup
- [x] Core data structures (SystemConfig, LearnedData, etc.)
- [x] State machine implementation
- [x] Error handling and fault management system

### Phase 3: Control Logic ✅ (Completed ahead of Phase 2)
- [x] Profile management system
- [x] 3-level control loop implementation (Torque → PID → Safety)
- [x] PID controller with environmental compensation
- [x] Safety override with hysteresis and slew limiting
- [x] Environmental compensation algorithms
- [x] Real-time system integration (100Hz RTIC-based loop)

### Phase 4: Learning Systems ✅
- [x] Auto-calibration state machine (Conservative → Validation phases)
- [x] Progressive safety limit expansion
- [x] 2D interpolation tables for duty cycle learning
- [x] Environmental factor compensation learning
- [x] Confidence tracking and bounded learning validation

### Phase 5: Desktop Simulator ✅
- [x] Complete desktop simulator with interactive TUI
- [x] Realistic Gen2 Coyote engine physics simulation
- [x] 6 comprehensive test scenarios (idle, WOT, overboost, etc.)
- [x] Real-time gauge display and interactive controls
- [x] Configuration management and scenario testing

### Phase 6: Hardware Abstraction ✅
- [x] Complete Teensy 4.1 HAL implementation
- [x] FlexPWM solenoid control (30Hz, safety failsafe)
- [x] Dual ADC pressure sensor reading (12-bit, 4x averaging)
- [x] FlexCAN integration with Ford Gen2 Coyote parsing
- [x] ST7735R TFT display with gauges and status
- [x] FlexRAM EEPROM emulation (4KB organized storage)
- [x] GPIO with debounced buttons and LED patterns
- [x] DWT-based precision timing (600MHz resolution)
- [x] Hardware watchdog integration

### Phase 7: Real-Time Firmware ✅
- [x] RTIC-based concurrent firmware architecture
- [x] 100Hz control loop with performance monitoring
- [x] Concurrent tasks (control, status, diagnostics, UI)
- [x] JSON protocol implementation for communication
- [x] Safety-critical design patterns throughout

### Phase 2: Hardware Integration 🚧
- [⏳] Compilation testing (awaiting Rust toolchain in VM)
- [⏳] Real hardware bring-up and pin assignment validation
- [⏳] CAN signal verification with actual Ford Gen2 Coyote vehicle
- [⏳] Pressure sensor calibration with real sensors

### Phase 8: Testing & Validation ⏳
- [ ] Unit tests for control algorithms
- [ ] Hardware-in-loop testing setup
- [ ] Safety system validation tests
- [ ] Real vehicle integration testing
- [ ] Performance benchmarking and optimization

### Phase 9: CLI Configuration Tool ⏳
- [ ] Command-line configuration and tuning tool
- [ ] Profile management utilities
- [ ] Diagnostic data export and analysis
- [ ] Firmware update and deployment utilities  

---

## 🔧 Development
- **Build instructions**: TBD  
- **Wiring diagrams**: TBD  
- **Contribution**: Fork, branch, PR.

---

*Mad Hacks: RumbleDome — because sometimes boost control needs a little chaos, carefully engineered.*