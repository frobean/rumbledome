# CAN Signals — Gen2 Coyote (SPECULATIVE)

⚠️ These are placeholders to enable code scaffolding and tests. Replace with verified IDs from a log.

- **RPM**
  - ID: TBD
  - Encoding: TBD (likely little-endian 16-bit; scale TBD)
- **MAP (vacuum to 0)**
  - ID: TBD
  - Units: kPa → convert to psi for display/logic
- **Desired/Commanded Torque**
  - ID: TBD
  - Units: Nm
- **Actual/Estimated Torque**
  - ID: TBD
  - Units: Nm
- **Torque Source**
  - ID: TBD
  - Enum mapping TBD

## Verification Plan
- Capture CAN logs during idle, cruise, WOT, and test events.
- Identify candidate IDs by correlation.
- Update this doc and the decoder tables in code; remove SPECULATIVE flags.
