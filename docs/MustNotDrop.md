# Must-Not-Drop Invariants

These are non-negotiable. Any change must preserve them.

1. **Fail-safe:** `duty = 0%` forces full pressure to the lower dome (via NO path) ⇒ wastegates open ⇒ no boost.
2. **No raw-duty config:** All user-facing config is in PSI/kPa; internal maps convert to duty.
3. **Overboost:** Immediate duty cut to 0% on OB; recovery only below `(limit − hysteresis)` with configurable policy (latched cooldown or hysteretic resume).  
   Record OB events and soften controller aggressiveness after OB (anti-oscillation).
4. **Faults:** Any sensor/CAN/NVM fault ⇒ duty=0%, visible fault reason, and logged timestamp.
5. **Profiles:** Live switching is safe; scramble is an explicit, configured profile jump.
6. **Learning:** Trims are bounded, rate-limited, and separate from user config; provide command to reset.
7. **Core portability:** Control logic compiles/runs on desktop without MCU deps.
8. **Tests:** All math & key behaviors are covered by unit/integration tests; CI must pass before merge.
