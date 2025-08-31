# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**RumbleDome** is a custom full-dome electronic boost controller (EBC) for turbo systems, built around the Teensy 4.1 microcontroller and written in Rust. This is a collaborative project between a human architect and AI assistant, focused on creating a safety-first, closed-loop, self-learning boost controller that prioritizes driveability over pure drag-strip performance.

## Key Architecture Principles

- **Safety First**: 0% duty cycle = no boost (failsafe operation)
- **Modular Design**: Core logic separated from hardware abstraction layers
- **Self-Learning**: STFT/LTFT-style trims for adaptive control
- **Rust-Based**: No unsafe code where possible, verbose and testable design
- **Dual Environment**: Code must run on both desktop (simulation) and MCU (firmware)

## Workspace Structure (Planned)

```
rumbledome-hal/   # Hardware abstraction traits + implementations
rumbledome-core/  # State machine, PI control, learning algorithms, safety logic
rumbledome-iface/ # JSON/CLI protocol definitions
rumbledome-fw/    # Teensy 4.1 firmware implementation
rumbledome-sim/   # Desktop simulator for testing
```

## Critical Safety Requirements (Never Drop)

From `docs/MustNotDrop.md`:
- 0% PWM duty = full pressure to lower dome = wastegates open = no boost
- All fault conditions MUST result in 0% duty (boost cut)
- Default behavior on any error is always fail-safe (no boost)
- Overboost protection is mandatory and configurable
- Self-learning trims must be bounded and rate-limited

## Key Documentation Files

**ALWAYS read these files before making code changes:**

1. `docs/Context.md` - High-level design context and goals
2. `docs/Requirements.md` - Functional and performance requirements
3. `docs/Safety.md` - Non-negotiable safety requirements (CRITICAL)
4. `docs/Architecture.md` - System design and component architecture
5. `docs/Protocols.md` - JSON/CLI communication protocol specifications
6. `docs/Hardware.md` - Hardware abstraction layer specifications  
7. `docs/Implementation.md` - Code structure and development workflow

## Development Workflow

1. **Always anchor to README.md** at session start to regain context
2. **Check .rd-manifest.json** for latest repository structure (auto-generated)
3. **Never drop requirements** from specification documents
4. **Work module-by-module** respecting API boundaries
5. **Document assumptions** with `⚠ SPECULATIVE` markers
6. **Preserve code style**: verbose names, extensive comments, self-documenting
7. **Test everything**: unit tests must work without hardware
8. **Fail safe**: all error paths must default to zero boost

## Hardware Context

- **Target MCU**: Teensy 4.1 (Cortex-M7, 600 MHz)
- **Solenoid**: MAC 4-port valve at ~30 Hz PWM
- **Sensors**: 3x pressure sensors (0-30 psi, 0.5-4.5V ratiometric)
- **Display**: ST7735R TFT LCD (128×160)
- **CAN Bus**: Ford Gen2 Coyote integration via SN65HVD230 transceiver
- **Control**: Closed-loop PI with torque-awareness from CAN bus

## Configuration Philosophy

- All user configuration in **pressure units (PSI/kPa)**, never raw duty cycles
- Multiple profiles: Valet, Daily, Aggressive, Track, Scramble
- JSON-based configuration protocol over serial/Bluetooth
- Self-learning data stored separately from user configuration
- EEPROM wear-aware storage with batching and thresholds

## AI Collaboration Rules

- Mark speculative CAN implementation with `⚠ SPECULATIVE` comments
- Surface specification gaps rather than guessing
- Respect HAL abstractions - hardware-specific code only in appropriate layers  
- Maintain testability - core logic must work with mocked hardware
- Follow Rust best practices but prioritize clarity over cleverness
- Never assume missing context - ask for clarification when requirements are unclear

## Build Commands

**Note**: No build system is currently implemented. Rust workspace with Cargo will be used once source code development begins.

Expected commands (TBD):
- `cargo build` - Build all workspace crates
- `cargo test` - Run unit tests with mocked hardware
- `cargo run --bin rumbledome-sim` - Run desktop simulator
- Embedded build commands TBD based on Teensy 4.1 toolchain setup