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

--

## 📖 Project Overview
- **Firmware**: Rust (no unsafe where possible), modular design, Teensy 4.1 target.  
- **Hardware**:
  - Teensy 4.1 MCU  
  - 4-port MAC solenoid (30 Hz, 0% duty = lower dome pressure = failsafe to no boost)  
  - 3 pressure sensors (0–30 psi, 0.5–4.5 V output)  
  - ST7735R TFT LCD (1.8" 128×160)  
  - CAN bus transceiver (SN65HVD230 or similar)  
  - OBD2 adapter for CAN connection  
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

---

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

### Phase 0: Foundation
- [x] Context defined  
- [x] Design spec fleshed out  
- [ ] Interfaces defined (partial - needs CAN reverse engineering)

### Phase 1: Core Infrastructure  
- [ ] Rust workspace scaffolding (Cargo.toml, crate structure)
- [ ] HAL trait definitions and mock implementations
- [ ] Basic unit test framework setup
- [ ] Core data structures (SystemConfig, LearnedData, etc.)
- [ ] State machine implementation
- [ ] Error handling and fault management system

### Phase 2: Hardware Integration
- [ ] Teensy 4.1 HAL implementation (PWM, ADC, GPIO)
- [ ] Pressure sensor calibration and reading
- [ ] CAN bus integration (hardware + protocol)
- [ ] Display driver (ST7735R TFT)
- [ ] EEPROM/Flash storage with wear leveling
- [ ] Watchdog and safety monitoring

### Phase 3: Control Logic
- [ ] Profile management system
- [ ] 3-level control loop implementation
- [ ] PID controller with tuning
- [ ] Safety override and slew limiting
- [ ] Environmental compensation algorithms
- [ ] Real-time system integration (100Hz loop)

### Phase 4: Learning Systems
- [ ] Auto-calibration state machine
- [ ] Progressive safety limit expansion
- [ ] Duty cycle learning and storage
- [ ] Environmental factor compensation learning
- [ ] Confidence tracking and validation

### Phase 5: User Interface
- [ ] JSON protocol implementation
- [ ] CLI configuration tool
- [ ] Desktop simulator with test scenarios
- [ ] Real-time display updates and gauge rendering
- [ ] Diagnostic and telemetry reporting

### Phase 6: Integration & Testing
- [ ] Hardware-in-loop testing setup
- [ ] Safety system validation tests
- [ ] Performance benchmarking
- [ ] Vehicle integration testing
- [ ] Documentation and user guides  

---

## 🔧 Development
- **Build instructions**: TBD  
- **Wiring diagrams**: TBD  
- **Contribution**: Fork, branch, PR.

---

*Mad Hacks: RumbleDome — because sometimes boost control needs a little chaos, carefully engineered.*