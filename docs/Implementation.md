# RumbleDome Implementation Guide

## Workspace Structure

```
rumbledome/
├── Cargo.toml                 # Workspace root configuration
├── crates/
│   ├── rumbledome-hal/        # Hardware abstraction layer
│   ├── rumbledome-core/       # Core control logic (no hardware deps)
│   ├── rumbledome-protocol/   # JSON/CLI protocol definitions
│   ├── rumbledome-fw/         # Teensy 4.1 firmware binary
│   ├── rumbledome-sim/        # Desktop simulator and test scenarios
│   └── rumbledome-cli/        # Configuration tool
└── docs/                      # Project documentation
```

## Crate Dependencies

### Dependency Graph
```
rumbledome-fw     ──┐
                    ├─► rumbledome-core ──┐
rumbledome-sim    ──┘                     ├─► rumbledome-hal
                                         ┌┘
rumbledome-cli    ──┐                   ┌┘
                    ├─► rumbledome-protocol
                    └─► rumbledome-core
```

### Version Management
- **Workspace-level version**: Single version for all crates
- **Semantic versioning**: Major.Minor.Patch according to API changes
- **Feature flags**: Platform-specific features controlled via Cargo features

## Core Implementation Architecture

### State Machine (rumbledome-core)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SystemState {
    Initializing,
    Idle,
    Armed,
    Calibrating(CalibrationState),
    OverboostCut,
    Fault(FaultCode),
}

pub struct RumbleDomeCore<H: HalTrait> {
    state: SystemState,
    config: SystemConfig,
    learned_data: LearnedData,
    control_loop: ControlLoop,
    safety_monitor: SafetyMonitor,
    calibration: AutoCalibration,
    pneumatic_optimizer: PneumaticOptimizer,
    hal: H,
}
```

### Control Loop Implementation (3-Level Hierarchy)

```rust
impl<H: HalTrait> RumbleDomeCore<H> {
    pub fn execute_control_cycle(&mut self) -> Result<(), CoreError> {
        // Input Processing
        let inputs = self.read_inputs()?; // Sensors + CAN (torque, RPM, MAP)
        self.validate_inputs(&inputs)?;
        
        // LEVEL 1: Torque-Based Boost Target Adjustment
        let torque_error = inputs.desired_torque - inputs.actual_torque;
        let base_boost_target = self.get_profile_boost_target(inputs.rpm)?;
        let adjusted_boost_target = self.modulate_boost_for_torque_gap(
            base_boost_target, 
            torque_error,
            inputs.desired_torque
        )?;
        
        // LEVEL 2: Precise Boost Delivery (PID + Learned)
        let learned_baseline_duty = self.lookup_learned_duty(adjusted_boost_target, &inputs)?;
        let boost_error = adjusted_boost_target - inputs.manifold_pressure_gauge;
        let pid_adjustment = self.pid_controller.update(boost_error)?;
        let target_duty = learned_baseline_duty + pid_adjustment;
        
        // LEVEL 3: Safety and Output
        let safe_duty = self.apply_safety_overrides(target_duty, &inputs)?;
        let slew_limited_duty = self.apply_slew_limits(safe_duty)?;
        
        self.update_solenoid_output(slew_limited_duty)?;
        self.update_learning_data(&inputs, slew_limited_duty)?;
        self.update_system_state(&inputs)?;
        
        Ok(())
    }
    
    fn modulate_boost_for_torque_gap(&self, base_target: f32, torque_error: f32, desired_torque: f32) -> Result<f32, CoreError> {
        let torque_target_threshold = desired_torque * self.config.torque_target_percentage / 100.0;
        
        if torque_error > 10.0 {
            // Large gap - ECU needs help, increase boost target slightly
            Ok((base_target * 1.05).min(self.get_profile_max_boost()))
        } else if inputs.actual_torque > torque_target_threshold {
            // Approaching ceiling - back off to prevent ECU intervention  
            Ok(base_target * 0.95)
        } else {
            // ECU satisfied - maintain current boost target
            Ok(base_target)
        }
    }
}
```

## HAL Implementation Strategy

### Platform Abstraction

```rust
// rumbledome-hal/src/traits.rs
pub trait HalTrait: Send + Sync {
    type Time: Time;
    type Pwm: Pwm;
    type Analog: Analog;
    type Storage: Storage;
    type Can: Can;
    type Display: Display;
    type Gpio: Gpio;
    type Logger: Logger;
    type FaultReporter: FaultReporter;
    type Watchdog: Watchdog;
}

// Platform-specific implementations
#[cfg(feature = "teensy41")]
pub mod teensy41;

#[cfg(feature = "sim")]
pub mod mock;
```

### Teensy 4.1 Implementation

```rust
// rumbledome-hal/src/teensy41/mod.rs
pub struct Teensy41Hal {
    pwm: Teensy41Pwm,
    analog: Teensy41Analog,
    storage: Teensy41Storage,
    can: Teensy41Can,
    display: Teensy41Display,
    gpio: Teensy41Gpio,
}

impl HalTrait for Teensy41Hal {
    type Time = Teensy41Time;
    type Pwm = Teensy41Pwm;
    // ... other associated types
}
```

### Mock Implementation for Testing

```rust
// rumbledome-hal/src/mock/mod.rs
pub struct MockHal {
    // Test scenario configuration
    scenario: TestScenario,
    // Simulated sensor values
    sensors: MockSensors,
    // CAN message simulation
    can_sim: MockCan,
}

impl HalTrait for MockHal {
    type Time = MockTime;
    type Pwm = MockPwm;
    // ... other mock implementations
}
```

## Data Structure Implementation

### Configuration Management

```rust
// rumbledome-core/src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub spring_pressure: f32,
    pub profiles: HashMap<String, BoostProfile>,
    pub active_profile: String,
    pub scramble_profile: String,
    pub torque_target_percentage: f32,
    pub boost_slew_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoostProfile {
    pub name: String,
    pub boost_targets: Vec<(u16, f32)>, // (RPM, PSI)
    pub overboost_limit: f32,
    pub overboost_hysteresis: f32,
}
```

### Learning Data Management

```rust
// rumbledome-core/src/learning.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedData {
    pub calibration_map: CalibrationMap,
    pub environmental_factors: EnvironmentalFactors,
    pub confidence_metrics: ConfidenceMetrics,
    pub last_updated: SystemTime,
}

pub struct CalibrationMap {
    // 2D interpolation table: (RPM, Boost) -> Duty Cycle
    pub duty_map: InterpolationTable2D,
    pub bounds: DutyCycleBounds,
}

#[derive(Debug, Clone)]
pub struct EnvironmentalFactors {
    pub temperature_compensation: f32,
    pub altitude_compensation: f32,
    pub supply_pressure_baseline: f32,
}
```

### Safety System Implementation

```rust
// rumbledome-core/src/safety.rs
pub struct SafetyMonitor {
    overboost_detector: OverboostDetector,
    fault_manager: FaultManager,
    response_validator: ResponseValidator,
}

impl SafetyMonitor {
    pub fn check_safety(&mut self, inputs: &SystemInputs, duty_cycle: f32) -> Result<f32, SafetyError> {
        // Check overboost conditions
        self.overboost_detector.check(inputs.manifold_pressure)?;
        
        // Validate pneumatic response capability
        self.response_validator.validate_response_time(inputs.dome_pressures)?;
        
        // Apply progressive limits
        let safe_duty = self.apply_progressive_limits(duty_cycle, inputs)?;
        
        Ok(safe_duty)
    }
}
```

## Auto-Calibration Implementation

### Progressive Calibration Strategy

```rust
// rumbledome-core/src/calibration.rs
pub struct AutoCalibration {
    state: CalibrationState,
    current_limits: ProgressiveLimits,
    learning_session: LearningSession,
}

#[derive(Debug, Clone)]
pub enum CalibrationState {
    Inactive,
    Conservative { target_rpm: u16, target_boost: f32, runs_completed: u8 },
    Progressive { current_limit: f32, confidence: f32 },
    Complete,
}

impl AutoCalibration {
    pub fn start_session(&mut self, target: CalibrationTarget) -> Result<(), CalibrationError> {
        // Start with spring + 1 psi overboost limit
        let initial_limit = target.spring_pressure + 1.0;
        
        self.state = CalibrationState::Conservative {
            target_rpm: target.rpm,
            target_boost: target.boost,
            runs_completed: 0,
        };
        
        self.current_limits.overboost_limit = initial_limit;
        Ok(())
    }
    
    pub fn process_run_result(&mut self, result: CalibrationRunResult) -> CalibrationAction {
        match &mut self.state {
            CalibrationState::Conservative { runs_completed, .. } => {
                *runs_completed += 1;
                if *runs_completed >= 3 && result.consistency_check() {
                    CalibrationAction::AdvanceToProgressive
                } else {
                    CalibrationAction::RepeatRun
                }
            },
            // ... other states
        }
    }
}
```

## Testing Strategy

### Unit Testing Structure

```rust
// rumbledome-core/tests/control_loop_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use rumbledome_hal::mock::MockHal;
    
    #[test]
    fn test_overboost_response() {
        let mut hal = MockHal::new();
        hal.set_manifold_pressure(10.0); // Above limit
        
        let mut core = RumbleDomeCore::new(hal);
        core.execute_control_cycle().unwrap();
        
        assert_eq!(core.get_duty_cycle(), 0.0); // Should cut to 0%
        assert_eq!(core.get_state(), SystemState::OverboostCut);
    }
    
    #[test]
    fn test_torque_cooperation() {
        let mut hal = MockHal::new();
        hal.set_torque_signals(200.0, 190.0); // Desired: 200, Actual: 190
        
        let mut core = RumbleDomeCore::new(hal);
        core.execute_control_cycle().unwrap();
        
        // Should target 95% of 200 = 190 Nm (already achieved)
        assert!(core.get_duty_cycle() < 0.1); // Minimal adjustment needed
    }
}
```

### Integration Testing

```rust
// rumbledome-core/tests/integration_tests.rs
#[test]
fn test_full_calibration_cycle() {
    let mut hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal);
    
    // Start calibration at 4000 RPM, 8.0 PSI target
    core.start_calibration(4000, 8.0).unwrap();
    
    // Simulate multiple WOT runs
    for run in 0..5 {
        hal.simulate_wot_run(4000, 8.0);
        
        // Execute multiple control cycles
        for _ in 0..100 {
            core.execute_control_cycle().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    
    // Verify calibration completed successfully  
    assert_eq!(core.get_calibration_state(), CalibrationState::Complete);
    assert!(core.get_learned_duty(4000, 8.0).unwrap() > 0.0);
}
```

## Build Configuration

### Workspace Cargo.toml

```toml
[workspace]
members = [
    "crates/rumbledome-hal",
    "crates/rumbledome-core", 
    "crates/rumbledome-protocol",
    "crates/rumbledome-fw",
    "crates/rumbledome-sim",
    "crates/rumbledome-cli"
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
authors = ["RumbleDome Team"]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
log = "0.4"
```

### Platform-Specific Features

```toml
# rumbledome-fw/Cargo.toml
[package]
name = "rumbledome-fw"
version.workspace = true

[dependencies]
rumbledome-core = { path = "../rumbledome-core" }
rumbledome-hal = { path = "../rumbledome-hal", features = ["teensy41"] }

[target.thumbv7em-none-eabihf]
runner = "teensy_loader_cli --mcu=TEENSY41 -w"
```

## Deployment and Flashing

### Firmware Build Process

```bash
# Build for Teensy 4.1
cd crates/rumbledome-fw
cargo build --release --target thumbv7em-none-eabihf

# Flash to device
teensy_loader_cli --mcu=TEENSY41 -w target/thumbv7em-none-eabihf/release/rumbledome-fw.hex
```

### Desktop Simulation

```bash
# Run desktop simulator
cd crates/rumbledome-sim
cargo run --release

# Run with specific test scenario
cargo run --release -- --scenario overboost_test
```

### Configuration Tool

```bash
# Build CLI configuration tool
cd crates/rumbledome-cli
cargo build --release

# Connect to device and configure
./target/release/rumbledome-cli --port /dev/ttyUSB0 configure --profile daily
```

## Development Workflow

### Pre-commit Validation
1. Run all unit tests: `cargo test --workspace`
2. Run safety-critical integration tests
3. Lint and format: `cargo clippy && cargo fmt`
4. Check embedded target builds: `cargo check --target thumbv7em-none-eabihf`

### Continuous Integration
- Automated testing on all platforms
- Safety requirement validation
- Hardware-in-loop testing for releases
- Documentation generation and deployment

### Release Process
1. Version bump across workspace
2. Full test suite execution
3. Hardware validation testing
4. Documentation updates  
5. Release tag and artifact publication