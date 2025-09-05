# RumbleDome Test Plan

📋 **Safety requirements reference**: See **[Safety.md](Safety.md)** for all SY-* safety requirements being validated  
🏗️ **Implementation approach**: See **[Implementation.md](Implementation.md)** for testing architecture and framework  
⚙️ **Technical specifications**: See **[TechnicalSpecs.md](TechnicalSpecs.md)** for hardware validation targets

## Testing Philosophy

**Safety-First Validation**: All safety-critical logic must achieve 100% test coverage through desktop simulation before any hardware integration. The HAL abstraction enables comprehensive testing without physical hardware risk.

**Progressive Validation Approach**:
1. **Desktop Simulation** - Validate all control logic with zero hardware risk
2. **CAN Signal Validation** - Verify Ford S550 signal interpretation with actual vehicle
3. **Pneumatic Bench Testing** - Validate mechanical response times and safety systems
4. **Vehicle Integration** - Full system validation in controlled environment

## Unit Testing Framework (rumbledome-core)

### Safety System Validation (SY-* Requirements)

#### SY-1: Pneumatic Fail-Safe Operation
```rust
#[test]
fn test_duty_zero_forces_wastegate_open() {
    let mut hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, test_config());
    
    // Force duty cycle to 0%
    core.hal.set_manifold_pressure(15.0);  // Above overboost limit
    core.execute_control_cycle().unwrap();
    
    // Verify pneumatic fail-safe behavior
    assert_eq!(core.get_duty_cycle(), 0.0);
    assert!(core.hal.get_lower_dome_pressure() > core.hal.get_upper_dome_pressure());
    assert_eq!(core.get_system_state(), SystemState::OverboostCut);
}

#[test]
fn test_electronic_failure_defaults_to_safe_state() {
    let mut hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, test_config());
    
    // Simulate complete electronic failure
    core.hal.inject_fault(HalError::SystemFailure);
    
    let result = core.execute_control_cycle();
    assert!(result.is_err());
    assert_eq!(core.get_duty_cycle(), 0.0);  // Must default to safe state
}
```

#### SY-2: High-Authority System Recognition  
```rust
#[test]
fn test_conservative_duty_cycle_limits() {
    let mut hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, test_config());
    
    // Even aggressive settings should be conservative in duty cycle
    core.set_aggression(1.0);  // Maximum aggression
    core.hal.set_torque_signals(300.0, 200.0);  // Large torque gap
    
    core.execute_control_cycle().unwrap();
    
    // High-authority system: even max aggression should be conservative
    assert!(core.get_duty_cycle() < 0.8);  // Never exceed 80% duty
    assert!(core.get_estimated_boost_capability() < 15.0);  // Reasonable boost estimate
}
```

#### SY-3: Overboost Response Validation
```rust
#[test]
fn test_overboost_response_time() {
    let mut hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, test_config());
    
    // Set normal operation
    core.hal.set_manifold_pressure(5.0);
    core.execute_control_cycle().unwrap();
    assert!(core.get_duty_cycle() > 0.0);
    
    // Trigger overboost condition
    let start_time = core.hal.get_time_ms();
    core.hal.set_manifold_pressure(16.0);  // Above overboost limit
    core.execute_control_cycle().unwrap();
    let response_time = core.hal.get_time_ms() - start_time;
    
    // Verify immediate response (SY-3)
    assert_eq!(core.get_duty_cycle(), 0.0);
    assert!(response_time < 100);  // <100ms response requirement
    assert_eq!(core.get_system_state(), SystemState::OverboostCut);
}

#[test]
fn test_user_configurable_overboost_limits() {
    let mut hal = MockHal::new();
    
    // Test different user-configured overboost limits
    for overboost_limit in [12.0, 15.0, 18.0] {
        let config = SystemConfig {
            overboost_limit,
            ..test_config()
        };
        let mut core = RumbleDomeCore::new(MockHal::new(), config);
        
        // Should not trigger below limit
        core.hal.set_manifold_pressure(overboost_limit - 0.1);
        core.execute_control_cycle().unwrap();
        assert_ne!(core.get_system_state(), SystemState::OverboostCut);
        
        // Should trigger at limit
        core.hal.set_manifold_pressure(overboost_limit + 0.1);
        core.execute_control_cycle().unwrap();
        assert_eq!(core.get_system_state(), SystemState::OverboostCut);
    }
}
```

#### SY-4: Progressive Calibration Safety
```rust
#[test]
fn test_calibration_starts_conservative() {
    let mut hal = MockHal::new();
    let config = SystemConfig {
        spring_pressure: 5.0,
        overboost_limit: 15.0,
        ..test_config()
    };
    let mut core = RumbleDomeCore::new(hal, config);
    
    core.start_calibration_session().unwrap();
    
    // Should start at spring + 1 PSI
    let initial_limit = core.get_current_overboost_limit();
    assert_eq!(initial_limit, 6.0);  // 5.0 + 1.0
    
    // Should only increase after proving safety
    for _ in 0..2 {  // Not enough runs yet
        core.process_calibration_run(successful_run()).unwrap();
    }
    assert_eq!(core.get_current_overboost_limit(), 6.0);  // Still conservative
    
    // After sufficient validation runs
    core.process_calibration_run(successful_run()).unwrap();
    assert!(core.get_current_overboost_limit() > 6.0);  // Now can increase
}
```

#### SY-5: No Raw Duty Cycle Configuration
```rust
#[test]
fn test_no_user_duty_cycle_access() {
    let config = SystemConfig::default();
    
    // Configuration should only expose PSI/kPa values
    assert!(config.max_boost_psi > 0.0);
    assert!(config.overboost_limit > 0.0);
    assert!(config.spring_pressure > 0.0);
    
    // Should not expose any raw duty cycle parameters
    // This is a compile-time test - if these fields exist, compilation fails
    // assert!(config.target_duty_cycle);  // Should not compile
    // assert!(config.calibration_duty);   // Should not compile
}
```

#### SY-6: Torque Ceiling Enforcement
```rust
#[test]
fn test_torque_ceiling_respected() {
    let mut hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, test_config());
    
    // Set desired torque and actual torque at ceiling (95%)
    let desired_torque = 200.0;
    let torque_ceiling = desired_torque * 0.95;  // 95% ceiling
    core.hal.set_torque_signals(desired_torque, torque_ceiling);
    
    core.execute_control_cycle().unwrap();
    
    // Should provide minimal assistance when at ceiling
    assert!(core.get_duty_cycle() < 0.2);
    
    // When exceeding ceiling, should back off
    core.hal.set_torque_signals(desired_torque, torque_ceiling + 5.0);
    core.execute_control_cycle().unwrap();
    
    // Should reduce boost assistance
    assert!(core.get_duty_cycle() < 0.1);
}
```

### Torque-Following Control Logic

#### Core Torque Gap Analysis
```rust
#[test]
fn test_torque_gap_calculation() {
    let mut hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, test_config());
    
    // Test various torque gap scenarios
    let test_cases = [
        (200.0, 200.0, 0.0),    // No gap - minimal assistance
        (200.0, 190.0, 10.0),   // Small gap - some assistance  
        (200.0, 170.0, 30.0),   // Large gap - significant assistance
        (200.0, 210.0, -10.0),  // Negative gap - reduce assistance
    ];
    
    for (desired, actual, expected_gap) in test_cases {
        core.hal.set_torque_signals(desired, actual);
        core.execute_control_cycle().unwrap();
        
        let calculated_gap = core.get_current_torque_gap();
        assert_eq!(calculated_gap, expected_gap);
        
        // Assistance should correlate with gap size
        let duty = core.get_duty_cycle();
        if expected_gap > 15.0 {
            assert!(duty > 0.3);  // Significant assistance
        } else if expected_gap < 5.0 {
            assert!(duty < 0.2);  // Minimal assistance
        }
    }
}

#[test]
fn test_ecu_cooperation_behavior() {
    let mut hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, test_config());
    
    // ECU successfully achieving torque target
    core.hal.set_torque_signals(200.0, 195.0);  // 5 Nm gap - acceptable
    core.execute_control_cycle().unwrap();
    let baseline_duty = core.get_duty_cycle();
    assert!(baseline_duty < 0.3);  // Minimal assistance when ECU succeeding
    
    // ECU struggling with torque delivery
    core.hal.set_torque_signals(200.0, 175.0);  // 25 Nm gap - ECU needs help
    core.execute_control_cycle().unwrap();
    let assistance_duty = core.get_duty_cycle();
    assert!(assistance_duty > baseline_duty);  // Increased assistance
    
    // ECU returns to successful operation  
    core.hal.set_torque_signals(200.0, 195.0);
    core.execute_control_cycle().unwrap();
    // Should gradually return to minimal assistance (not instant)
    assert!(core.get_duty_cycle() < assistance_duty);
}
```

#### Aggression Scaling Behavior
```rust
#[test]
fn test_aggression_scaling_characteristics() {
    let mut hal = MockHal::new();
    
    // Test different aggression levels with same torque scenario
    let torque_scenarios = [(200.0, 175.0)];  // 25 Nm gap
    let aggression_levels = [0.2, 0.5, 0.8];
    
    for &(desired, actual) in &torque_scenarios {
        let mut previous_duty = 0.0;
        
        for &aggression in &aggression_levels {
            let config = SystemConfig {
                aggression,
                ..test_config()
            };
            let mut core = RumbleDomeCore::new(MockHal::new(), config);
            core.hal.set_torque_signals(desired, actual);
            
            core.execute_control_cycle().unwrap();
            let duty = core.get_duty_cycle();
            
            // Higher aggression should result in higher duty cycle response
            assert!(duty > previous_duty);
            previous_duty = duty;
        }
    }
}
```

### Learning System Validation

#### Duty Cycle Calibration Learning
```rust
#[test]
fn test_duty_cycle_learning_convergence() {
    let mut hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, test_config());
    
    // Simulate repeated operation at specific RPM/boost point
    let target_rpm = 4000;
    let target_boost = 8.0;
    
    // Initially no learned data
    assert!(core.get_learned_duty(target_rpm, target_boost).is_err());
    
    // Run multiple learning cycles
    for cycle in 0..10 {
        core.hal.set_rpm(target_rpm);
        core.hal.set_manifold_pressure(target_boost);
        core.hal.set_torque_signals(250.0, 240.0);  // Consistent torque gap
        
        core.execute_control_cycle().unwrap();
        
        // Learning should converge over time
        if cycle > 5 {
            let learned_duty = core.get_learned_duty(target_rpm, target_boost).unwrap();
            assert!(learned_duty > 0.0);
            assert!(learned_duty < 1.0);
        }
    }
}

#[test]  
fn test_environmental_compensation_learning() {
    let mut hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, test_config());
    
    // Establish baseline at standard conditions
    for _ in 0..5 {
        core.hal.set_environmental_conditions(20.0, 1013.25, 50.0);  // 20°C, sea level, 50% humidity
        core.execute_learning_cycle().unwrap();
    }
    let baseline_duty = core.get_learned_duty(4000, 8.0).unwrap();
    
    // Change environmental conditions
    for _ in 0..5 {
        core.hal.set_environmental_conditions(5.0, 900.0, 80.0);  // Cold, high altitude, humid
        core.execute_learning_cycle().unwrap();
    }
    let compensated_duty = core.get_learned_duty(4000, 8.0).unwrap();
    
    // Should learn environmental compensation
    assert_ne!(baseline_duty, compensated_duty);
    assert!(core.get_environmental_compensation_factor() != 1.0);
}
```

#### SY-11: Learning System Bounds
```rust
#[test]
fn test_learning_bounds_enforcement() {
    let mut hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, test_config());
    
    // Establish baseline
    let baseline_duty = 0.5;
    core.set_learned_baseline(4000, 8.0, baseline_duty);
    
    // Try to learn extreme adjustment
    for _ in 0..100 {  // Many learning cycles
        core.inject_extreme_learning_condition();
        core.execute_control_cycle().unwrap();
    }
    
    // Should be bounded to ±20% maximum from baseline
    let final_duty = core.get_learned_duty(4000, 8.0).unwrap();
    assert!(final_duty >= baseline_duty * 0.8);
    assert!(final_duty <= baseline_duty * 1.2);
    
    // Slew rate limiting should prevent rapid changes
    assert!(core.get_max_learning_rate_per_hour() <= 0.05);  // ±5% per hour max
}
```

### Configuration System Testing

#### 6-Parameter Configuration Model
```rust
#[test]
fn test_complete_configuration_model() {
    let config = SystemConfig {
        aggression: 0.7,
        spring_pressure: 5.0,
        max_boost_psi: 9.0,
        overboost_limit: 15.0,
        scramble_enabled: true,
        cold_engine_protection: true,
    };
    
    // All behavior should be derivable from these 6 parameters
    let response_profile = config.get_response_characteristics();
    
    assert!(response_profile.tip_in_sensitivity > 0.0);
    assert!(response_profile.tip_out_decay_rate < 1.0);
    assert!(response_profile.torque_following_gain > 0.0);
    assert!(response_profile.boost_ramp_rate > 0.0);
    
    // Aggression should scale all parameters
    assert!(response_profile.tip_in_sensitivity > config.aggression * 0.01);
    assert!(response_profile.boost_ramp_rate > config.aggression * 1.0);
}

#[test]
fn test_scramble_button_override() {
    let mut hal = MockHal::new();
    let config = SystemConfig {
        aggression: 0.3,  // Conservative setting
        scramble_enabled: true,
        ..test_config()
    };
    let mut core = RumbleDomeCore::new(hal, config);
    
    // Normal operation with low aggression
    core.hal.set_torque_signals(200.0, 175.0);
    core.execute_control_cycle().unwrap();
    let normal_duty = core.get_duty_cycle();
    
    // Scramble button activated
    core.hal.set_scramble_button(true);
    core.execute_control_cycle().unwrap();
    let scramble_duty = core.get_duty_cycle();
    
    // Should behave like 100% aggression regardless of knob setting
    assert!(scramble_duty > normal_duty);
    
    // Scramble button released
    core.hal.set_scramble_button(false);
    core.execute_control_cycle().unwrap();
    
    // Should return to normal aggression behavior
    assert!(core.get_duty_cycle() < scramble_duty);
}
```

## Integration Testing (Desktop Simulation)

### Test Scenario Framework

```rust
// rumbledome-sim/src/scenarios/
pub struct TestScenario {
    pub name: String,
    pub vehicle_model: VehicleSimulation,
    pub environmental_conditions: EnvironmentalSim,
    pub failure_injection: FailureInjection,
    pub expected_outcomes: ExpectedResults,
}

impl TestScenario {
    pub fn aggressive_acceleration_test() -> Self {
        Self {
            name: "Aggressive Acceleration - High Aggression".to_string(),
            vehicle_model: VehicleSimulation::stock_s550_gt(),
            environmental_conditions: EnvironmentalSim::standard_conditions(),
            failure_injection: FailureInjection::none(),
            expected_outcomes: ExpectedResults {
                max_boost_achieved: Some(8.5),
                overboost_violations: 0,
                max_response_time_ms: 100,
                ecu_cooperation_score: 0.9,
            },
        }
    }
    
    pub fn safety_fault_injection_test() -> Self {
        Self {
            name: "Safety Response - Multiple Fault Injection".to_string(),
            vehicle_model: VehicleSimulation::stock_s550_gt(),
            environmental_conditions: EnvironmentalSim::standard_conditions(),
            failure_injection: FailureInjection::multiple([
                FaultType::CanTimeout(300),
                FaultType::SensorFailure(SensorId::ManifoldPressure),
                FaultType::OverboostCondition(12.0),
            ]),
            expected_outcomes: ExpectedResults {
                system_enters_safe_state: true,
                fault_detection_time_ms: Some(100),
                recovery_possible: false,
            },
        }
    }
}
```

### Comprehensive Integration Scenarios

#### Normal Operation Validation
```rust
#[test]
fn test_complete_drive_cycle_simulation() {
    let scenario = TestScenario::complete_drive_cycle();
    let mut sim = DesktopSimulator::new(scenario);
    
    let result = sim.run_full_simulation(Duration::from_secs(300)).unwrap();
    
    // Verify comprehensive operation
    assert!(result.total_boost_cycles > 50);
    assert_eq!(result.safety_violations, 0);
    assert!(result.avg_torque_gap < 15.0);  // Good ECU cooperation
    assert!(result.fuel_economy_impact < 5.0);  // Minimal efficiency impact
}

#[test]
fn test_environmental_variation_handling() {
    let scenarios = [
        EnvironmentalSim::hot_day(40.0),      // 40°C
        EnvironmentalSim::cold_morning(-10.0), // -10°C
        EnvironmentalSim::high_altitude(2000.0), // 2000m elevation
        EnvironmentalSim::humid_conditions(95.0), // 95% humidity
    ];
    
    for env_condition in scenarios {
        let mut scenario = TestScenario::steady_state_cruise();
        scenario.environmental_conditions = env_condition;
        
        let mut sim = DesktopSimulator::new(scenario);
        let result = sim.run_simulation(Duration::from_secs(60)).unwrap();
        
        // System should adapt to all environmental conditions
        assert_eq!(result.safety_violations, 0);
        assert!(result.environmental_compensation_active);
        assert!(result.boost_consistency_score > 0.8);
    }
}
```

#### Safety System Integration Testing
```rust
#[test]
fn test_all_safety_requirements_integration() {
    // Test every SY-* requirement in integrated environment
    let safety_scenarios = [
        TestScenario::pneumatic_failsafe_test(),        // SY-1
        TestScenario::high_authority_recognition_test(), // SY-2  
        TestScenario::overboost_response_test(),         // SY-3
        TestScenario::progressive_calibration_test(),    // SY-4
        TestScenario::torque_ceiling_test(),            // SY-6
        TestScenario::learning_bounds_test(),           // SY-11
        // ... all other SY-* requirements
    ];
    
    for scenario in safety_scenarios {
        let mut sim = DesktopSimulator::new(scenario);
        let result = sim.run_simulation(Duration::from_secs(30)).unwrap();
        
        // Every safety requirement must pass
        assert!(result.safety_requirement_passed);
        assert_eq!(result.safety_violations, 0);
        assert!(result.response_time_ms < 100);
    }
}
```

## CAN Signal Validation Testing

### Ford S550 Signal Interpretation Tests

```rust
// Tests to run with actual vehicle CAN data
#[test] 
fn test_can_signal_decoding_accuracy() {
    let mut can_interface = FordS550Can::new();
    
    // RPM signal validation (0x109)
    let rpm_readings = can_interface.collect_rpm_samples(100);
    for rpm in rpm_readings {
        assert!(rpm > 500);   // Reasonable idle RPM
        assert!(rpm < 8000);  // Reasonable max RPM
    }
    
    // Manifold pressure signal validation (0x167 bytes 5-6)
    let map_readings = can_interface.collect_map_samples(100);
    for map_psi in map_readings {
        assert!(map_psi > -15.0);  // Reasonable vacuum limit
        assert!(map_psi < 30.0);   // Reasonable boost limit
    }
}

#[test]
fn test_torque_signal_behavioral_analysis() {
    // This test requires actual vehicle operation
    let mut can_interface = FordS550Can::new();
    
    // Record signals during acceleration event
    let test_data = can_interface.record_acceleration_event(Duration::from_secs(10));
    
    let signal_a_data = test_data.signal_0x167_torque;  // "Engine load/torque"
    let signal_b_data = test_data.signal_0x43e_load;    // "Engine load percentage"
    
    // Behavioral analysis to determine desired vs actual
    let (a_leads_b, b_leads_a) = analyze_signal_timing(signal_a_data, signal_b_data);
    
    if a_leads_b {
        println!("0x167 appears to be desired torque (leads 0x43E)");
    } else if b_leads_a {
        println!("0x43E appears to be desired torque (leads 0x167)");
    } else {
        println!("Signals show no clear leader/follower relationship");
    }
    
    // Cross-reference with HPTuners data if available
    if let Some(hptuners_data) = test_data.hptuners_reference {
        let correlation_a = correlate_signals(signal_a_data, hptuners_data.desired_torque);
        let correlation_b = correlate_signals(signal_b_data, hptuners_data.desired_torque);
        
        assert!(correlation_a > 0.8 || correlation_b > 0.8);  // One should correlate strongly
    }
}

#[test]
fn test_can_message_update_frequencies() {
    let mut can_interface = FordS550Can::new();
    
    let frequency_test = can_interface.measure_message_frequencies(Duration::from_secs(5));
    
    // Verify control loop requirements
    assert!(frequency_test.rpm_frequency_hz >= 20.0);
    assert!(frequency_test.torque_frequency_hz >= 20.0);  
    assert!(frequency_test.map_frequency_hz >= 20.0);
    
    // Verify consistent timing
    assert!(frequency_test.rpm_jitter_ms < 10.0);
    assert!(frequency_test.torque_jitter_ms < 10.0);
}
```

## Hardware-in-Loop (HIL) Testing

**⚠️ Note**: HIL testing requires building a test rig with pneumatic components, pressure sensors, and controlled air supply. Test rig specifications and feasibility validation are TBD.

### Pneumatic System Validation

```rust
#[test]
fn test_actual_pneumatic_response_times() {
    let mut hil = HardwareInLoopTestRig::new();
    
    // Test SY-3: Overboost response time validation
    hil.set_manifold_pressure(5.0);  // Normal operation
    hil.set_duty_cycle(0.5);          // 50% duty
    std::thread::sleep(Duration::from_millis(100));  // Settle
    
    let start_time = hil.get_timestamp_us();
    hil.trigger_overboost_condition(12.0);  // Above limit
    
    // Measure actual pneumatic response
    let response_time = hil.measure_wastegate_opening_time();
    
    assert!(response_time < 100_000);  // <100ms requirement (microseconds)
    assert!(hil.wastegate_fully_open());
}

#[test]
fn test_dome_pressure_cross_validation() {
    let mut hil = HardwareInLoopTestRig::new();
    
    // Test 4-port MAC solenoid operation
    for duty_cycle in [0.0, 0.25, 0.5, 0.75, 1.0] {
        hil.set_duty_cycle(duty_cycle);
        std::thread::sleep(Duration::from_millis(50));  // Settle
        
        let upper_dome_psi = hil.read_upper_dome_pressure();
        let lower_dome_psi = hil.read_lower_dome_pressure();
        
        if duty_cycle > 0.8 {
            // High duty - upper dome should be pressurized
            assert!(upper_dome_psi > 8.0);
            assert!(lower_dome_psi < 2.0);
        } else if duty_cycle < 0.2 {
            // Low duty - lower dome should be pressurized
            assert!(lower_dome_psi > 8.0);
            assert!(upper_dome_psi < 2.0);
        }
        
        // Should never have both domes pressurized simultaneously
        assert!(!(upper_dome_psi > 8.0 && lower_dome_psi > 8.0));
    }
}
```

## System Integration Testing

### Vehicle Integration Protocol

**Phase 1: Static Validation**
1. **CAN Signal Validation**: Verify Ford S550 signal interpretation with engine off, key on
2. **Pneumatic Checkout**: Test full pneumatic system operation with engine off
3. **Display and Interface**: Validate all user interface elements
4. **Configuration Loading**: Test SD card configuration and learned data handling

**Phase 2: Idle Testing**
1. **Engine Running, No Load**: Validate basic system operation at idle
2. **Safety System Checkout**: Test all fault detection and response systems
3. **Learning System Initialization**: Begin basic calibration data collection

**Phase 3: Controlled Dynamic Testing**
1. **Parking Lot Testing**: Low-speed, low-load boost system validation
2. **Progressive Boost Testing**: Gradually increase boost targets with safety monitoring
3. **Aggression Level Validation**: Test all aggression settings from conservative to aggressive

**Phase 4: Performance Validation** 
1. **Track/Dyno Testing**: Full performance envelope validation in controlled environment
2. **Endurance Testing**: Extended operation validation
3. **Environmental Testing**: Hot/cold weather, altitude testing

### Acceptance Criteria

**Safety Requirements (Must Pass All)**:
- [ ] All SY-* safety requirements validated through automated testing
- [ ] Overboost response time <100ms measured on actual hardware
- [ ] No single point of failure can cause unsafe operation
- [ ] System defaults to safe state (duty=0%) for all fault conditions
- [ ] User cannot configure unsafe parameters through any interface

**Performance Requirements**:
- [ ] Torque-following response provides smooth ECU cooperation
- [ ] No ECU fault codes triggered during normal operation
- [ ] Boost response time competitive with stock turbocharger behavior
- [ ] Learning system converges to stable calibration within 50 miles of driving

**Usability Requirements**:
- [ ] Single aggression knob provides intuitive control across full range
- [ ] System configuration requires <6 parameters total
- [ ] SD card portability enables easy configuration backup/restore
- [ ] All diagnostic information accessible through CLI interface

## Test Coverage Requirements

**Safety-Critical Code**: 100% line coverage required
**Control Logic**: 95% line coverage required  
**HAL Implementations**: 90% line coverage required
**Protocol/CLI**: 85% line coverage required

**Test Execution Strategy**:
- All safety tests must pass before any hardware integration
- Desktop simulation must validate all control scenarios
- CAN signal validation must complete before vehicle integration
- Progressive hardware testing with immediate abort on any safety violation

This comprehensive test plan ensures safe, reliable operation while validating all aspects of the torque-following electronic boost controller.