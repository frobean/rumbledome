# Test Plan

## Unit Tests (rumbledome-core)
- PSI scaling from mV
- Curve interpolation (monotonic RPM, clamped ends)
- PI math (step response, no wind-up beyond bounds)
- Slew limit behavior (below/above 3k)
- Overboost latch & hysteresis
- Trim EMA update and bounds
- Fault → duty=0%

## Integration (rumbledome-sim)
- **Cruise:** rpm=2000, tps low → target=0, duty≈0
- **Tip-in:** at 2500 rpm, TPS jump → short pulse + PI → MAP tracks to target without overshoot
- **High RPM handoff:** 3500→4500 rpm blend to holdDuty
- **Overboost:** inject MAP > limit → cut to 0%; recover below limit−hyst with configured policy
- **Faults:** NaN/implausible sensor; CAN timeout; storage error → duty=0%, fault visible
- **Learning:** sustained bias (e.g., dome in drops from 15→10 psi) → trims compensate within bounds

## HIL
- Bench with real 4‑port solenoid @ 30 Hz; verify PWM, duty mapping, and sensor reads.
- Validate display updates and profile switching.

CI should run unit + sim tests on every push/PR.
