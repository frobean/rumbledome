# Calibration & Defaults

## Solenoid
- Frequency: **30 Hz**
- Duty convention: **0% = lower dome pressurized (no-boost)**

## Profiles (initial seed)

### Valet (Boost Kill)
- Max curve: all RPM → **0 psi**
- OB limit: 2.0 psi (paranoia)
- Gains: kp=0.3, ki=0.1 (soft)
- Tip-in pulse: disabled
- Notes: This mode should keep gates open nearly always.

### Daily
- Spring: 5.0 psi
- Target curve (psi):
  - 1500: 0.0
  - 2500: 3.0
  - 3500: 7.0
  - 4000+: 8.0
- Overboost: limit=9.5 psi, hyst=0.3
- PI: kp=0.45, ki=0.30
- Slew: below 3k=2 %/s, above 3k=5 %/s
- Tip-in: ΔTPS=25%, min RPM=2600, pulse=100 ms

### Aggressive
- Curve +1–2 psi vs Daily, same OB limit initially
- PI a bit sharper: kp=0.6, ki=0.35

### Scramble
- Explicit profile, e.g., 9.0 psi target, same OB limit; activated by “scramble” input.

## Trims
- Fast EMA α≈0.2, Slow EMA α≈0.02 (tune in sim)
- Bound trims (e.g., ±10% duty equivalent)
- Persist every ~5–10 s when changed; batch writes to reduce wear.
