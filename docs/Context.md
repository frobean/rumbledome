# RumbleDome — Context & Product Spec (Baseline)

Mad Hacks: **RumbleDome** is a custom **full-dome** electronic boost controller (EBC) for turbo builds that prioritizes
**OEM-style driveability** and **safety** over drag-strip peak performance. Unlike traditional EBCs that are configured
by duty cycle, RumbleDome is configured by desired boost targets and learns the appropriate duty cycles through
auto-calibration. It implements torque-aware, predictive control using a Teensy 4.1 MCU, a 4‑port "Mac-style" solenoid,
three pressure sensors, TFT display, and CAN integration with the ECU's torque management system.

## Physical Layout

- **MCU:** Teensy 4.1
- **Solenoid:** 4‑port Mac‑style, ~30 Hz drive.  
  - 0% duty → lower dome gets full input pressure → **wastegate forced OPEN**  
  - 100% duty → upper dome gets full input pressure → wastegate forced CLOSED
- **Air Supply:** onboard compressed air; regulated from ~150 psi tank to **configurable pressure** at solenoid input.
  - System calculates optimal input pressure based on configured boost limits and wastegate spring pressure
  - Input pressure optimized for good duty cycle resolution across the engine's safe boost range
- **Sensors (3):**
  1) Regulator→solenoid input (dome supply pressure)  
  2) Upper dome line (near wastegate)  
  3) Intake manifold (post-throttle) for boost reference
- **Transducers:** 1/8″ NPT, 0–30 psi gauge, 0.5–4.5 V out at 5 V supply.  
  Estimated scale: `PSI = ((Vout − 0.5) / 4.0) * 30`.
- **Display:** ST7735R TFT 1.8″ 128×160 in a 60 mm gauge pod.
- **CAN:** via OBD‑II adapter; transceiver SN65HVD230 (3.3 V) or equivalent.
- **Inputs:** at least one momentary switch (profile change); optional scramble button.
- **Bluetooth:** TBD module for config/telemetry (future).

## Operating Concept (Phase 1: RumbleDome MVP)

- **ECU torque production assistant**: Helps ECU achieve torque targets by monitoring torque gap and adjusting boost accordingly
- **Automatic safety system integration**: Works seamlessly with traction control, ABS, clutch protection by responding to final torque values
- **Auto-calibration**: User configures desired boost targets; system learns required duty cycles through progressive safety testing
- **Pneumatic optimization**: Recommends optimal input air pressure for control resolution and safety response
- **Predictable boost delivery**: Maintains consistent response characteristics to keep ECU driver demand tables valid
- **Inputs:**
  - 3 pressure sensors; CAN signals (RPM, MAP, **desired torque**, **actual torque**)
  - User profiles specifying **target boost vs RPM** curves (psi/kPa)
  - **Configurable wastegate spring pressure** and engine-specific safety limits
- **Outputs:**
  - PWM duty to solenoid (30 Hz nominal) with learned duty cycle mappings
  - TFT display: boost gauge, targets, pneumatic system health, calibration status
  - Console/BT: configuration, calibration mode, system recommendations

## Future Roadmap (Phase 2+: "Beyond RumbleDome")
- **Drive mode integration**: Adaptive aggressiveness based on Normal/Sport/Track modes from CAN
- **Throttle position predictive control**: Anticipatory boost management using TPS signals
- **Multi-variable environmental compensation**: Advanced algorithms for temperature, altitude, humidity
- **Driving pattern recognition**: Learning driver behavior for personalized response tuning

## Design Philosophy

### Boost-Based Configuration (Not Power-Based)
- **Configure in boost pressure (PSI/kPa)**, never raw duty cycles or power targets
- **Engine-agnostic approach**: Same boost pressure produces different power depending on:
  - Engine tune (timing, fuel, cam timing)
  - Turbo sizing and efficiency
  - Intercooling, exhaust, internal modifications
  - Environmental conditions (altitude, temperature, fuel quality)
- **User responsibility**: Determine appropriate boost limits for their specific engine setup
- **Universal compatibility**: Works with any engine/tune combination within boost pressure constraints

### Profile Strategy  
- **Valet (0-2 psi)**: Near naturally-aspirated operation for inexperienced drivers or valet use
- **Daily**: Conservative boost curve for comfortable daily driving power increase
- **Aggressive**: Moderate boost curve for spirited driving scenarios  
- **Track**: Maximum safe boost curve for experienced drivers and track use
- **Live switching**: Safe profile changes during operation
- **Explicit selection**: User must intentionally select power level (no automatic power increases)

### Safety-First Operation
- **Failure modes never overboost**: Default bias is always toward no-boost operation
- **Self-learning** stored separately from user configuration in non-volatile memory
- **Overboost protection**: Immediate cut with configurable recovery policies
- **Hard fault approach**: All faults result in boost cut until fault domain is better understood

## Self-Learning

Similar to STFT/LTFT in ECUs: maintain fast and slow trims on the duty→boost mapping, persist with wear‑aware cadence.
Trims are **rate-limited** and bounded, and are **shared across profiles** (physics are common) while keeping profile targets distinct.
Provide a **console command** to reset trims.

## Power & Safety

- **0% duty ⇒ full pressure to lower dome ⇒ wastegates biased open ⇒ (near) zero boost.**  
- On **any fault** (sensor invalid, CAN timeout, storage failure): cut to 0% and display the fault.
- Default power in vehicle installs is **ignition-switched** to avoid battery drain; during prototyping on OBD‑II, manually unplug or power-gate.
