# RumbleDome Design Specification

**Mad Hacks: RumbleDome** is a custom electronic boost controller (EBC) built around a Teensy 4.1 for a full-dome control turbo system. This document defines the system architecture, design philosophy, and intended implementation details. It serves as the reference for developers, testers, and collaborators.

---

## 1. Purpose

Modern off-the-shelf EBCs are optimized for drag racing scenarios, where fast and predictable boost onset down a strip is the primary goal. They lack flexibility for daily driving, spool management, and integration with modern ECU torque modeling.

RumbleDome fills this gap by providing:
- **Full-dome boost control** for maximum flexibility and safety.
- **Torque-aware control loop** tied directly into the ECU’s CAN bus.
- **Self-learning capabilities** to adapt to sensor drift and environmental factors.
- **Multiple profiles** (valet, daily, aggressive, track).
- **Failsafe behavior** that always defaults to safe boost levels.

---

## 2. Physical Architecture

- **Microcontroller**: Teensy 4.1 (Cortex-M7, 600 MHz).
- **Solenoid**: MAC 4-port valve.  
  - Normally Open (NO) → lower dome (failsafe → zero boost).  
  - Normally Closed (NC) → upper dome.  
  - PWM frequency: 30 Hz.  
  - Duty cycle:  
    - 0% → full pressure to lower dome (no boost).  
    - 100% → full pressure to upper dome (maximum boost).  

- **Compressed air supply**: Shared with air suspension. Regulated from ~150 psi down to 10–15 psi for dome input.

- **Pressure sensors** (0–30 psi, ratiometric 0.5–4.5 V):  
  1. Between regulator and solenoid input.  
  2. Upper dome (near wastegate).  
  3. Intake manifold (post-throttle).  

- **Display**: ST7735R TFT LCD (1.8”, 128×160).  
- **CAN bus**: via SN65HVD230 transceiver, attached at OBD2 port.  
- **Enclosure**: Gauge pod (60 mm, 3D-printed prototype).  
- **Controls**:  
  - Momentary toggle switch (profile switching).  
  - Optional “scramble button” to instantly switch to configured “track” profile.  
- **Connectivity**: Bluetooth shield (TBD) for wireless configuration.  
- **Power**: Initially via OBD2, later tied to ignition-switched 12 V line for reliability.

---

## 3. Inputs & Outputs

### Inputs
- Pressure sensors (analog voltage → PSI).  
- CAN bus telemetry:  
  - RPM / tach.  
  - MAP (ECU).  
  - Desired (commanded) torque.  
  - Actual (estimated) torque.  
  - Torque source (optional).  
- Profile select switch.  
- Scramble mode button.  
- Configuration updates (Bluetooth / console).  

### Outputs
- Solenoid PWM duty cycle.  
- TFT screen (boost gauge, profile, warnings, faults).  
- Debug/console logs (serial or Bluetooth).  

---

## 4. Operating Modes

- **Valet Mode**: Forces spring-only boost (effectively disables turbo).  
- **Daily Mode**: Smooth, conservative response.  
- **Aggressive Mode**: Optimized for spirited street driving.  
- **Track Mode**: Maximum boost control, aggressive PI gains, rapid spool.  
- **Scramble Mode**: Override to configured “track” profile until button release.  

---

## 5. Control Logic

### 5.1 Closed Loop PID
- Primary loop uses **torque error**: `(desired_torque – actual_torque)` from CAN bus.  
- Boost target bias applied from RPM bins (user-defined map).  
- PID computes solenoid duty cycle.  
- Feed-forward term from dome regulator pressure helps minimize lag.

### 5.2 Self-Learning
- Adapts duty cycle trims over time.  
- Similar to ECU short-term and long-term fuel trims.  
- Stored in NVM separate from user configuration.  
- Protects against drift (temperature, dome supply variance).  

### 5.3 Overboost Protection
- Hard limit configured in PSI.  
- If exceeded:  
  - Solenoid drives NO path → lower dome pressurized.  
  - Result: wastegate forced open → boost drops to spring pressure.  
- Configurable response: full cut vs pulse-dump then recovery.  

---

## 6. Fault Handling

- All faults default to **zero boost** by setting duty cycle to 0%.  
- Fault types:  
  - Sensor missing or invalid (implausible readings).  
  - CAN bus disconnected.  
  - Overboost beyond configured safety threshold.  
- Faults displayed on TFT + logged over serial/Bluetooth.  
- Initial release: all faults treated as **hard faults** (boost cut).  
- Future: differentiate soft vs hard faults.

---

## 7. User Interface

### Display
- Analog-style gauge (needle).  
- Range: max vacuum to slightly beyond overboost limit.  
- Color zones:  
  - Green = within configured boost range.  
  - Yellow = between max boost and overboost.  
  - Red = overboost.  
- Target boost shown as tick mark.  
- Overboost condition: gauge background flashes red.

### Console/Bluetooth
- Commands for configuration, profile switching, debug info.  
- Console command to **reset trims**.  
- JSON-based config exchange (PSI units, not duty cycles).  

---

## 8. Data Persistence

- **Configuration data** (profiles, limits): stored in NVM, user-editable.  
- **Trim data** (self-learning): stored in separate NVM space, auto-managed.  
- EEPROM wear minimized via:  
  - Batching writes.  
  - Threshold-based updates (only write when delta > threshold).  

---

## 9. Software Architecture

- Language: **Rust** (safety, modern tooling).  
- Layered modules:  
  - `hal/` → hardware abstraction (CAN, sensors, PWM, display, storage).  
  - `control/` → PID, trims, safety logic.  
  - `profiles/` → multi-profile management.  
  - `ui/` → TFT rendering + console/Bluetooth.  
  - `tests/` → unit + integration tests with fake CAN/sensor injection.  

- Code style:  
  - Verbose, self-documenting.  
  - Clear APIs, no hidden magic.  
  - Heavy inline comments for math and algorithms.  

---

## 10. Testing Strategy

- **Unit tests**:  
  - PID loop, duty cycle calculations, trim updates.  
  - Pure math/logic, no hardware required.  

- **Integration tests**:  
  - Fake CAN + fake sensor inputs → system reactions.  
  - Overboost, sensor fault, CAN dropouts.  

- **Bench testing**:  
  - Simulated solenoid + dome plumbing on test rig.  
  - Verify response matches simulated torque/load curves.  

---

## 11. Future Extensions

- Soft vs hard faults classification.  
- Configurable per-profile PID gains.  
- Bluetooth mobile app for configuration.  
- Alternate CAN HAL for non-Ford ECUs.  
- Wider sensor support (100 psi range).  

---

## 12. Summary

RumbleDome is a torque-aware, self-learning, full-dome boost controller.  
It prioritizes **safety first**, **clarity in configuration**, and **future extensibility**.  
This design spec defines the structure needed to move forward into API contracts (`Interfaces.md`) and implementation.

---
