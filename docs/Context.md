# RumbleDome — Context & Product Spec (Baseline)

Mad Hacks: **RumbleDome** is a custom **full-dome** electronic boost controller (EBC) for turbo builds that favors
**driveability** and **safety** over pure drag-strip use. It implements closed-loop, self-learning control using
a Teensy 4.1 MCU, a 4‑port “Mac-style” solenoid, and three pressure sensors, with a TFT display and CAN integration.

## Physical Layout

- **MCU:** Teensy 4.1
- **Solenoid:** 4‑port Mac‑style, ~30 Hz drive.  
  - NO → lower dome (failsafe to **no boost** @ 0% duty)  
  - NC → upper dome
- **Air Supply:** onboard compressed air; regulated from ~150 psi tank to **~10–15 psi** at solenoid IN.
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

## Operating Concept

- **Closed-loop** with **self-learning trims** (temperature, dome feed drift, etc.).
- **Inputs:**
  - 3 pressures above; CAN signals (RPM, MAP/vacuum up to 0, **desired/commanded torque**, **actual/estimated torque**, optional torque source).
  - User profile specifying **max boost vs RPM** (psi/kPa).
- **Outputs:**
  - PWM duty to solenoid (30 Hz nominal).
  - On-screen gauge + warnings; serial/BT console for config/telemetry.

## Feature Requirements

- Configure in **PSI/kPa**, never in raw duty.
- **Self-learning** stored in non-volatile storage (separate from user config).
- **Failure modes must never overboost**; default bias is no-boost.
- Multiple **profiles**: valet/kill, daily, aggressive, scramble. Live switching. Configurable scramble target/profile.
- **Overboost protection:** immediate cut; recovery policy configurable (latched cool-down vs hysteretic resume).
- **Fault handling:** for initial release, all faults are **hard faults** (boost cut). Display reason.  
  Later: split hard vs soft once fault domain is better understood.
- **Gen2 Coyote Focus:** initial CAN support assumes Ford Gen2 Coyote; other platforms later via HAL.
- **UI Gauge Idea:** needle shows actual boost, tick around outside of gauge shows current target; background zones: vacuum (black) / normal boost (green) / near‑limit (yellow) / overboost (red + engire background flashing).

## Self-Learning

Similar to STFT/LTFT in ECUs: maintain fast and slow trims on the duty→boost mapping, persist with wear‑aware cadence.
Trims are **rate-limited** and bounded, and are **shared across profiles** (physics are common) while keeping profile targets distinct.
Provide a **console command** to reset trims.

## Power & Safety

- **0% duty ⇒ full pressure to lower dome ⇒ wastegates biased open ⇒ (near) zero boost.**  
- On **any fault** (sensor invalid, CAN timeout, storage failure): cut to 0% and display the fault.
- Default power in vehicle installs is **ignition-switched** to avoid battery drain; during prototyping on OBD‑II, manually unplug or power-gate.
