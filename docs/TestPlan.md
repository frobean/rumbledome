# RumbleDome Test Plan

## Testing Strategy Overview

RumbleDome's testing strategy employs multiple validation layers to ensure safety-critical operation:

1. **Unit Testing**: Individual component validation with 100% safety path coverage
2. **Integration Testing**: Desktop simulation with comprehensive scenarios  
3. **Hardware-in-Loop (HIL)**: Real hardware validation with safety systems
4. **System Validation**: Full vehicle integration testing

**Safety-First Approach**: All safety-critical functionality must be validated through automated testing before any hardware integration.

## Unit Testing (rumbledome-core)

### Safety System Testing

#### SY-001: Overboost Detection and Response
```rust
#[test]
fn test_overboost_immediate_response() {
    let mut core = setup_test_core();
    
    // Set manifold pressure above limit
    core.inject_manifold_pressure(10.0); // Limit: 9.5 psi
    
    core.execute_control_cycle().unwrap();
    
    assert_eq!(core.get_duty_cycle(), 0.0);
    assert_eq!(core.get_state(), SystemState::OverboostCut);
    assert!(core.get_response_time() < 100); // ms
}

#[test]
fn test_overboost_hysteresis() {
    let mut core = setup_test_core();
    core.set_overboost_limit(9.5, 0.3); // 9.5 psi limit, 0.3 hysteresis
    
    // Trigger overboost
    core.inject_manifold_pressure(9.6);
    core.execute_control_cycle().unwrap();
    assert_eq!(core.get_state(), SystemState::OverboostCut);
    
    // Drop below limit but above hysteresis threshold  
    core.inject_manifold_pressure(9.3); // Above 9.2 (9.5-0.3)
    core.execute_control_cycle().unwrap();
    assert_eq!(core.get_state(), SystemState::OverboostCut); // Still cut
    
    // Drop below hysteresis threshold
    core.inject_manifold_pressure(9.1); // Below 9.2
    core.execute_control_cycle().unwrap();
    assert_eq!(core.get_state(), SystemState::Armed); // Resume
}
```

#### SY-002: Fault Response Testing
```rust
#[test]
fn test_can_timeout_fault() {
    let mut core = setup_test_core();
    
    // Simulate CAN timeout (no torque signals for 500ms)
    core.simulate_can_timeout(600); // ms
    
    core.execute_control_cycle().unwrap();
    
    assert_eq!(core.get_duty_cycle(), 0.0);
    assert_eq!(core.get_state(), SystemState::Fault(FaultCode::CanTimeout));
}

#[test]
fn test_sensor_fault_detection() {
    let mut core = setup_test_core();
    
    // Inject implausible sensor reading
    core.inject_manifold_pressure(-5.0); // Negative pressure impossible
    
    core.execute_control_cycle().unwrap();
    
    assert_eq!(core.get_duty_cycle(), 0.0);
    assert_eq!(core.get_state(), SystemState::Fault(FaultCode::SensorImplausible));
}
```

### ECU Cooperation Testing

#### EC-001: Torque Ceiling Enforcement
```rust
#[test] 
fn test_torque_ceiling_respect() {
    let mut core = setup_test_core();
    core.set_torque_target_percentage(95.0);
    
    // ECU requests 200 Nm max, actual is 190 Nm
    core.inject_torque_signals(200.0, 190.0);
    
    // Target should be 95% of 200 = 190 Nm (already achieved)
    let torque_target = core.calculate_torque_target();
    assert_eq!(torque_target, 190.0);
    
    // Should not increase duty cycle significantly
    core.execute_control_cycle().unwrap();
    assert!(core.get_duty_cycle() < 5.0); // Minimal adjustment
}

#[test]
fn test_torque_undershoot_response() {
    let mut core = setup_test_core();
    core.set_torque_target_percentage(95.0);
    
    // ECU can handle 200 Nm, actual is only 150 Nm
    core.inject_torque_signals(200.0, 150.0);
    
    let torque_target = core.calculate_torque_target();
    assert_eq!(torque_target, 190.0); // 95% of 200
    
    // Should increase duty cycle to achieve more boost
    core.execute_control_cycle().unwrap();
    assert!(core.get_duty_cycle() > 10.0);
}

#[test]
fn test_torque_overshoot_prevention() {
    let mut core = setup_test_core();
    
    // Actual torque approaching ceiling
    core.inject_torque_signals(200.0, 198.0); // Very close to ceiling
    
    core.execute_control_cycle().unwrap();
    
    // Should reduce duty cycle to prevent ECU intervention
    assert!(core.get_duty_cycle() < core.get_previous_duty_cycle());
}
```

### Auto-Calibration Testing

#### AC-001: Progressive Safety Calibration
```rust
#[test]
fn test_progressive_calibration_start() {
    let mut core = setup_test_core();
    core.set_spring_pressure(5.0);
    
    core.start_calibration(4000, 8.0).unwrap(); // Target: 4000 RPM, 8.0 psi
    
    // Should start with conservative overboost limit
    assert_eq!(core.get_current_overboost_limit(), 6.0); // Spring + 1
    assert_eq!(core.get_calibration_state(), CalibrationState::Conservative);
}

#[test]
fn test_calibration_confidence_building() {
    let mut core = setup_test_core();
    core.start_calibration(4000, 8.0).unwrap();
    
    // Simulate 5 consistent calibration runs
    for run in 0..5 {
        let result = simulate_calibration_run(&mut core, 4000, 8.0, 23.5); // Consistent duty
        core.process_calibration_result(result).unwrap();
    }
    
    // Should advance to progressive phase
    assert_eq!(core.get_calibration_state(), CalibrationState::Progressive);
    assert!(core.get_current_overboost_limit() > 6.0);
}

#[test]
fn test_calibration_safety_bounds() {
    let mut core = setup_test_core();
    core.start_calibration(4000, 8.0).unwrap();
    
    // Simulate dangerous duty cycle result
    let dangerous_result = CalibrationRunResult {
        duty_cycle: 50.0, // Unreasonably high
        achieved_boost: 12.0, // Way over target
        response_time: 200, // Slow response
    };
    
    // Should reject dangerous result and abort calibration
    assert!(core.process_calibration_result(dangerous_result).is_err());
    assert_eq!(core.get_calibration_state(), CalibrationState::Aborted);
}
```

### Learning System Testing

#### LS-001: Duty Cycle Learning
```rust
#[test]
fn test_duty_cycle_learning_bounds() {
    let mut core = setup_test_core();
    
    // Start with baseline duty cycle
    let initial_duty = core.get_learned_duty(4000, 8.0).unwrap();
    
    // Simulate learning scenario - slight undershoot
    for _ in 0..100 {
        core.inject_scenario(4000, 8.0, 7.8); // Slight undershoot
        core.execute_control_cycle().unwrap();
    }
    
    let learned_duty = core.get_learned_duty(4000, 8.0).unwrap();
    
    // Should learn slightly higher duty, but within bounds
    assert!(learned_duty > initial_duty);
    assert!(learned_duty - initial_duty < 5.0); // Max 5% change
}

#[test]
fn test_learning_slew_rate_limits() {
    let mut core = setup_test_core();
    core.set_learning_slew_rate(1.0); // 1% per second max
    
    let initial_duty = core.get_learned_duty(4000, 8.0).unwrap();
    
    // Try to force rapid learning change
    for _ in 0..10 { // 10 cycles at 100Hz = 0.1 seconds
        core.inject_rapid_change_scenario();
        core.execute_control_cycle().unwrap();
    }
    
    let learned_duty = core.get_learned_duty(4000, 8.0).unwrap();
    
    // Should be slew-rate limited to ~0.1% change (0.1s * 1%/s)
    assert!((learned_duty - initial_duty).abs() < 0.2);
}
```

### Pneumatic System Testing

#### PS-001: Pressure Sensor Validation
```rust
#[test]
fn test_pressure_sensor_scaling() {
    // Test 0-30 psi, 0.5-4.5V scaling
    assert_eq!(scale_pressure_mv(500), 0.0);   // 0.5V = 0 psi
    assert_eq!(scale_pressure_mv(2500), 15.0); // 2.5V = 15 psi  
    assert_eq!(scale_pressure_mv(4500), 30.0); // 4.5V = 30 psi
    
    // Test bounds
    assert_eq!(scale_pressure_mv(400), 0.0);   // Below 0.5V clamped
    assert_eq!(scale_pressure_mv(5000), 30.0); // Above 4.5V clamped
}

#[test]
fn test_pneumatic_optimization() {
    let mut optimizer = PneumaticOptimizer::new();
    
    // Test optimal pressure calculation
    let recommendation = optimizer.recommend_input_pressure(
        9.5,  // max_boost_target
        5.0,  // spring_pressure
        20.0, // current_duty_usage_low
        70.0  // current_duty_usage_high
    );
    
    assert!(recommendation.recommended_psi > 10.0);
    assert!(recommendation.recommended_psi < 20.0);
    assert_eq!(recommendation.rationale, "Optimal for 20-70% duty cycle range");
}
```

## Integration Testing (rumbledome-sim)

### Comprehensive Scenario Testing

#### IS-001: Complete System Scenarios
```rust
#[test]
fn test_full_driving_scenario() {
    let mut sim = RumbledomeSim::new();
    sim.load_scenario("highway_acceleration.json");
    
    // Simulate 30-second acceleration scenario
    sim.run_scenario_duration(Duration::from_secs(30));
    
    let results = sim.get_results();
    
    // Validate system behavior
    assert!(results.max_boost < 9.5); // Never exceeded limit
    assert!(results.ecu_interventions == 0); // No harsh ECU responses
    assert!(results.boost_consistency > 0.9); // Consistent response
    assert!(results.torque_delivery_smoothness > 0.85);
}

#[test] 
fn test_calibration_sequence() {
    let mut sim = RumbledomeSim::new();
    sim.load_scenario("dyno_calibration.json");
    
    // Start calibration at 4000 RPM, 8.0 PSI target
    sim.start_calibration(4000, 8.0);
    
    // Run 5 WOT pulls with progressive limits
    for pull in 0..5 {
        sim.execute_wot_pull(Duration::from_secs(15));
        assert!(sim.get_overboost_limit() >= 6.0); // Progressive increase
    }
    
    // Verify successful calibration
    assert!(sim.get_calibration_state() == CalibrationState::Complete);
    assert!(sim.get_learned_duty(4000, 8.0).is_some());
}
```

#### IS-002: Fault Injection Testing
```rust
#[test]
fn test_sensor_failure_scenarios() {
    let mut sim = RumbledomeSim::new();
    
    // Simulate manifold sensor failure during boost
    sim.run_duration(Duration::from_secs(5)); // Normal operation
    sim.inject_sensor_failure(SensorType::Manifold);
    sim.run_duration(Duration::from_secs(2));
    
    // System should immediately go to fault state
    assert_eq!(sim.get_system_state(), SystemState::Fault(FaultCode::SensorFailure));
    assert_eq!(sim.get_duty_cycle(), 0.0);
}

#[test]
fn test_can_bus_failure_recovery() {
    let mut sim = RumbledomeSim::new();
    
    // Normal operation then CAN failure
    sim.run_duration(Duration::from_secs(5));
    sim.inject_can_failure();
    sim.run_duration(Duration::from_secs(1));
    
    // Should be in fault state
    assert_eq!(sim.get_system_state(), SystemState::Fault(FaultCode::CanTimeout));
    
    // Restore CAN and verify recovery
    sim.restore_can();
    sim.run_duration(Duration::from_secs(2));
    
    // Should require manual fault acknowledgment
    assert_eq!(sim.get_system_state(), SystemState::Fault(FaultCode::CanTimeout));
    
    sim.acknowledge_fault();
    sim.run_duration(Duration::from_secs(1));
    
    assert_eq!(sim.get_system_state(), SystemState::Armed);
}
```

#### IS-003: Safety System Integration Testing
```rust
#[test]
fn test_traction_control_integration() {
    let mut sim = RumbledomeSim::new();
    
    // Simulate high torque request during normal acceleration
    sim.inject_torque_signals(250.0, 200.0); // ECU wants 250 Nm
    sim.run_duration(Duration::from_millis(500));
    let normal_duty = sim.get_duty_cycle();
    
    // Simulate traction control activation (desired torque drops)
    sim.inject_torque_signals(150.0, 200.0); // TC drops desired to 150 Nm
    sim.run_duration(Duration::from_millis(100));
    
    // Should immediately reduce boost to help ECU reduce torque
    assert!(sim.get_duty_cycle() < normal_duty);
    assert_eq!(sim.get_system_state(), SystemState::Armed); // No fault
}

#[test]
fn test_abs_integration() {
    let mut sim = RumbledomeSim::new();
    
    // Normal braking scenario with some boost
    sim.inject_torque_signals(180.0, 180.0); // Balanced
    sim.run_duration(Duration::from_millis(500));
    
    // ABS activation - ECU drastically reduces desired torque
    sim.inject_torque_signals(50.0, 180.0); // ABS drops desired torque
    sim.run_duration(Duration::from_millis(50));
    
    // Should quickly reduce boost to help ECU cut torque
    assert!(sim.get_duty_cycle() < 10.0);
    assert_eq!(sim.get_system_state(), SystemState::Armed);
}

#[test]
fn test_rapid_profile_switching() {
    let mut sim = RumbledomeSim::new();
    
    // Rapidly switch between profiles during boost
    sim.set_profile("Daily");
    sim.run_duration(Duration::from_millis(100));
    
    sim.set_profile("Aggressive");
    sim.run_duration(Duration::from_millis(100));
    
    sim.set_profile("Valet");
    sim.run_duration(Duration::from_millis(100));
    
    // System should handle gracefully without faults
    assert_ne!(sim.get_system_state(), SystemState::Fault(_));
    assert!(sim.get_duty_cycle() >= 0.0);
}

#[test]
fn test_environmental_compensation() {
    let mut sim = RumbledomeSim::new();
    
    // Baseline calibration at sea level, 70°F
    sim.set_environmental_conditions(0, 70.0); // altitude_ft, temp_f
    sim.calibrate_point(4000, 8.0);
    let baseline_duty = sim.get_learned_duty(4000, 8.0).unwrap();
    
    // Test high altitude compensation
    sim.set_environmental_conditions(5000, 70.0);
    sim.run_duration(Duration::from_secs(10));
    let altitude_duty = sim.get_learned_duty(4000, 8.0).unwrap();
    
    // Should require higher duty cycle at altitude
    assert!(altitude_duty > baseline_duty);
    assert!(altitude_duty - baseline_duty < 10.0); // Bounded compensation
}
```

## Hardware-in-Loop (HIL) Testing

### HIL-001: Pneumatic System Validation

#### Real Solenoid Response Testing
```
Test Equipment:
- 4-port MAC solenoid valve  
- Compressed air supply (regulated to test pressures)
- Pressure measurement equipment
- Teensy 4.1 with production firmware

Test Procedure:
1. Measure actual overboost response times at various input pressures
2. Validate duty cycle linearity and accuracy
3. Confirm pressure sensor calibration accuracy
4. Test pneumatic system optimization recommendations

Pass Criteria:
- Overboost response < 100ms for recommended input pressures
- Duty cycle accuracy within ±1%
- Pressure sensor accuracy within ±0.25% full scale
- System recommendations result in measurable improvements
```

#### HIL-002: CAN Bus Integration Testing
```
Test Equipment:
- Ford Gen2 Coyote ECU or CAN simulator
- Production wiring harness
- CAN bus analyzer

Test Procedure:
1. Verify CAN message reception and parsing
2. Test ECU cooperation - no harsh interventions
3. Validate fault detection for CAN timeouts
4. Test message filtering and processing performance

Pass Criteria:
- 100% message reception rate at 500 kbps
- Zero ECU spark cuts or fuel cuts during normal operation  
- CAN timeout detection within 500ms
- <10ms latency from CAN message to control action
```

### HIL-003: Safety System Validation

#### Emergency Response Testing
```
Test Procedure:
1. Induce actual overboost condition with controlled setup
2. Measure response time from detection to wastegate opening
3. Test multiple failure scenarios (sensor, power, communication)
4. Validate fault reporting and logging accuracy

Pass Criteria:
- All overboost responses < 100ms
- All fault conditions result in duty = 0%
- Fault logging captures complete system state
- Manual recovery required after fault conditions
```

## System Validation Testing

### SV-001: Vehicle Integration Testing

#### On-Vehicle Validation (Controlled Environment)
```
Test Environment: Closed course or dyno facility

Test Phases:
1. **Baseline Testing**: Compare against spring-only operation
2. **Calibration Validation**: Perform complete auto-calibration sequence  
3. **Performance Testing**: Validate smooth power delivery
4. **Safety Testing**: Confirm overboost protection under real conditions
5. **Durability Testing**: Extended operation validation

Success Criteria:
- Smoother torque delivery than spring-only system
- No ECU adaptation or fault codes
- Successful auto-calibration completion  
- Zero overboost events during normal operation
- Stable operation over extended test period
```

## Test Automation and CI/CD

### Continuous Integration Pipeline
```yaml
# .github/workflows/test.yml
- Unit Tests: Run on every commit
- Integration Tests: Run on every PR
- Safety Tests: Required for merge
- HIL Tests: Run on release candidates
- Performance Tests: Weekly regression testing
```

### Test Coverage Requirements
- **Safety Code**: 100% path coverage mandatory
- **Control Logic**: 95% coverage minimum  
- **HAL Implementations**: 90% coverage minimum
- **Protocol Handling**: 95% coverage minimum

### Test Data Management
- **Scenario Libraries**: Standardized test scenarios for repeatability
- **Baseline Data**: Known-good reference data for regression testing
- **Failure Databases**: Historical failure modes for comprehensive testing

## Validation Sign-off Criteria

### Phase 1 Release Criteria
- [ ] All unit tests pass (100% safety coverage)
- [ ] All integration scenarios pass
- [ ] HIL validation complete with production hardware
- [ ] Vehicle integration testing successful
- [ ] Safety system validation complete
- [ ] Documentation and test reports complete

### Quality Gates
- **No safety test failures tolerated**
- **No regressions in core functionality**
- **Performance meets or exceeds requirements**
- **All identified risks mitigated or accepted**

This comprehensive test plan ensures that RumbleDome meets its safety-critical requirements while delivering the innovative torque-aware boost control capabilities.