# Interfaces

## Console/JSON Protocol (examples)

All messages are JSON lines over Serial/BLE; responses include `"ok": true|false` and `"err"` when false.

### Read current status
```json
{ "cmd": "status" }
```
**Response:**
```json
{
  "ok": true,
  "data": {
    "rpm": 2850,
    "map_psi": 3.2,
    "dome_in_psi": 12.1,
    "dome_top_psi": 0.8,
    "target_psi": 5.0,
    "duty": 24.5,
    "profile": "Daily",
    "state": "Armed",
    "faults": []
  }
}
```

### Get config
```json
{ "cmd": "get_config" }
```

### Set profile curve (psi vs rpm)
```json
{
  "cmd": "set_profile",
  "name": "Daily",
  "curve": [[1500, 0.0], [2500, 3.0], [3500, 7.0], [4500, 8.0]],
  "ob_limit": 9.5,
  "ob_hyst": 0.3,
  "kp": 0.45,
  "ki": 0.30
}
```

### Reset trims
```json
{ "cmd": "reset_trims" }
```

### Set scramble
```json
{ "cmd": "set_scramble_profile", "name": "Scramble" }
```

## ADC Scaling

For 0–30 psi, 0.5–4.5 V at 5 V supply:
```
psi = clamp(((mv/1000.0) - 0.5) / 4.0 * 30.0, 0, 30)
```

## PWM

- Nominal **30 Hz**.  
- **0% duty** = NO path active ⇒ lower dome pressurized ⇒ **wastegates open**.  
- 100% duty biases upper dome (close gates), within mechanical limits.

## CAN (SPECULATIVE — replace with verified IDs)

- **RPM**: ID TBD, scale TBD  
- **MAP (vacuum to 0)**: ID TBD, kPa → psi conversion  
- **Desired torque**: ID TBD, Nm  
- **Actual torque**: ID TBD, Nm  
- **Torque source**: ID TBD (optional)

*Mark all code paths consuming CAN as “SPECULATIVE” until verified.*
