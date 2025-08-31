
# RumbleDome Definitions

This document defines the acronyms, jargon, and domain-specific terminology used throughout the **RumbleDome** project.  
It ensures consistency and clarity for all contributors.

---

## Core Acronyms

- **EBC** – Electronic Boost Controller.  
  The primary purpose of RumbleDome: controlling turbo boost via solenoids, sensors, and ECU data.

- **ECU** – Engine Control Unit.  
  The car’s factory computer, which manages fuel, spark, torque, throttle, and emissions strategies.

- **CAN / CANbus** – Controller Area Network bus.  
  A vehicle-standard communication bus that allows the ECU and modules (ABS, TCM, BCM, etc.) to exchange telemetry and commands.

- **OBD-II** – On-Board Diagnostics, version 2.  
  The standardized diagnostic connector (1996+) used to access vehicle CAN messages and diagnostic trouble codes.

- **TPS** – Throttle Position Sensor.  
  Reports throttle blade angle (or pedal input) as a percentage. Used for load and torque calculations.

- **MAP** – Manifold Absolute Pressure.  
  Reports pressure inside the intake manifold. Expressed in kPa or PSI absolute. Subtracting atmospheric pressure gives gauge PSI (vacuum/boost).

- **PID** – Proportional–Integral–Derivative.  
  A control algorithm used for precise boost delivery in RumbleDome. Works on `(target_boost - actual_boost)` error to achieve exact pressure targets after torque-based system determines what boost level is needed.

- **PWM** – Pulse Width Modulation.  
  The technique used to drive solenoids and control airflow proportionally by varying duty cycle (% on-time).

- **HAL** – Hardware Abstraction Layer.  
  A layer of code isolating hardware-specific drivers (CAN, ADC, GPIO, SPI) from higher-level business logic. Makes code portable.

- **NVM** – Non-Volatile Memory.  
  Memory that retains data across resets/power cycles. EEPROM and Flash are common examples. Used for config and self-learning trims.

- **CRC** – Cyclic Redundancy Check.  
  A checksum algorithm to validate configuration data integrity.

---

## Turbo & Wastegate Terms

- **Dome Control / Full Dome** – Using compressed air on both sides of the wastegate diaphragm.  
  *Upper dome pressure pushes the gate closed; lower dome pressure pushes it open.*  
  By actively controlling both, the system can hold the wastegate in a neutral or biased position, enabling much finer control of boost.

- **Half Dome** – Only the upper dome is actively controlled with solenoid pressure; the lower dome is vented to atmosphere or connected to boost reference.  
  Simplifies plumbing but cannot achieve boost levels lower than the spring baseline.

- **Overboost** – A condition where actual boost exceeds configured safe limits.  
  RumbleDome must always fail-safe to prevent engine damage (typically by forcing wastegate open).

- **Spring Pressure / Base Boost** – The minimum boost achievable with the solenoid disabled. Determined by the wastegate spring alone.

- **Tip-in** – The sudden throttle increase event when a driver rapidly presses the accelerator.  
  Creates rapid torque demand changes that the system must handle smoothly (Phase 2+ feature).

- **Scramble Boost** – A temporary higher-boost setting, activated by a user switch/button. In RumbleDome, this maps to a configurable profile.

- **Profile** – A saved configuration defining boost pressure limits and behavior for different driving scenarios.  
  Profiles specify boost pressure curves (PSI vs RPM), not power targets, since power output varies with engine tune, turbo sizing, and other factors.

- **Valet Profile** – 0-2 psi boost limits providing near naturally-aspirated operation.  
  Designed for inexperienced drivers or valet use where full turbo power should not be available.

- **Daily Profile** – Conservative boost curve for comfortable daily driving power increase.  
  Balances improved performance with smooth, predictable delivery.

- **Aggressive Profile** – Moderate boost curve for spirited driving scenarios.  
  Higher boost limits than Daily but still within comfortable safety margins.

- **Track Profile** – Maximum safe boost curve for experienced drivers and track use.  
  Uses full available boost within engine's safety limits.

---

## Electrical Terms

- **Duty Cycle** – Percentage of time a PWM signal is "on" during a cycle.  
  Example: 0% = no solenoid activation, 100% = always energized.

- **Pull-up Resistor** – A resistor used to ensure a signal defaults to a high logic level unless actively driven low.

- **Load Switch / MOSFET** – A semiconductor used to control higher-current loads (like solenoids) with a logic-level input.

- **Buck Converter** – A DC-DC step-down voltage regulator. Used to provide stable 5V or 3.3V rails from 12V car power.

---

## Control System Terms

- **Torque-Based Control** – RumbleDome's primary control strategy using ECU torque signals rather than traditional pressure-based control.  
  Uses `(desired_torque - actual_torque)` from CAN bus to make boost decisions.

- **ECU Cooperation** – The strategy of working with the ECU's torque management rather than fighting it.  
  Targets torque levels slightly below ECU ceilings to avoid harsh interventions (spark cuts, fuel cuts).

- **Torque Ceiling** – The maximum torque value (`desired_torque`) that the ECU can safely handle.  
  RumbleDome targets ~95% of this value to maintain ECU cooperation and prevent harsh clamping.

- **Driver Demand (DD) Tables** – ECU lookup tables that map driver inputs (pedal position, RPM, conditions) to torque targets.  
  Predictable boost response keeps these tables valid and prevents hunting/surging. Traditional EBCs make these tables inaccurate due to unpredictable boost curves.

- **Final Desired Torque** – The ECU's torque target after all safety systems (traction control, ABS, clutch protection, etc.) have applied their modifications.  
  RumbleDome responds to this final value, automatically integrating with all vehicle safety systems without needing specific knowledge of each one.

- **Auto-Calibration** – RumbleDome's system for learning duty cycle mappings from user-specified boost targets.  
  User configures desired PSI values; system learns required duty cycles through safe, progressive testing.

- **Progressive Safety** – The calibration approach that starts with conservative limits and gradually increases as system proves safety response capability.  
  Begins at spring+1 psi, advances only after multi-pass validation.

- **High-Authority System** – A pneumatic system where small duty cycle changes produce large boost changes.  
  RumbleDome recognizes that 15-20% duty can potentially cause engine damage, requiring conservative control strategies.

- **Pneumatic Optimization** – System feature that recommends optimal air supply pressure for best control resolution and safety response times.  
  Balances duty cycle resolution with overboost response capability.

---

## Miscellaneous

- **Gen2 Coyote** – Ford's second-generation 5.0L Coyote V8 engine (2011–2017 Mustang GT). Target platform for initial RumbleDome builds.

- **Failsafe** – A default condition ensuring the engine is protected during system failure.  
  For RumbleDome: **0% duty cycle → full pressure to lower dome → wastegate forced open → minimal boost.**

- **Self-Learning / Trim** – The process of automatically adjusting duty cycle corrections based on observed performance over time.  
  Uses slew-rate limited adjustments stored separately from user configuration.

- **Overboost Hysteresis** – A dead band that prevents rapid cycling between overboost and normal states.  
  System triggers overboost at limit, but only clears when pressure drops below (limit - hysteresis).

- **Phase 1 vs Phase 2** – Development phases with distinct scopes:
  - **Phase 1 (RumbleDome MVP)**: Core torque-based control, auto-calibration, manual profile selection
  - **Phase 2 ("Beyond RumbleDome")**: Separates Power Level (Profile) from Delivery Style (Drive Mode)
    - **Power Level**: What boost/power you get (user-selected profile: Valet/Daily/Aggressive/Track)  
    - **Delivery Style**: How that power is delivered (drive mode aggressiveness: Normal/Sport+/Track)
    - **Safety Benefit**: Drive mode changes don't automatically increase power - prevents accidental power jumps

- **Boost vs Power Independence** – Core design principle that profiles configure boost pressure limits, not power targets.  
  Actual power output varies based on engine tune, turbo sizing, intercooling, exhaust modifications, and environmental conditions. This makes RumbleDome universal across different engine configurations.

---
