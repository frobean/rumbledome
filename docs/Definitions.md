
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
  A control algorithm used to correct error between target and measured values. In this project, PID regulates boost.

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
  Boost controllers may momentarily add duty to reduce lag.

- **Scramble Boost** – A temporary higher-boost setting, activated by a user switch/button. In RumbleDome, this maps to a configurable profile.

- **Profile** – A saved configuration set defining boost targets and behavior (e.g., valet/kill mode, daily, aggressive, track).

---

## Electrical Terms

- **Duty Cycle** – Percentage of time a PWM signal is "on" during a cycle.  
  Example: 0% = no solenoid activation, 100% = always energized.

- **Pull-up Resistor** – A resistor used to ensure a signal defaults to a high logic level unless actively driven low.

- **Load Switch / MOSFET** – A semiconductor used to control higher-current loads (like solenoids) with a logic-level input.

- **Buck Converter** – A DC-DC step-down voltage regulator. Used to provide stable 5V or 3.3V rails from 12V car power.

---

## Miscellaneous

- **Gen2 Coyote** – Ford’s second-generation 5.0L Coyote V8 engine (2015–2017 Mustang GT). Target platform for initial RumbleDome builds.

- **Failsafe** – A default condition ensuring the engine is protected during system failure.  
  For RumbleDome: **0% duty cycle → full pressure to lower dome → wastegate forced open → near-zero boost.**

- **Self-Learning / Trim** – The process of automatically adjusting duty cycle corrections based on observed boost error over time, and saving these values to NVM.

---
