# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**RumbleDome** is a torque-based electronic boost controller (EBC) that cooperates with modern ECU torque management systems rather than fighting them. Built around the Teensy 4.1 microcontroller and written in Rust, it prioritizes safety, predictable driveability, and automatic integration with vehicle safety systems over pure drag-strip performance.

**License**: CC BY-NC 4.0 (see LICENSE.md for full terms)

## Core Innovation

**ECU Cooperation Strategy**: Unlike traditional EBCs that operate independently, RumbleDome monitors ECU torque requests and delivery, then modulates boost to help the ECU achieve its torque targets. This automatically integrates with traction control, ABS, clutch protection, and other safety systems without requiring specific knowledge of each system.

## Key Architecture Principles

- **Safety First**: 0% duty cycle = wastegate forced open = failsafe to no boost
- **Torque-Based Control**: Uses ECU torque signals rather than traditional pressure-only control
- **Boost-Based Configuration**: User configures boost pressure limits, not power targets (engine-agnostic)
- **Progressive Auto-Calibration**: Conservative safety limits that gradually expand as system proves response capability
- **Modular Design**: HAL abstraction enables desktop testing and multi-platform support
- **Predictable Response**: Maintains consistent boost delivery to keep ECU driver demand tables valid

## Workspace Structure (Current)

```
crates/
├── rumbledome-hal/      # Hardware abstraction layer with traits and mock implementation
├── rumbledome-core/     # Core control logic (hardware-independent)
├── rumbledome-protocol/ # JSON/CLI protocol definitions  
├── rumbledome-fw/       # Teensy 4.1 firmware binary (skeleton)
├── rumbledome-sim/      # Desktop simulator (skeleton)
└── rumbledome-cli/      # Configuration tool (skeleton)
```

## Critical Safety Requirements (Never Drop)

- **Failsafe Design**: 0% PWM duty = full pressure to lower dome = wastegates forced open = no boost
- **Fault Response**: ALL fault conditions MUST result in 0% duty cycle (boost cut)
- **Safety Overrides**: Overboost protection, progressive limits, bounded learning
- **High-Authority Recognition**: Small duty cycle changes can produce large boost changes - conservative control required

## 3-Level Control Hierarchy

**Level 1: Torque-Based Boost Target Adjustment**
- Monitor ECU torque gap (desired vs actual)
- Modulate boost target to help ECU achieve torque goals
- Target ~95% of ECU torque ceiling to prevent harsh interventions

**Level 2: Precise Boost Delivery (PID + Learned)**  
- Use learned duty cycle baseline from auto-calibration
- Apply PID correction for precise boost delivery
- Environmental compensation (temperature, altitude, supply pressure)

**Level 3: Safety and Output**
- Apply safety overrides and progressive limits
- Slew rate limiting to prevent unsafe responses
- Update solenoid PWM output

## Phase Roadmap

**Phase 1 (RumbleDome MVP)** ✅ COMPLETE
- Core torque-based control with auto-calibration
- Manual profile selection (Valet/Daily/Aggressive/Track) 
- Hardware abstraction layer and mock implementation

**Phase 2 ("Beyond RumbleDome")** 🚧 PLANNED
- **Power Level (Control Knob)**: What boost/power you get (single knob: 0-100%)
- **Delivery Style (Drive Mode)**: How that power is delivered (Normal/Sport+/Track aggressiveness)
- **Safety Benefit**: Drive mode changes don't automatically increase power

## Key Documentation Files

**ALWAYS read these files before making code changes:**

1. `docs/Context.md` - Project narrative and boost-based design philosophy
2. `docs/Requirements.md` - Functional requirements emphasizing torque-based control  
3. `docs/Safety.md` - Non-negotiable safety requirements (CRITICAL)
4. `docs/Architecture.md` - System design with 3-level control hierarchy
5. `docs/Implementation.md` - Code structure and build process
6. `docs/Definitions.md` - Domain terminology and Phase 2 concepts
7. `docs/Hardware.md` - HAL specifications and sensor requirements
8. `docs/Protocols.md` - JSON/CLI communication protocol

## Hardware Context

- **Target MCU**: Teensy 4.1 (Cortex-M7, 600 MHz)
- **Solenoid**: 4-port MAC valve at 30 Hz PWM
- **Full-Dome Control**: Both upper and lower dome pressures actively controlled
- **Sensors**: 3x pressure sensors (dome input, upper dome, manifold pressure)
- **Display**: ST7735R TFT LCD 1.8" (128×160) in 60mm gauge pod
- **CAN Bus**: Ford Gen2 Coyote integration (initial target platform)
- **Air Supply**: Compressed air regulated to optimal input pressure

## Configuration Philosophy

**Control Knob-Based (Not Power-Based)**:
- All user configuration in **pressure units (PSI/kPa)**, never raw duty cycles or power targets
- Same boost pressure produces different power depending on engine tune, turbo sizing, environmental conditions
- Engine-agnostic approach - works with any engine setup within boost pressure constraints
- User responsibility to determine appropriate boost limits for their specific engine

**Control Knob Strategy**:
- **0% (Valet)**: Minimal torque amplification (near naturally-aspirated for inexperienced drivers)
- **~30% (Daily)**: Conservative torque amplification for comfortable daily driving
- **~70% (Sport)**: Moderate torque amplification for spirited driving  
- **100% (Track)**: Maximum torque amplification for experienced drivers/track use

## Current Implementation Status

✅ **Phase 1 Complete**:
- Workspace scaffolding and crate structure
- Complete HAL trait definitions with mock implementation
- Core data structures (SystemConfig, SystemState, error handling)
- State machine implementation with calibration states
- JSON protocol message definitions
- Comprehensive configuration system with 4 default profiles
- Unit test framework setup

🚧 **Next Priorities**:
- Teensy 4.1 HAL implementation (Phase 2: Hardware Integration)
- 3-level control loop implementation (Phase 3: Control Logic)  
- Auto-calibration algorithms (Phase 4: Learning Systems)

## Development Workflow

1. **Read relevant docs** before making changes to understand context
2. **Respect safety requirements** - never compromise failsafe behavior
3. **Work module-by-module** respecting HAL abstractions and API boundaries
4. **Test everything** - unit tests must work with mock hardware
5. **Document assumptions** with `⚠ SPECULATIVE` markers for unverified details
6. **Preserve code style**: verbose names, extensive comments, self-documenting code
7. **Fail safe**: all error paths must default to zero boost

## AI Collaboration Rules

- **Never drop requirements** from specification documents
- **Surface gaps** rather than guessing when requirements are unclear
- **Respect layering** - hardware-specific code only in HAL implementations
- **Maintain testability** - core logic must work with mocked hardware
- **Follow safety constraints** - overboost protection and progressive limits are non-negotiable
- **Mark speculative areas** with `⚠ SPECULATIVE` for human verification
- **Preserve design philosophy** - boost-based configuration, torque cooperation, ECU integration

## Build Commands

**Current Status**: Phase 1 workspace is buildable

```bash
# Build all crates
cargo build --workspace

# Run tests with mock hardware
cargo test --workspace

# Build specific crates
cargo build -p rumbledome-core
cargo build -p rumbledome-hal --features mock

# Check embedded target builds (when ready)
cargo check --target thumbv7em-none-eabihf -p rumbledome-fw
```

**Future Commands**:
```bash
# Desktop simulator (when implemented)
cargo run --bin rumbledome-sim

# CLI configuration tool (when implemented)  
cargo run --bin rumbledome-cli

# Flash firmware to Teensy 4.1 (when implemented)
teensy_loader_cli --mcu=TEENSY41 -w target/thumbv7em-none-eabihf/release/rumbledome-fw.hex
```

## Critical Implementation Notes

- **CAN Signal Mapping**: Current CAN signal definitions in config are `⚠ SPECULATIVE` - require real vehicle reverse engineering
- **Sensor Calibration**: Pressure sensor voltage-to-PSI conversion uses estimated curves - verify with actual hardware
- **PID Tuning**: Default PID parameters in profiles are starting points - require real-world tuning
- **Safety Limits**: Progressive overboost limits start conservative (spring + 1 PSI) and expand only after proven safety response

## Testing Strategy

- **Mock HAL**: Complete hardware simulation for desktop testing
- **Unit Tests**: All control logic testable without hardware dependencies
- **Integration Tests**: Full calibration sequences with simulated sensor data
- **Safety Tests**: Verify overboost response, fault handling, failsafe behavior
- **Hardware-in-Loop**: Real hardware validation before vehicle integration