# RumbleDome Implementation Guide

## 🏗️ Tier 3: Development Support Document

**🔗 Dependencies:** 
- **Tier 1**: Context.md, Requirements.md, Safety.md (system goals and constraints)
- **Tier 2**: TechnicalSpecs.md, Architecture.md, LearnedData.md, Hardware.md (design specifications)
- **Constraints**: Physics.md, CAN_Signals.md (implementation constraints)

**📤 Impacts:** Changes to code structure here may affect:
- **Tier 3**: TestPlan.md (test organization)

## 🔄 Change Impact Checklist
Before modifying this document:
- [ ] **✅ TIER 3 CHANGE**: This affects development workflow and code structure
- [ ] Verify alignment with ALL Tier 2 specifications
- [ ] Check consistency with Tier 1 goals and Safety.md constraints
- [ ] Review Physics.md and CAN_Signals.md constraints
- [ ] Update TestPlan.md if code structure changes affect testing
- [ ] Ensure build process changes don't conflict with TechnicalSpecs.md
- [ ] Update cross-references and file paths

📋 **Technical specifications**: See **[TechnicalSpecs.md](TechnicalSpecs.md)** for hardware platform details  
📖 **Architecture overview**: See **[Architecture.md](Architecture.md)** for system design philosophy  
⚠️ **Safety requirements**: See **[Safety.md](Safety.md)** for all safety-critical implementation requirements

## Implementation Overview

RumbleDome implements **torque-following electronic boost control** through a layered architecture with strict hardware abstraction. The system prioritizes safety, simplicity, and ECU cooperation over maximum performance.

### Core Design Principles
- **Single aggression parameter** scales all system behavior (0.0-1.0)
- **Torque gap analysis** drives boost assistance decisions
- **HAL abstraction** enables desktop simulation and multi-platform support
- **SD card storage** for configuration portability and wear management
- **Fail-safe operation** with multiple safety layers

## Workspace Structure

**🔗 T3-BUILD-001**: **Rust Workspace Architecture**  
**Derived From**: T2-CONTROL-001 (Priority Hierarchy) + Hardware abstraction requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Layered crate structure for HAL abstraction  
**Engineering Rationale**: Workspace isolates hardware-independent core logic for desktop testing  
**AI Traceability**: Drives crate organization, dependency management, build process

```
rumbledome/
├── Cargo.toml                 # Workspace root configuration
├── crates/
│   ├── rumbledome-hal/        # Hardware abstraction layer
│   ├── rumbledome-core/       # Core control logic (hardware-independent)
│   ├── rumbledome-protocol/   # JSON/CLI protocol definitions
│   ├── rumbledome-fw/         # Teensy 4.1 firmware binary
│   ├── rumbledome-sim/        # Desktop simulator and test scenarios
│   └── rumbledome-cli/        # Configuration and diagnostic tool
├── tests/                     # Integration tests
└── docs/                      # Project documentation
```

## Crate Architecture

**🔗 T3-BUILD-002**: **Crate Dependency Structure**  
**Derived From**: T2-HAL-001 (Platform-Independent Hardware Abstraction) + testing requirements  
**Decision Type**: 🔗 **Direct Derivation** - Implementation of HAL abstraction principle  
**AI Traceability**: Drives module boundaries, trait definitions, test isolation

### Dependency Flow
```
rumbledome-fw     ──┐
                    ├─► rumbledome-core ──┐
rumbledome-sim    ──┘                     ├─► rumbledome-hal
                                         ┌┘
rumbledome-cli    ──┐                   ┌┘
                    ├─► rumbledome-protocol
                    └─► rumbledome-core
```

**Key isolation**: `rumbledome-core` has **zero hardware dependencies** - all safety-critical logic can run on desktop for comprehensive testing.

## Core Control Implementation

**🔗 T3-BUILD-003**: **Core Control State Machine**  
**Derived From**: T2-CONTROL-003 (3-Level Control Hierarchy) + fault management requirements  
**Decision Type**: ⚠️ **Engineering Decision** - State machine organization for safety-critical control  
**Engineering Rationale**: State machine ensures predictable behavior during faults and transitions  
**AI Traceability**: Drives control flow, fault handling, safety state transitions

### System State Machine

```rust
// rumbledome-core/src/state.rs
#[derive(Debug, Clone, PartialEq)]
pub enum SystemState {
    Initializing,
    Idle,
    Armed,
    Calibrating(CalibrationProgress),
    OverboostCut,
    Fault(FaultCode),
}

pub struct RumbleDomeCore<H: HalTrait> {
    state: SystemState,
    config: SystemConfig,           // 5 parameters total
    learned_data: LearnedData,      // See LearnedData.md
    torque_following: TorqueFollowing,
    safety_monitor: SafetyMonitor,
    calibration: AutoCalibration,
    hal: H,
}
```

### Configuration Structure (Simplified)

**🔗 T3-BUILD-004**: **5-Parameter Configuration Implementation**  
**Derived From**: T2-HAL-003 (5-Parameter Configuration Structure) + storage requirements  
**Decision Type**: 🔗 **Direct Derivation** - Software implementation of single-knob philosophy  
**AI Traceability**: Drives configuration serialization, parameter validation, UI consistency

```rust
// rumbledome-core/src/config.rs - Only 5 user parameters!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    // User-controlled parameters (stored on SD card)
    pub aggression: f32,              // 0.0-1.0 - scales all behavior
    pub spring_pressure: f32,         // PSI - wastegate spring pressure
    pub max_boost_psi: f32,          // PSI - performance ceiling
    pub overboost_limit: f32,        // PSI - hard safety limit
    pub scramble_enabled: bool,       // Enable scramble button
    
    // System-derived parameters (not user-configurable)
    pub torque_target_percentage: f32, // Always 95% - derived from safety requirements
    pub boost_slew_rate: f32,         // Aggression-scaled slew limiting
}

impl SystemConfig {
    // All complex behavior derived from single aggression parameter
    pub fn get_response_characteristics(&self) -> ResponseProfile {
        ResponseProfile {
            tip_in_sensitivity: self.aggression * 2.0,
            tip_out_decay_rate: self.aggression * 0.5 + 0.2,
            torque_following_gain: self.aggression * 1.5 + 0.3,
            boost_ramp_rate: self.aggression * 3.0 + 1.0,
        }
    }
}
```

## Torque-Following Control Loop

**🔗 T3-BUILD-005**: **3-Level Control Hierarchy Implementation**  
**Derived From**: T2-CONTROL-003 (3-Level Control Hierarchy) + T2-CONTROL-004 (Torque-Based Targeting)  
**Decision Type**: 🔗 **Direct Derivation** - Software implementation of torque-following control  
**AI Traceability**: Drives control algorithms, torque analysis, ECU cooperation logic

### 3-Level Control Hierarchy Implementation

```rust
// rumbledome-core/src/control/mod.rs
impl<H: HalTrait> RumbleDomeCore<H> {
    pub fn execute_control_cycle(&mut self) -> Result<(), CoreError> {
        let inputs = self.read_system_inputs()?;
        self.safety_monitor.validate_inputs(&inputs)?;
        
        // LEVEL 1: Torque Gap Analysis
        let torque_gap = inputs.desired_torque - inputs.actual_torque;
        let assistance_needed = self.analyze_torque_assistance_need(torque_gap, &inputs)?;
        
        // LEVEL 2: Boost Assistance Calculation  
        let target_boost = if assistance_needed {
            self.calculate_boost_assistance(torque_gap, &inputs)?
        } else {
            self.get_baseline_boost(&inputs)?  // Minimal assistance
        };
        
        // LEVEL 3: Safety-Validated Output
        let target_duty = self.boost_to_duty_conversion(target_boost, &inputs)?;
        let safe_duty = self.safety_monitor.validate_and_limit(target_duty, &inputs)?;
        let final_duty = self.apply_aggression_scaling(safe_duty)?;
        
        self.update_output(final_duty)?;
        self.update_learning_system(&inputs, final_duty)?;
        
        Ok(())
    }
    
    fn analyze_torque_assistance_need(&self, torque_gap: f32, inputs: &SystemInputs) -> Result<bool, CoreError> {
        let torque_ceiling = inputs.desired_torque * 0.95;  // Always 95%
        
        // ECU struggling to achieve torque target?
        if torque_gap > 10.0 {  // Nm threshold
            return Ok(true);    // Provide assistance
        }
        
        // Approaching ECU torque ceiling? 
        if inputs.actual_torque > torque_ceiling {
            return Ok(false);   // Back off to prevent ECU intervention
        }
        
        // Normal operation - minimal assistance
        Ok(false)
    }
}
```

### Learning System Integration

```rust
// rumbledome-core/src/learning/mod.rs
pub struct LearnedData {
    pub duty_calibration: DutyCalibrationMap,      // RPM×Boost → Duty cycle
    pub environmental: EnvironmentalFactors,       // Temperature/altitude compensation  
    pub sensor_fusion: SensorCrossCal,           // CAN MAP vs boost gauge calibration
    pub safety_parameters: SafetyCharacteristics, // Response time validation
}

impl LearnedData {
    pub fn lookup_baseline_duty(&self, rpm: u16, boost_psi: f32) -> Result<f32, LearningError> {
        let base_duty = self.duty_calibration.interpolate(rpm, boost_psi)?;
        let env_adjusted = self.environmental.compensate(base_duty)?;
        Ok(env_adjusted.clamp(0.0, 1.0))
    }
    
    pub fn update_from_operation(&mut self, inputs: &SystemInputs, duty: f32) -> Result<(), LearningError> {
        // Update duty cycle calibration
        self.duty_calibration.learn_point(inputs.rpm, inputs.manifold_pressure, duty)?;
        
        // Update environmental compensation
        self.environmental.adapt_to_conditions(&inputs)?;
        
        // Update sensor cross-calibration
        if let (Some(can_map), Some(boost_gauge)) = (inputs.can_map, inputs.boost_gauge) {
            self.sensor_fusion.update_cross_calibration(can_map, boost_gauge)?;
        }
        
        Ok(())
    }
}
```

## Hardware Abstraction Layer

### HAL Trait Definition

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
}

// CAN abstraction for Ford S550 signals
pub trait Can {
    fn read_rpm(&mut self) -> Result<u16, HalError>;
    fn read_torque_signals(&mut self) -> Result<TorqueData, HalError>;
    fn read_manifold_pressure(&mut self) -> Result<f32, HalError>;
}

#[derive(Debug, Clone)]
pub struct TorqueData {
    pub engine_load_torque: f32,  // From 0x167 - need to determine if desired or actual
    pub engine_load_percent: f32, // From 0x43E - alternative torque signal
    pub timestamp_ms: u32,
}
```

### Platform Implementations

```rust
// rumbledome-hal/src/teensy41/mod.rs
pub struct Teensy41Hal {
    can: Teensy41Can,
    analog: Teensy41Analog,
    storage: SdCardStorage,  // MicroSD instead of EEPROM
    display: St7735Display,
    pwm: FlexPwm,
}

impl Can for Teensy41Can {
    fn read_torque_signals(&mut self) -> Result<TorqueData, HalError> {
        // Read Ford S550 CAN messages
        let msg_167 = self.read_can_message(0x167)?;  // Engine load/torque + MAP
        let msg_43e = self.read_can_message(0x43E)?;  // Engine load percentage
        
        Ok(TorqueData {
            engine_load_torque: self.decode_167_torque(&msg_167)?,
            engine_load_percent: self.decode_43e_load(&msg_43e)?, 
            timestamp_ms: self.get_timestamp_ms(),
        })
    }
    
    fn decode_167_torque(&self, msg: &CanMessage) -> Result<f32, HalError> {
        // Implementation: ((b1-128)<<8 + b2) / 4
        let raw = ((msg.data[1] as u16 - 128) << 8) + msg.data[2] as u16;
        Ok(raw as f32 / 4.0)
    }
}

// rumbledome-hal/src/mock/mod.rs  
pub struct MockHal {
    scenario: TestScenario,
    mock_time: MockTime,
    mock_sensors: MockSensors,
    mock_can: MockCan,
}

impl Can for MockCan {
    fn read_torque_signals(&mut self) -> Result<TorqueData, HalError> {
        // Simulate realistic torque signal behavior for testing
        Ok(self.scenario.get_current_torque_data())
    }
}
```

## Safety System Implementation

### Multi-Layer Safety Architecture

```rust
// rumbledome-core/src/safety/mod.rs
pub struct SafetyMonitor {
    overboost_detector: OverboostDetector,
    response_validator: ResponseTimeValidator,
    fault_manager: FaultManager,
    pneumatic_monitor: PneumaticHealthMonitor,
}

impl SafetyMonitor {
    pub fn validate_and_limit(&mut self, target_duty: f32, inputs: &SystemInputs) -> Result<f32, SafetyError> {
        // SY-1: Overboost protection (highest priority)
        if inputs.manifold_pressure > inputs.config.overboost_limit {
            self.fault_manager.trigger_overboost_fault()?;
            return Ok(0.0);  // Immediate duty=0% for wastegate open
        }
        
        // SY-2: Pneumatic response validation
        self.response_validator.validate_dome_response(&inputs.dome_pressures)?;
        
        // SY-3: Progressive safety limits
        let conservative_limit = self.calculate_safe_duty_ceiling(inputs)?;
        let limited_duty = target_duty.min(conservative_limit);
        
        // SY-4: Slew rate limiting (aggression-scaled)
        let slew_limited = self.apply_slew_limits(limited_duty, inputs.config.aggression)?;
        
        Ok(slew_limited)
    }
    
    fn calculate_safe_duty_ceiling(&self, inputs: &SystemInputs) -> Result<f32, SafetyError> {
        // Dynamic safety ceiling based on pneumatic system health
        let base_ceiling = 0.8;  // 80% max duty for safety margin
        let health_factor = inputs.dome_pressures.calculate_health_factor();
        Ok(base_ceiling * health_factor)
    }
}
```

## Auto-Calibration System

### Progressive Calibration Implementation

```rust
// rumbledome-core/src/calibration/mod.rs
pub struct AutoCalibration {
    state: CalibrationState,
    safety_validator: CalibrationSafetyValidator,
    learning_session: LearningSession,
}

#[derive(Debug, Clone)]
pub enum CalibrationState {
    Inactive,
    Phase1Conservative { runs_completed: u8, target_boost: f32 },
    Phase2Progressive { current_limit: f32, confidence: f32 },
    Complete { final_overboost_limit: f32 },
}

impl AutoCalibration {
    pub fn start_calibration_session(&mut self, config: &SystemConfig) -> Result<(), CalibrationError> {
        // SY-4: Start at spring + 1 psi for ultra-conservative safety
        let initial_limit = config.spring_pressure + 1.0;
        
        self.state = CalibrationState::Phase1Conservative {
            runs_completed: 0,
            target_boost: initial_limit - 0.5,  // Target below limit
        };
        
        // Update runtime safety limits
        self.safety_validator.set_calibration_limits(initial_limit)?;
        Ok(())
    }
    
    pub fn process_calibration_run(&mut self, result: CalibrationResult) -> CalibrationAction {
        match &mut self.state {
            CalibrationState::Phase1Conservative { runs_completed, .. } => {
                *runs_completed += 1;
                
                // Validate safety response time during run
                if !result.safety_response_acceptable() {
                    return CalibrationAction::AbortDueToSafetyResponse;
                }
                
                if *runs_completed >= 3 && result.consistency_check_passed() {
                    CalibrationAction::AdvanceToProgressive
                } else {
                    CalibrationAction::RepeatRun
                }
            },
            CalibrationState::Phase2Progressive { current_limit, confidence } => {
                if result.target_achieved_safely() {
                    *confidence += 0.1;
                    *current_limit += 0.2;  // Gradual limit increases
                    
                    if *confidence > 0.8 {
                        CalibrationAction::CompleteCalibration(*current_limit)
                    } else {
                        CalibrationAction::ContinueProgressive
                    }
                } else {
                    CalibrationAction::RollbackToSafeLimit
                }
            },
            _ => CalibrationAction::NoAction,
        }
    }
}
```

## Storage Architecture

### SD Card-Based Configuration

```rust
// rumbledome-hal/src/storage/sd_card.rs
pub struct SdCardStorage {
    card: SdCard,
    filesystem: Fat32,
    write_debouncer: WriteDebouncer,  // 5-10 second debouncing
}

pub struct PortableConfiguration {
    // Only 5 user-configurable parameters
    pub aggression: f32,
    pub spring_pressure: f32, 
    pub max_boost_psi: f32,
    pub overboost_limit: f32,
    pub scramble_enabled: bool,
}

impl Storage for SdCardStorage {
    fn save_configuration(&mut self, config: &PortableConfiguration) -> Result<(), StorageError> {
        // Atomic write using temp file + rename
        let temp_file = "/RUMBLEDOME/config/.user_config_temp.json";
        let final_file = "/RUMBLEDOME/config/user_config.json";
        
        // Debounce writes to extend SD card life
        self.write_debouncer.schedule_write(|| {
            self.filesystem.write_file(temp_file, &config.serialize()?)?;
            self.filesystem.rename(temp_file, final_file)?;
            Ok(())
        })
    }
    
    fn load_learned_data(&mut self) -> Result<LearnedData, StorageError> {
        // Learned data stored in binary format for efficiency
        let calibration = self.filesystem.read_file("/RUMBLEDOME/learned/calibration_maps.bin")?;
        let environmental = self.filesystem.read_json("/RUMBLEDOME/learned/environmental.json")?;
        let sensor_fusion = self.filesystem.read_json("/RUMBLEDOME/learned/sensor_fusion.json")?;
        let safety_params = self.filesystem.read_json("/RUMBLEDOME/learned/safety_params.json")?;
        
        Ok(LearnedData::from_components(calibration, environmental, sensor_fusion, safety_params)?)
    }
}
```

## Testing Strategy

### Desktop Simulation Framework

```rust
// rumbledome-sim/src/scenarios/mod.rs
pub struct TestScenario {
    name: String,
    vehicle_model: VehicleSimulation,
    environmental_conditions: EnvironmentalSim,
    failure_injection: FailureInjection,
}

pub struct VehicleSimulation {
    engine: EngineModel,
    turbo: TurboModel, 
    wastegate: WastegateModel,
    ecu_torque_model: EcuTorqueModel,
}

impl TestScenario {
    pub fn overboost_response_test() -> Self {
        Self {
            name: "Overboost Response Validation".to_string(),
            vehicle_model: VehicleSimulation::stock_s550(),
            environmental_conditions: EnvironmentalSim::standard(),
            failure_injection: FailureInjection::none(),
        }
    }
    
    pub fn run_scenario(&mut self, core: &mut RumbleDomeCore<MockHal>) -> TestResult {
        // 1. Initialize to normal operation
        for _ in 0..100 { core.execute_control_cycle()?; }
        
        // 2. Inject overboost condition
        self.vehicle_model.set_manifold_pressure(12.0);  // Above 10 psi limit
        
        // 3. Verify safety response
        core.execute_control_cycle()?;
        
        TestResult {
            response_time_ms: self.measure_response_time(),
            final_duty_cycle: core.get_current_duty_cycle(),
            system_state: core.get_system_state(),
            safety_triggered: core.safety_monitor.get_active_faults().len() > 0,
        }
    }
}
```

### Unit Test Structure

```rust
// rumbledome-core/tests/safety_tests.rs
#[cfg(test)]
mod safety_tests {
    use super::*;
    use rumbledome_hal::mock::MockHal;
    
    #[test]
    fn test_overboost_immediate_response() {
        let mut hal = MockHal::new();
        let mut core = RumbleDomeCore::new(hal, test_config());
        
        // Set normal operation
        core.hal.set_manifold_pressure(5.0);  // Normal
        core.execute_control_cycle().unwrap();
        assert!(core.get_duty_cycle() > 0.0);
        
        // Trigger overboost
        core.hal.set_manifold_pressure(12.0);  // Above 10 psi limit
        core.execute_control_cycle().unwrap();
        
        // Verify immediate response
        assert_eq!(core.get_duty_cycle(), 0.0);  // Should cut to 0%
        assert_eq!(core.get_system_state(), SystemState::OverboostCut);
    }
    
    #[test]
    fn test_torque_cooperation_behavior() {
        let mut hal = MockHal::new();
        let mut core = RumbleDomeCore::new(hal, test_config());
        
        // ECU achieving torque target - should provide minimal assistance
        core.hal.set_torque_signals(200.0, 195.0);  // 5 Nm gap - acceptable
        core.execute_control_cycle().unwrap();
        assert!(core.get_duty_cycle() < 0.3);  // Minimal boost assistance
        
        // ECU struggling with torque - should provide assistance
        core.hal.set_torque_signals(200.0, 175.0);  // 25 Nm gap - needs help
        core.execute_control_cycle().unwrap();
        assert!(core.get_duty_cycle() > 0.5);  // Increased boost assistance
        
        // ECU at torque ceiling - should back off
        core.hal.set_torque_signals(200.0, 195.0);  // Back to normal
        core.execute_control_cycle().unwrap();
        // Duty cycle should decrease from previous high value
    }
}
```

## Build System Configuration

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
defmt = "0.3"           # Embedded logging
embedded-hal = "0.2"    # Hardware abstraction
nb = "1.0"
```

### Platform-Specific Builds

```toml
# rumbledome-fw/Cargo.toml
[package]
name = "rumbledome-fw"
version.workspace = true

[dependencies]
rumbledome-core = { path = "../rumbledome-core" }
rumbledome-hal = { path = "../rumbledome-hal", features = ["teensy41"] }
rumbledome-protocol = { path = "../rumbledome-protocol" }

cortex-m = "0.7"
cortex-m-rt = "0.7"
teensy4-bsp = "0.4"     # Teensy 4.x board support

[target.thumbv7em-none-eabihf]
runner = "teensy_loader_cli --mcu=TEENSY41 -w"
rustflags = ["-C", "link-arg=-Tlink.x"]
```

### Development Commands

```bash
# Desktop simulation and testing
cargo test --workspace                           # All unit tests
cargo run -p rumbledome-sim --release           # Desktop simulator
cargo run -p rumbledome-cli -- status           # CLI tool

# Embedded development  
cargo check --target thumbv7em-none-eabihf     # Check embedded build
cargo build -p rumbledome-fw --release --target thumbv7em-none-eabihf
teensy_loader_cli --mcu=TEENSY41 -w target/thumbv7em-none-eabihf/release/rumbledome-fw.hex

# Code quality
cargo clippy --workspace                        # Linting
cargo fmt --workspace                          # Formatting  
cargo doc --workspace --open                   # Documentation
```

## Development Workflow

### Pre-Implementation Validation
1. **Safety requirements review** - Verify all SY-* requirements are covered
2. **Desktop simulation** - Validate control logic without hardware
3. **HAL mock testing** - Ensure platform independence
4. **Integration test coverage** - All safety scenarios tested

### Implementation Priority
1. **Core control logic** - Torque-following without hardware dependencies
2. **Safety monitoring** - All overboost and fault detection logic  
3. **HAL implementation** - Teensy 4.1 platform support
4. **Learning system** - Calibration and environmental adaptation
5. **CLI and protocol** - Configuration and diagnostic interface

### Testing Strategy
- **Unit tests**: All safety-critical logic with 100% coverage
- **Integration tests**: Full system scenarios with mock hardware
- **Hardware-in-loop**: Real pneumatic system validation before deployment
- **Regression tests**: Automated testing of all control scenarios

This implementation approach prioritizes **safety validation** and **platform independence** while maintaining the simplified user interface and ECU-cooperative control philosophy.