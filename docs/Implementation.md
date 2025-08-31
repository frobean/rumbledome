# RumbleDome Implementation Guide

## Recent Implementation Progress

### ✅ PWM Synchronization System (Jan 2025)
Implemented comprehensive PWM-synchronized control loop timing to prevent phase noise in dome pressure control:

- **PWM frequency increased**: 30Hz → 100Hz for better solenoid response
- **Multiple sync strategies**: CycleStart, CycleMidpoint, SubCycle timing options
- **Jitter reduction**: Deadband filtering using FlexPWM resolution (0.003% duty cycle)
- **Beat frequency detection**: Prevents harmonics between control and PWM frequencies
- **PWM-aware slew limiting**: Coordinated output changes during synchronized updates

### ✅ EEPROM/NVM Storage Implementation (Jan 2025)
Completed non-volatile storage HAL from placeholder to full implementation:

- **4KB FlexRAM EEPROM**: Using i.MX RT1062 hardware emulation
- **Immediate writes**: Automotive-grade persistence (no graceful shutdown dependency)
- **Comprehensive wear tracking**: 8-region monitoring with health status
- **Storage sections**: Config (512B), Learned Data (2KB), Calibration (1KB), Safety Log (512B)
- **Complete test suite**: Storage operations, wear tracking, section management

### ✅ Voltage Divider Support (Jan 2025)
Updated sensor calibration for 5V pressure sensors with 3.3V ADC input:

- **10kΩ+20kΩ resistor configuration**: 0.333 voltage ratio for faster ADC reads
- **Voltage scaling**: 0.5V-4.5V → 0.167V-1.5V (optimized divider ratio)
- **Updated scale factor**: 22.56 PSI/V (30 PSI ÷ 1.33V span)
- **Resolution**: 0.018 PSI with 12-bit ADC (more than adequate for boost control)
- **Formula**: `PSI = ((Vout - 0.167) / 1.33) * 30`
- **Documentation updated**: Context.md, TestPlan.md, all calibration defaults

### ✅ MAP Sensor Fusion & Cross-Calibration (Jan 2025)
Implemented intelligent sensor fusion for full vacuum-to-boost manifold pressure range:
- **Dual sensor approach**: CAN MAP (vacuum, 0-1 bar) + boost gauge (positive, 0-30 PSI)
- **Automatic cross-calibration**: Learns systematic offset between sensors in overlap zone
- **Dynamic compensation**: 1% learning rate with exponential moving average
- **Seamless operation**: No faults for sensor disagreement - system adapts continuously
- **Persistent learning**: Cross-calibration stored in EEPROM with environmental factors
- **Transition zone blending**: Weighted sensor fusion around atmospheric pressure
- **Atmospheric compensation**: Automatic baseline tracking for altitude/weather changes

## Workspace Structure

```
rumbledome/
├── Cargo.toml                 # Workspace root configuration
├── crates/
│   ├── rumbledome-hal/        # Hardware abstraction layer ✅
│   ├── rumbledome-core/       # Core control logic (no hardware deps) ✅
│   ├── rumbledome-protocol/   # JSON/CLI protocol definitions ✅
│   ├── rumbledome-fw/         # Teensy 4.1 firmware binary ⚠️
│   ├── rumbledome-sim/        # Desktop simulator and test scenarios ❌
│   └── rumbledome-cli/        # Configuration tool ❌
├── tests/                     # Integration tests ✅
└── docs/                      # Project documentation ✅
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

# Storage health monitoring commands
./target/release/rumbledome-cli --port /dev/ttyUSB0 status --storage-health
./target/release/rumbledome-cli --port /dev/ttyUSB0 diagnostics --eeprom-report
./target/release/rumbledome-cli --port /dev/ttyUSB0 diagnostics --wear-tracking

# Backup and restore commands for microcontroller replacement
./target/release/rumbledome-cli backup --port /dev/ttyUSB0 --output my_ebc_backup.json
./target/release/rumbledome-cli restore --port /dev/ttyUSB0 --backup-file my_ebc_backup.json
./target/release/rumbledome-cli verify my_ebc_backup.json
./target/release/rumbledome-cli list-backups --directory ./backups/

# MicroSD card configuration management
./target/release/rumbledome-cli sd-card list-profiles
./target/release/rumbledome-cli sd-card load-profile daily_driver
./target/release/rumbledome-cli sd-card save-profile --name track_day --description "Aggressive track tune"
./target/release/rumbledome-cli sd-card backup --output sd_backup.json
./target/release/rumbledome-cli sd-card restore --backup-file sd_backup.json

# Bluetooth connection examples (same commands as USB-C)
./target/release/rumbledome-cli --bluetooth status --storage-health
./target/release/rumbledome-cli --bluetooth backup --output mobile_backup.json
./target/release/rumbledome-cli --bluetooth config --profile sport_mode
```

## Storage Health Monitoring Implementation

### Automotive Storage Reality

Unlike desktop applications, automotive ECUs experience abrupt power loss when the ignition is turned off. This requires a fundamentally different approach to data persistence:

```rust
// rumbledome-hal/src/teensy41/storage.rs

// CRITICAL: Immediate write-through strategy for automotive environment
impl NonVolatileStorage for Teensy41Storage {
    fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), HalError> {
        // Update cache for fast reads
        self.eeprom_cache[offset..offset + write_len].copy_from_slice(&data[..write_len]);
        
        // AUTOMOTIVE REALITY: Write immediately to EEPROM (no deferred writes)
        // Power loss (key-off) can happen at any time without warning
        #[cfg(target_arch = "arm")]
        unsafe {
            let eeprom_base = 0x1401C000u32 as *mut u8;
            core::ptr::copy_nonoverlapping(data.as_ptr(), eeprom_base.add(offset), write_len);
        }
        
        // Update comprehensive wear tracking
        let region = self.get_region_index(offset);
        if region < self.write_counters.len() {
            self.write_counters[region] += 1;
        }
        
        Ok(())
    }
}
```

### Comprehensive Wear Tracking

The system tracks detailed wear statistics for predictive maintenance:

```rust
// Storage health monitoring structures
pub struct StorageHealthReport {
    pub overall_health: StorageHealth,           // Excellent → Failed
    pub estimated_lifespan_years: f32,          // Predictive modeling
    pub most_worn_region: RegionWearInfo,       // Detailed wear analysis
    pub write_statistics: WriteStatistics,      // Usage patterns
    pub health_summary: String,                 // Human-readable status
    pub recommendations: Vec<String>,           // Actionable advice
}

// Health status classification with clear thresholds
pub enum StorageHealth {
    Excellent,  // < 50% of 100,000 cycle limit
    Good,       // 50-79% 
    Warning,    // 80-94% (years of advance warning)
    Critical,   // 95-99% (months before failure)
    Failed,     // ≥ 100% (immediate replacement needed)
}
```

### CLI Health Commands

Storage health monitoring is accessible via multiple interfaces:

```bash
# Quick health status check
rumbledome status --storage-health
# Output: "✅ Storage: Excellent (2.3% worn, >100 years remaining)"

# Detailed EEPROM health report  
rumbledome diagnostics --eeprom-report
# Output: Full console report with wear percentages, statistics, recommendations

# Raw wear tracking data for analysis
rumbledome diagnostics --wear-tracking --format json
# Output: JSON data for trending analysis and logging
```

### TFT Display Integration

Storage health is surfaced on the Teensy 4.1 display:

```rust
// Display integration in rumbledome-hal/src/teensy41/display.rs
impl DisplayController for Teensy41Display {
    fn show_status(&mut self, storage_health: StorageHealth) -> Result<(), HalError> {
        // Show health icon on main status screen
        let health_icon = match storage_health {
            StorageHealth::Excellent | StorageHealth::Good => "✅",
            StorageHealth::Warning => "⚠️", 
            StorageHealth::Critical => "🚨",
            StorageHealth::Failed => "❌",
        };
        
        // Health indicator always visible in corner of display
        self.draw_status_icon(health_icon, Point::new(110, 5))?;
        
        // Detailed health info available in diagnostic menu
        if in_diagnostic_mode {
            self.show_detailed_health_info(storage_health)?;
        }
        
        Ok(())
    }
}
```

### Desktop Simulator Health Monitoring

The desktop simulator includes full wear tracking simulation:

```rust
// rumbledome-sim/src/storage_sim.rs
impl StorageSimulator {
    // Simulate realistic wear patterns based on learning activity
    pub fn simulate_wear_progression(&mut self, learning_events: usize) {
        for _ in 0..learning_events {
            // Simulate learned data write (typically 50-100 bytes)
            let write_size = rand::range(50..100);
            self.simulate_write(LEARNED_DATA_REGION, write_size);
            
            // Update wear tracking as real hardware would
            self.wear_data.update_wear_statistics(write_size);
        }
    }
    
    // Provide predictive wear timeline for testing different usage patterns
    pub fn project_lifespan(&self, daily_drive_hours: f32) -> f32 {
        // Model realistic automotive usage patterns
        let writes_per_hour = self.estimate_writes_per_hour(daily_drive_hours);
        let annual_writes = writes_per_hour * daily_drive_hours * 365.0;
        
        EEPROM_WEAR_LIMIT as f32 / annual_writes
    }
}
```

This comprehensive approach prevents mystery storage failures by providing years of advance warning with clear, actionable guidance for end users.

## Microcontroller Backup and Replacement System

### Development Reality: "Unplanned Thermal Events"

Hardware development inevitably involves microcontroller failures - whether from development mistakes, component failures, or "unplanned thermal events" during prototyping. The backup/restore system ensures no learning data or configuration is ever lost:

### Complete System Backup

```rust
// Full system backup via SystemBackup trait
pub trait SystemBackup {
    fn create_full_backup(&mut self) -> Result<SystemBackupData, HalError>;
    fn restore_from_backup(&mut self, backup: &SystemBackupData) -> Result<RestoreResult, HalError>;
    fn verify_backup(&self, backup: &SystemBackupData) -> Result<BackupVerification, HalError>;
}

// Comprehensive backup data structure
pub struct SystemBackupData {
    pub metadata: BackupMetadata,        // Version, hardware, timestamps
    pub user_config: Vec<u8>,           // User profiles and settings
    pub learned_data: Vec<u8>,          // Calibration maps and environmental factors
    pub calibration_state: Vec<u8>,     // Auto-calibration progress
    pub safety_log: Vec<u8>,           // Historical safety events
    pub wear_tracking: WearTrackingBackup, // Storage health history
    pub system_stats: SystemStatsBackup,   // Runtime and performance data
    pub checksum: u32,                 // Data integrity verification
}
```

### CLI Backup Commands

**Create System Backup**:
```bash
# Quick backup with auto-generated filename
rumbledome backup --port /dev/ttyUSB0

# Backup with custom filename and description
rumbledome backup --port /dev/ttyUSB0 \
  --output "daily_driver_backup_2025.json" \
  --description "Pre-modification backup of daily driver tune"

# Compressed backup for archival
rumbledome backup --port /dev/ttyUSB0 --compress
```

**Restore to New Microcontroller**:
```bash
# Standard restore with compatibility verification
rumbledome restore --port /dev/ttyUSB0 --backup-file my_backup.json

# Force restore despite compatibility warnings (development)
rumbledome restore --port /dev/ttyUSB0 --backup-file old_backup.json --force

# Selective restore (config only, skip learned data)
rumbledome restore --port /dev/ttyUSB0 --backup-file backup.json \
  --skip-learned-data --yes

# Safe restore with pre-restore backup
rumbledome restore --port /dev/ttyUSB0 --backup-file backup.json \
  --backup-first
```

**Backup Management**:
```bash
# Verify backup integrity before restore
rumbledome verify backup_file.json --detailed

# List and analyze available backups
rumbledome list-backups --directory ./backups --sort-date

# Show backups from specific system
rumbledome list-backups --system-filter "teensy41-12345678"
```

### Compatibility and Version Management

The system handles version compatibility intelligently:

```rust
pub struct BackupVerification {
    pub is_valid: bool,                    // Overall validity
    pub checksum_valid: bool,              // Data integrity
    pub version_compatible: bool,          // Firmware compatibility
    pub hardware_compatible: bool,         // Platform compatibility
    pub compatibility_report: CompatibilityReport,
    pub issues: Vec<String>,               // Specific problems
}

pub struct CompatibilityReport {
    pub learned_data_compatible: bool,     // Can learned data be restored?
    pub config_compatible: bool,           // Can user config be restored?
    pub version_delta: VersionDelta,       // Version difference analysis
    pub required_migrations: Vec<String>,  // Needed data transformations
}
```

**Example Compatibility Handling**:
```
🔍 Checking compatibility...
⚠️  Compatibility warnings:
   • Firmware version mismatch: 0.1.0 → 0.2.0
   • Minor version change detected - learned data may need recalibration

❌ Restore aborted due to compatibility issues.
   Use --force to override (not recommended for production)
```

### Development Workflow Integration

**Typical Development Scenarios**:

1. **Pre-Modification Backup**:
   ```bash
   # Before making risky changes
   rumbledome backup --port /dev/ttyUSB0 \
     --description "Pre-turbo-upgrade baseline tune"
   ```

2. **Failed Microcontroller Replacement**:
   ```bash
   # Magic smoke escaped - replace Teensy and restore
   rumbledome restore --port /dev/ttyUSB0 \
     --backup-file "working_tune_backup.json" \
     --force  # Development environment
   ```

3. **Production ECU Upgrade**:
   ```bash
   # Customer ECU replacement with verification
   rumbledome backup --port /dev/ttyUSB0 \
     --output "customer_ecu_backup.json"
   
   # Install new ECU, then restore with verification
   rumbledome restore --port /dev/ttyUSB0 \
     --backup-file "customer_ecu_backup.json"
   ```

### Data Preservation Across Replacements

**What Gets Preserved**:
- **User Configuration**: Boost profiles, sensor calibrations, safety limits
- **Learned Data**: Complete calibration maps with confidence metrics
- **Auto-Calibration Progress**: Partially completed calibration sessions
- **Safety Event Log**: Historical fault analysis data
- **System Statistics**: Runtime hours, learning sessions, performance metrics
- **Wear History**: Previous microcontroller EEPROM wear data for continuity

**What Starts Fresh**:
- **Storage Wear Counters**: New microcontroller = fresh EEPROM
- **System Serial Number**: Each micro has unique hardware ID
- **Real-Time Metrics**: Current boost/RPM readings (transient data)

### Error Recovery and Validation

**Restore Validation Process**:
1. **Backup Verification**: Checksum and structure validation
2. **Compatibility Check**: Hardware and firmware compatibility analysis  
3. **Data Migration**: Handle version differences with appropriate transforms
4. **Incremental Restore**: Per-section restoration with rollback capability
5. **Post-Restore Validation**: Verify system operation and sensor calibration
6. **Required Actions**: Clear checklist of manual verification steps

**Example Restore Results**:
```
📊 Restore Results:
   User Config:      ✅ Success
   Learned Data:     ✅ Success
   Calibration:      ✅ Success
   Safety Log:       ✅ Success

📋 Required actions:
   • Restart EBC to activate restored configuration
   • Verify sensor calibrations in safe environment
   • Test all boost profiles before normal operation

⚠️  Warnings:
   • Firmware version mismatch detected
```

This system ensures that hardware failures during development never result in lost tuning data, while production deployments have full traceability and verification.

## MicroSD Card and Bluetooth Serial Integration

### Two-Tier Storage Architecture

The system uses a **dual storage strategy** that separates portable configuration from hardware-specific data:

```rust
// Storage architecture implementation
pub struct RumbleDomeStorage {
    // Instance-specific storage (tied to physical micro)
    eeprom: Teensy41Storage,           // 4KB EEPROM emulation
    
    // Portable storage (hardware-independent)  
    sd_card: PortableStorage,          // MicroSD card
    
    // Configuration resolution
    active_config: SystemConfiguration,
}

impl RumbleDomeStorage {
    pub fn load_configuration(&mut self) -> Result<SystemConfiguration, HalError> {
        // 1. Load learned data from EEPROM (hardware-specific)
        let learned_data = self.eeprom.load_learned_data()?;
        
        // 2. Load portable config from SD card
        let portable_config = match self.sd_card.load_user_profiles() {
            Ok(config) => Some(config),
            Err(_) => {
                log::warn!("SD card not available, using EEPROM defaults");
                None
            }
        };
        
        // 3. Resolve configuration with priority
        let config = SystemConfiguration::resolve(
            portable_config,  // SD card takes precedence for user settings
            learned_data,     // EEPROM provides hardware-specific calibration
        )?;
        
        self.active_config = config;
        Ok(self.active_config.clone())
    }
}
```

### MicroSD Card File Organization

```
/RUMBLEDOME/
├── profiles/
│   ├── daily_driver.json           # Conservative daily tune
│   ├── sport_mode.json             # Moderate performance tune  
│   ├── track_day.json              # Aggressive track tune
│   ├── valet_mode.json             # Ultra-safe parking attendant mode
│   └── winter_cold_start.json      # Cold weather adaptation
├── config/
│   ├── sensor_calibrations.json    # Pressure sensor parameters
│   ├── safety_limits.json          # User-defined safety boundaries
│   └── system_preferences.json     # Display/CAN/UI settings
├── backups/
│   ├── 2025-01-15_baseline.bak     # Full system backups
│   ├── 2025-01-20_post_tune.bak    # Post-modification backup
│   └── emergency_recovery.bak      # Known-good emergency config
├── logs/                           # Optional session logging
│   └── learning_sessions/
└── firmware/                       # Future OTA updates
    └── updates/
```

### Bluetooth Serial as Wireless Console

The Bluetooth interface provides **wireless access to the exact same CLI** that works over USB-C:

```rust
// Bluetooth serial abstraction - transparent to application layer
pub struct BluetoothSerial {
    uart: SerialPort,
    connection_state: BluetoothConnectionState,
}

impl SerialInterface for BluetoothSerial {
    // Same interface as USB-C serial - transparent to CLI layer
    fn write(&mut self, data: &[u8]) -> Result<(), HalError> {
        self.uart.write(data)
    }
    
    fn read(&mut self) -> Result<Vec<u8>, HalError> {
        self.uart.read()
    }
}

// CLI layer sees no difference between USB-C and Bluetooth
pub struct ConsoleInterface<T: SerialInterface> {
    serial: T,
    command_parser: CommandParser,
}

// Works identically with USB-C or Bluetooth
impl<T: SerialInterface> ConsoleInterface<T> {
    pub fn process_command(&mut self, cmd: &str) -> Result<String, HalError> {
        match self.command_parser.parse(cmd)? {
            Command::Backup { output_file } => self.handle_backup(output_file),
            Command::Restore { backup_file } => self.handle_restore(backup_file),
            Command::Status { live_mode } => self.handle_status(live_mode),
            Command::Config { profile } => self.handle_config_change(profile),
            // ... same commands regardless of connection type
        }
    }
}
```

### Mobile App as CLI GUI Wrapper

```typescript
// Mobile app - GUI wrapper around CLI commands
class RumbleDomeMobileApp {
    bluetooth: BluetoothSerial;
    
    // GUI actions translate directly to CLI commands
    async downloadConfig(): Promise<BackupData> {
        const response = await this.bluetooth.sendCommand(
            "rumbledome backup --output mobile_backup.json --format json"
        );
        return JSON.parse(response);
    }
    
    async uploadTune(backupData: BackupData): Promise<RestoreResult> {
        // Transfer backup file over serial
        await this.bluetooth.sendFile("uploaded_tune.json", backupData);
        
        // Execute restore command
        const response = await this.bluetooth.sendCommand(
            "rumbledome restore --backup-file uploaded_tune.json --format json"
        );
        return JSON.parse(response);
    }
    
    async switchProfile(profileName: string): Promise<void> {
        await this.bluetooth.sendCommand(
            `rumbledome config --profile ${profileName}`
        );
    }
    
    // Real-time telemetry via live status stream
    startLiveTelemetry(): AsyncIterator<TelemetryData> {
        return this.bluetooth.sendCommandStream(
            "rumbledome status --live --format json"
        );
    }
}
```

### Development and Deployment Workflows

**Development Scenarios**:
```bash
# Rapid prototyping - swap SD cards between test micros
cp daily_driver_v2.json /Volumes/RUMBLEDOME/profiles/daily_driver.json
# Eject SD card, move to different test micro → instant profile access

# Debug with wired connection
rumbledome status --storage-health --port /dev/ttyUSB0

# Deploy with wireless connection  
# Mobile app connects via Bluetooth → same functionality
```

**Production Deployment**:
1. **Initial Setup**: USB-C for initial configuration and SD card setup
2. **Daily Operation**: Mobile app via Bluetooth for profile switching and monitoring  
3. **Service Access**: USB-C or Bluetooth for diagnostics and updates
4. **Emergency Recovery**: SD card physical access if all else fails

**Configuration Backup Strategy**:
- **SD Card**: Portable profiles and settings (survives micro replacement)
- **EEPROM**: Hardware-specific learned data (tied to physical micro)
- **Mobile App**: Cloud sync of user profiles (additional backup layer)
- **CLI Backup**: Full system backup combining both storage tiers

This architecture provides **maximum flexibility** while maintaining **single-source-of-truth** for the CLI interface - whether accessed via USB-C cable or Bluetooth, the functionality is identical.

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