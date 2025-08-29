# Implementation Specification

## Workspace Layout (Rust)

```
rumbledome-hal/   # Traits + MCU/Desktop impls (feature flags)
rumbledome-core/  # State machine, PI, learning, limits, safety (no std-unsafe MCU deps)
rumbledome-iface/ # JSON/CLI protocol & schema
rumbledome-fw/    # Teensy 4.1 firmware wiring (pins, PWM, ADC, CAN, display)
rumbledome-sim/   # Desktop simulator & scenarios
```

## HAL Traits (rumbledome-hal)

- `Time`: `now_ms()`, `sleep_ms(n)`
- `Pwm`: `set_duty(percent)`, `set_freq(hz)`
- `Analog`: `read_mv(channel) -> i32` (millivolts), scaling in core
- `Storage`: `read(key) -> bytes`, `write(key, bytes)`, wear-aware wrapper
- `Can`: `subscribe(ids)`, `poll() -> frames`, `send(frame)`
- `Display`: small primitives for the gauge & text
- `Gpio`: inputs for profile/scramble buttons
- `FaultReporter`: `raise(code, msg)`, `clear(code)`

Mock HALs live in `rumbledome-hal` behind `cfg(feature="host")`.

## Core Models (rumbledome-core)

- `Profile`: `name`, `max_boost_curve: [(rpm, psi)]`, `ob_limit`, `ob_hyst`, `kp`, `ki`, `slew_limits`, `tipin_pulse_ms`, etc.
- `Trims`: fast/slow EMA terms on duty map (bounded).
- `Targets`: torque-informed target computation: `target_psi = min(profile_curve(rpm), torque_model(...))`.
- `Controller`: PI + tip-in pulse + slew limits → duty%.
- `Safety`: OB detection/latch, fault integration (forces duty=0%), sensor plausibility checks.
- `State`: `Idle`, `Armed`, `CutOB`, `Fault`.

## Control Loop

At ~100 Hz:
1. Read sensors (MAP, dome supply, dome top), CAN (rpm, torque desired/actual, optional source).
2. Validate; on invalid → Fault (duty 0%).
3. Compute `target_psi` (profile + torque context + anti-lug rules).
4. PI step on `error = target_psi − measured_psi` (measured_psi from MAP above 0 psi; otherwise 0).
5. Apply feedforward/tip-in and slew limits; clamp to [0, 100].
6. Overboost check → possibly enter `CutOB` with recovery policy.
7. Update PWM duty; update trims (bounded/rate-limited).

## Iface (rumbledome-iface)

- JSON commands (examples in Interfaces.md): get/set config/profile, read stats, reset trims, set scramble target, etc.
- Robust parsing & validation; no policy decisions.

## Firmware (rumbledome-fw)

- Pinmap and init for Teensy 4.1: PWM ~30 Hz, ADC channels for 3 sensors, CAN via SN65HVD230, ST7735R display.
- Loop timing, NVM commit cadence, fault LEDs, button handling, UI updates.

## Simulator (rumbledome-sim)

- Deterministic model of boost response (first-order lag), dome dynamics, and tip-in scenarios.
- Scriptable test cases that assert expected controller outputs and transitions.
