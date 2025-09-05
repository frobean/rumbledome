# RumbleDome Definitions

This document defines the acronyms, jargon, and domain-specific terminology used throughout the **RumbleDome** project. All definitions reflect the current architecture and refined design.

📖 **Related Documentation:**
- [Context.md](Context.md) - High-level design context using these terms
- [Architecture.md](Architecture.md) - System architecture implementing these concepts
- [LearnedData.md](LearnedData.md) - Learning system terminology
- [Safety.md](Safety.md) - Safety-related terminology and requirements

---

## Core System Concepts

- **RumbleDome** – A torque-aware electronic boost controller that cooperates with modern ECU torque management systems rather than fighting them. Implements intelligent torque-following control using a single aggression parameter.

- **Torque Request Amplifier** – RumbleDome's fundamental approach: amplifying ECU torque requests rather than following predetermined boost curves. Works in harmony with ECU logic by helping it achieve torque goals faster or slower based on user preference.

- **3-Tier Priority Hierarchy** – RumbleDome's organizing control philosophy:
  - **Priority 1**: "Don't Kill My Car" (overboost protection, always overrides)
  - **Priority 2**: Performance (forceful max boost targeting)  
  - **Priority 3**: Comfort (smooth gentle operation)
  - **Aggression Setting**: Determines whether Priority 2 or 3 leads

- **Torque-Following Control** – Primary control strategy using ECU torque signals rather than traditional pressure-based control. Uses `(desired_torque - actual_torque)` from CAN bus to determine boost assistance.

- **ECU Cooperation** – The strategy of working with the ECU's torque management rather than fighting it. Maintains ECU driver demand table validity by providing predictable boost response.

---

## User Interface & Control

- **Aggression** – Single user control parameter (0.0-1.0) that scales all torque-following response characteristics:
  - **0.0 (Puppy Dog)**: System OFF, naturally aspirated feel
  - **0.5 (Daily Driver)**: Balanced torque assistance  
  - **1.0 (Brimstone)**: Maximum ECU torque request assistance

- **Scramble Button** – Momentary override button providing instant 100% aggression regardless of current setting. Returns to normal aggression when released.

- **6-Parameter Configuration** – Simplified user setup requiring only:
  - `aggression` (0.0-1.0): Torque-following aggressiveness
  - `spring_pressure` (PSI): Wastegate spring pressure
  - `max_boost_psi` (PSI): Operational target ceiling for boost pressure
  - `overboost_limit` (PSI): Hard safety fault threshold  
  - `scramble_enabled` (bool): Enable scramble button functionality
  - `cold_engine_protection` (bool): Enable temperature-based aggression limiting

- **User Responsibility Model** – Design philosophy where users set safety limits and the system provides intelligent guidance, never enforcement beyond overboost protection.

---

## Learning System

- **Learned Data** – System knowledge acquired through operation, stored separately from user configuration. Four categories:
  - **Duty Cycle Calibration**: Core boost control learning
  - **Environmental Compensation**: Temperature/altitude/pressure adaptation
  - **Sensor Fusion Cross-Calibration**: CAN MAP vs boost gauge offset learning
  - **Safety Response Parameters**: Optimal overboost recovery characteristics

- **STFT/LTFT Learning** – Fast and slow learning adaptation similar to ECU fuel trims:
  - **STFT (Short-Term)**: Fast adaptation to immediate conditions (5% per cycle)
  - **LTFT (Long-Term)**: Slow adaptation to long-term trends (0.1% per cycle)

- **Progressive Calibration** – Learning approach starting at conservative limits (spring+1 PSI), gradually expanding as system proves safety response capability.

- **Multi-Pass Validation** – Requirement for consistent results across multiple calibration runs before accepting learned values.

- **Learning Confidence** – Metric (0.0-1.0) tracking quality and reliability of learned parameters.

---

## Storage Architecture

- **SD Card Storage** – Primary storage system using FAT32 filesystem for all configuration and learned data. Provides unlimited capacity and excellent development workflow.

- **Debounced Persistence** – Write optimization strategy delaying storage writes 5-10 seconds after changes to minimize SD card wear during adjustment sessions.

- **Atomic Operations** – Crash-safe file operations using temporary files and atomic renames to prevent corruption from power loss.

- **File Structure** – Organized storage layout:
  ```
  /config/user_config.json      (6-parameter user configuration)
  /learned/[category].json       (learned data by category)
  /backups/[timestamp]/          (automatic rolling backups)
  /logs/[date]/                  (diagnostic and safety logs)
  ```

- **Separation Strategy** – User configuration and learned data stored in separate files, enabling independent reset and backup operations.

---

## Safety Systems

- **Max Boost Limit (PSI)** – Target operational ceiling for boost pressure during normal operation. System should avoid exceeding this level but brief overshoots are not dangerous to the engine. Part of normal configuration parameters.

- **Overboost Limit (PSI)** – Hard safety ceiling beyond which engine damage may occur. Any pressure above this level triggers immediate emergency response (0% duty cycle). Must be set higher than Max Boost Limit.

- **Safety Margin** – The gap between Max Boost Limit and Overboost Limit, providing protection against dangerous pressure levels while allowing operational flexibility.

- **Overboost vs Max Boost** – Critical distinction:
  - **Max Boost**: Operational target limit - avoid exceeding but brief spikes acceptable
  - **Overboost**: Hard safety fault threshold - immediate emergency response required

- **Defense in Depth** – Multiple independent safety layers:
  - **Electronic**: Software monitoring and response
  - **Pneumatic**: Physical system failsafe design
  - **Mechanical**: Spring-loaded wastegate backup

- **Fail-Safe Operation** – System design ensuring any failure results in safest state: 0% duty → lower dome pressurized → wastegate open → minimal boost.

- **Progressive Overboost Limits** – Safety approach increasing overboost limits only as system proves adequate safety response capability.

- **Learned Safety Parameters** – System learns optimal overboost recovery characteristics (hysteresis, timing) rather than using fixed user-configured values.

---

## Hardware & Sensors

- **Full-Dome Control** – Pneumatic system using compressed air on both sides of wastegate diaphragm for control both above and below spring pressure.

- **4-Sensor Configuration** – Complete pressure monitoring system:
  - **Manifold Pressure**: Primary boost measurement and safety monitoring
  - **Dome Input**: Air supply pressure for feedforward compensation
  - **Upper Dome**: Wastegate closing force monitoring
  - **Lower Dome**: Wastegate opening force and system health

- **Sensor Fusion** – Automatic cross-calibration between CAN MAP sensor (vacuum range) and boost gauge (positive pressure range) for seamless operation across full pressure spectrum.

- **MAC Solenoid** – 4-port solenoid controlling dome pressure distribution:
  - **0% duty**: Lower dome pressurized → wastegate forced OPEN
  - **100% duty**: Upper dome pressurized → wastegate forced CLOSED

---

## Control Algorithms

- **PID Controller** – Proportional-Integral-Derivative control for precise boost delivery using `(target_boost - actual_boost)` error after torque-based system determines boost requirement.

- **PWM Synchronization** – Advanced timing coordination preventing phase noise and jitter in pneumatic control through beat frequency elimination.

- **Environmental Compensation** – Learned correction factors adapting to temperature, altitude, and supply pressure variations.

- **Slew Rate Limiting** – Control of maximum duty cycle change rate to prevent unsafe rapid responses.

- **High-Authority System Recognition** – Acknowledgment that small duty cycle changes can produce large boost changes, requiring conservative control strategies.

---

## Communication & Protocols

- **CAN Bus Integration** – Real-time torque data acquisition from ECU via Controller Area Network:
  - **Desired Torque**: ECU's target torque output
  - **Actual Torque**: ECU's measured/estimated current torque
  - **RPM**: Engine speed for calibration context

- **JSON/CLI Protocol** – Human-readable communication protocol for configuration, calibration control, and system monitoring.

- **Ford Gen2 Coyote** – Initial target platform (2011-2017 Mustang GT 5.0L) for CAN signal specifications.

---

## Acronyms & Technical Terms

- **EBC** – Electronic Boost Controller
- **ECU** – Engine Control Unit  
- **CAN** – Controller Area Network
- **OBD-II** – On-Board Diagnostics, version 2
- **TPS** – Throttle Position Sensor
- **MAP** – Manifold Absolute Pressure
- **PID** – Proportional-Integral-Derivative (control algorithm)
- **PWM** – Pulse Width Modulation
- **HAL** – Hardware Abstraction Layer
- **PSI** – Pounds per Square Inch (pressure measurement)
- **RPM** – Revolutions Per Minute
- **ADC** – Analog-to-Digital Converter
- **GPIO** – General Purpose Input/Output
- **SPI** – Serial Peripheral Interface
- **FAT32** – File Allocation Table 32-bit (filesystem)

---

## Development Terminology

- **Teensy 4.1** – Target microcontroller platform (NXP i.MX RT1062, 600 MHz ARM Cortex-M7)

- **Hardware Abstraction Layer (HAL)** – Code layer isolating hardware-specific drivers from business logic, enabling multi-platform support.

- **Mock Implementations** – Software simulations of hardware interfaces for desktop testing without physical hardware.

- **Desktop Simulator** – Complete system simulation enabling testing and validation of control algorithms on development machines.

- **Unplanned Thermal Events** – Developer euphemism for accidentally releasing magic smoke from electronic components during prototyping.

---

## Legacy Terms (Deprecated)

**Note**: These terms appear in older documentation but have been superseded:

- ~~**Control Knob**~~ → **Aggression** (clearer terminology)
- ~~**EEPROM Storage**~~ → **SD Card Storage** (better capacity and reliability)
- ~~**Phase 2 Drive Modes**~~ → **Simplified to aggression-only control**
- ~~**Profile System**~~ → **Single aggression parameter**
- ~~**User-Configured Hysteresis**~~ → **Learned safety parameters**

---

*This definitions document reflects the current RumbleDome architecture. All terms are consistent with the latest design specifications and implementation requirements.*