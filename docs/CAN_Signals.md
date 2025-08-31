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
- **Throttle Position (TPS)**
  - ID: TBD
  - Units: % (0-100)
  - Notes: Phase 2+ predictive control feature
- **Drive Mode**
  - ID: TBD
  - Values: Normal/Sport/Sport+/Track (enum mapping TBD)
  - Notes: Phase 2+ delivery style integration

## Verification Plan
- Capture CAN logs during idle, cruise, WOT, and test events.
- Identify candidate IDs by correlation.
- Test drive mode switching to identify mode signals.
- Validate torque signals correlate with expected ECU behavior.
- Update this doc and the HAL CAN implementation; remove SPECULATIVE flags.

## Implementation Notes
- **Phase 1 MVP**: Only RPM, MAP, and torque signals required
- **Phase 2+**: Add TPS and drive mode for advanced features
- **HAL Integration**: Platform-specific signal decoding in HAL layer
- **Future Platforms**: GM, Mopar support through additional CAN protocol implementations
