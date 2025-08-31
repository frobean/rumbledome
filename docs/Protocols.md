# RumbleDome Communication Protocols

## JSON/CLI Protocol

All communication with RumbleDome uses JSON messages over Serial/Bluetooth. All responses include `"ok": true|false` and `"err"` field when false.

### System Status and Monitoring

#### Read Current Status
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
    "dome_input_psi": 12.1,
    "dome_upper_psi": 0.8,
    "desired_torque": 185.0,
    "actual_torque": 180.0,
    "target_boost": 5.0,
    "learned_duty": 24.5,
    "profile": "Daily",
    "state": "Armed",
    "calibration_progress": "6.5/9.5 psi",
    "pneumatic_health": "optimal",
    "faults": []
  }
}
```

#### Get System Configuration
```json
{ "cmd": "get_config" }
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "spring_pressure": 5.0,
    "profiles": {
      "Daily": {
        "boost_targets": [[1500, 0.0], [2500, 3.0], [3500, 7.0], [4500, 8.0]],
        "overboost_limit": 9.5,
        "overboost_hysteresis": 0.3
      }
    },
    "active_profile": "Daily",
    "scramble_profile": "Track",
    "torque_target_percentage": 95,
    "boost_slew_rate": 2.0
  }
}
```

### Profile Management

#### Set Profile Configuration
```json
{
  "cmd": "set_profile",
  "name": "Daily",
  "boost_targets": [[1500, 0.0], [2500, 3.0], [3500, 7.0], [4500, 8.0]],
  "overboost_limit": 9.5,
  "overboost_hysteresis": 0.3
}
```

#### Switch Active Profile
```json
{
  "cmd": "set_active_profile",
  "name": "Daily"
}
```

#### Set Scramble Profile
```json
{
  "cmd": "set_scramble_profile", 
  "name": "Track"
}
```

### Auto-Calibration Control

#### Start Calibration Session
```json
{
  "cmd": "start_calibration",
  "target_rpm": 4000,
  "target_boost": 8.0,
  "max_overboost": 9.0
}
```

#### Get Calibration Status
```json
{ "cmd": "calibration_status" }
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "active": true,
    "current_target": {
      "rpm": 4000,
      "boost_psi": 8.0
    },
    "current_overboost_limit": 9.0,
    "final_overboost_limit": 9.5,
    "runs_completed": 3,
    "runs_required": 5,
    "learned_duty": 23.5,
    "confidence": 0.85,
    "next_step": "increase_limit_to_9.2"
  }
}
```

#### Abort Calibration
```json
{ "cmd": "abort_calibration" }
```

### Learning Data Management

#### Reset All Learned Data
```json
{ "cmd": "reset_learned_data" }
```

#### Get Learning Status
```json
{ "cmd": "learning_status" }
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "calibration_points": 24,
    "confidence_average": 0.78,
    "environmental_factors": {
      "temperature_compensation": 1.05,
      "altitude_compensation": 0.98,
      "supply_pressure_baseline": 14.2
    },
    "last_updated": "2024-08-30T15:30:45Z"
  }
}
```

### Pneumatic System Optimization

#### Get Pneumatic System Health
```json
{ "cmd": "pneumatic_health" }
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "status": "optimal",
    "current_input_psi": 15.2,
    "recommended_input_psi": 16.0,
    "duty_cycle_utilization": "45-65%",
    "overboost_response_time": 85,
    "max_safe_response_time": 100,
    "recommendations": []
  }
}
```

#### Request Input Pressure Recommendation
```json
{
  "cmd": "recommend_input_pressure",
  "max_boost_target": 9.5,
  "spring_pressure": 5.0
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "recommended_psi": 16.0,
    "rationale": "Optimal for 20-70% duty cycle range",
    "current_psi": 15.2,
    "adjustment_needed": "+0.8 psi",
    "expected_benefits": [
      "Better control resolution",
      "Improved safety response time"
    ]
  }
}
```

### System Configuration

#### Set System Parameters
```json
{
  "cmd": "set_system_config",
  "spring_pressure": 5.0,
  "torque_target_percentage": 95,
  "boost_slew_rate": 2.0
}
```

#### Set Fault Response Configuration
```json
{
  "cmd": "set_fault_config",
  "can_timeout_ms": 500,
  "sensor_fault_threshold": 3,
  "overboost_response_time_ms": 100
}
```

### Diagnostic and Debug Commands

#### Get Diagnostic Data
```json
{ "cmd": "diagnostics" }
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "system_uptime": 3600,
    "control_loop_frequency": 99.8,
    "can_message_rate": 50.2,
    "sensor_readings": {
      "dome_input_mv": 2845,
      "dome_upper_mv": 1234,
      "manifold_mv": 3210
    },
    "memory_usage": {
      "heap_used": 1024,
      "stack_used": 512
    },
    "fault_history": []
  }
}
```

#### Enable Debug Logging
```json
{
  "cmd": "set_log_level",
  "level": "debug"
}
```

### Error Response Format

All commands return error responses in this format when `"ok": false`:

```json
{
  "ok": false,
  "err": "Invalid profile name",
  "details": {
    "code": "PROFILE_NOT_FOUND",
    "requested_profile": "InvalidName",
    "available_profiles": ["Valet", "Daily", "Aggressive", "Track"]
  }
}
```

## Communication Transport

### Serial Interface
- **Baud Rate**: 115200
- **Format**: 8N1
- **Flow Control**: None
- **Line Ending**: `\n`
- **Encoding**: UTF-8

### Bluetooth Interface (Future)
- **Protocol**: Bluetooth Serial Profile (SPP)
- **Same JSON message format as serial
- **Pairing**: Required for security
- **Range**: Typical 10-meter range for configuration

## Message Timing and Constraints

### Request Limits
- **Maximum message size**: 1KB
- **Request timeout**: 5 seconds
- **Concurrent requests**: 1 (serial protocol)
- **Status polling**: Maximum 10Hz recommended

### Response Guarantees
- **Status responses**: <100ms typical
- **Configuration changes**: <500ms
- **Calibration operations**: May take several seconds
- **Learning data operations**: <1 second

## Protocol Versioning

### Version Information
```json
{
  "cmd": "version"
}
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "firmware_version": "1.0.0",
    "protocol_version": "1.0",
    "build_date": "2024-08-30",
    "hardware_platform": "teensy41"
  }
}
```

### Protocol Compatibility
- **Major version changes**: Breaking changes requiring client updates
- **Minor version changes**: Backward-compatible additions
- **Clients should check protocol_version** on connection
- **Unsupported protocol versions** return specific error codes