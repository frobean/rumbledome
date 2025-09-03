# RumbleDome Learned Data Specification

This document defines the complete learned data system for RumbleDome - what the system learns, how it learns, and where that knowledge is stored.

📖 **Related Documentation:**
- [Requirements.md](Requirements.md) - System requirements for learning capabilities
- [Architecture.md](Architecture.md) - High-level learning system architecture
- [Safety.md](Safety.md) - Safety constraints for learning algorithms
- [TechnicalSpecs.md](TechnicalSpecs.md) - Storage hardware and capacity specifications

---

## Learning Philosophy

RumbleDome's learning system follows the **STFT/LTFT philosophy** from ECU fuel management:
- **Fast adaptation** to immediate conditions and changes
- **Slow adaptation** to long-term trends and environmental shifts  
- **Rate-limited and bounded** adjustments to prevent dangerous learned parameters
- **Separation of concerns**: User sets limits, system learns implementation details

**Core Principle**: The system learns the **"how"** (implementation details) while users control the **"what"** (safety limits and targets).

---

## Learned Data Categories

### 1. Duty Cycle Calibration Maps 📊

**Purpose**: Learn the fundamental relationship between solenoid duty cycle and achieved boost pressure across operating conditions.

**Why Learning is Necessary**:
- Turbo system variations (different turbos, piping, intercoolers)
- Pneumatic system characteristics (supply pressure, dome volumes, leakage rates)
- Environmental conditions (temperature, altitude, humidity)
- Component aging and wear over time

**Data Sources**:
- **Primary**: Manifold pressure sensor (achieved boost)
- **Secondary**: RPM from CAN bus (operating point context)
- **Control**: Solenoid duty cycle output (what we commanded)
- **Environmental**: Dome input pressure (supply conditions)

**Learning Algorithm**:
```
For each control cycle:
1. Record: (RPM, Target_Boost, Commanded_Duty, Achieved_Boost, Environmental_Context)
2. Calculate: boost_error = Target_Boost - Achieved_Boost
3. Update short-term trim: STFT += boost_error * fast_learn_rate
4. Update long-term trim: LTFT += STFT * slow_learn_rate  
5. Bounds checking: Limit trims to ±20% maximum adjustment
6. Confidence tracking: Increase confidence with consistent results
```

**Storage Structure**:
```rust
pub struct DutyCalibrationMap {
    // 2D lookup: (RPM_bucket, Boost_target) → CalibrationPoint
    calibration_grid: [[CalibrationPoint; BOOST_BUCKETS]; RPM_BUCKETS],
}

pub struct CalibrationPoint {
    baseline_duty: f32,        // Learned baseline duty cycle (0.0-1.0)
    short_term_trim: f32,      // Fast adaptation (-0.1 to +0.1)
    long_term_trim: f32,       // Slow adaptation (-0.2 to +0.2) 
    confidence: f32,           // Learning confidence (0.0-1.0)
    sample_count: u32,         // Number of learning samples
    last_updated: Timestamp,   // For staleness detection
}
```

**Grid Dimensions** (Full Resolution - No Storage Constraints):
- **RPM Range**: 1000-7500 RPM in 250 RPM buckets (26 buckets)
- **Boost Range**: 0-30 PSI in 1 PSI buckets (30 buckets) 
- **Total Points**: 26 × 30 = 780 calibration points
- **Storage Size**: ~18.7KB (780 points × 24 bytes/point)
- **High Resolution Benefits**: Smoother interpolation, better transient response, more precise learned calibration

**Learning Rate Parameters**:
- **Fast Learn Rate**: 0.05 (5% adjustment per cycle when needed)
- **Slow Learn Rate**: 0.001 (0.1% migration from STFT to LTFT per cycle)
- **Confidence Threshold**: 0.8 (80% confidence required for LTFT updates)

---

### 2. Environmental Compensation Factors 🌡️

**Purpose**: Adapt duty cycle calibration to changing environmental conditions that affect air density and turbo performance.

**Why Learning is Necessary**:
- **Temperature changes**: Hot air reduces turbo efficiency, requires higher duty
- **Altitude changes**: Lower air density affects boost production
- **Supply pressure variations**: Dome input pressure affects control authority  
- **Seasonal changes**: Long-term weather patterns and fuel quality variations

**Data Sources**:
- **Primary**: Dome input pressure sensor (supply pressure monitoring)
- **Secondary**: Performance deltas (when achieved boost differs from expected)
- **Future**: CAN temperature signals (engine, intake air, ambient)
- **Inferred**: Air density from performance characteristics

**Learning Algorithm**:
```
For each successful boost delivery:
1. Record baseline environmental conditions during calibration
2. Compare current environmental conditions to baseline
3. Calculate performance delta: actual_boost vs expected_boost  
4. Attribute delta to environmental factors using correlation analysis
5. Update compensation factors using exponential moving average (1% rate)
6. Apply bounds limiting: compensation factors limited to ±30%
```

**Storage Structure**:
```rust
pub struct EnvironmentalCompensation {
    temperature_baseline: f32,        // Reference temperature (°C)
    temperature_factor: f32,          // Duty correction per °C (0.7-1.3)
    
    altitude_baseline: f32,           // Reference pressure (kPa)  
    altitude_factor: f32,             // Duty correction for altitude (0.8-1.2)
    
    supply_pressure_baseline: f32,    // Reference dome input (PSI)
    supply_pressure_factor: f32,      // Correction for supply changes (0.9-1.1)
    
    humidity_factor: f32,             // Future: air density compensation (0.95-1.05)
    
    learning_confidence: f32,         // How well environmental model is learned
    sample_count: u32,                // Number of environmental learning events
    last_updated: Timestamp,          // Staleness tracking
}
```

**Storage Size**: ~48 bytes

**Learning Rate**: 1% exponential moving average for stable, gradual adaptation

---

### 3. Sensor Fusion Cross-Calibration ⚖️

**Purpose**: Learn the systematic offset between CAN MAP sensor and dedicated boost gauge to enable seamless sensor fusion across vacuum and boost ranges.

**Why Learning is Necessary**:
- **Sensor differences**: Manufacturing tolerances create systematic offsets
- **Installation variations**: Different sensor locations, hose lengths, restrictions  
- **Aging differences**: Sensors drift at different rates over time
- **Seamless operation**: System must operate without sensor faults or discontinuities

**Data Sources**:
- **Primary**: CAN MAP sensor readings (vacuum to ~2 PSI)
- **Secondary**: Dedicated boost gauge readings (0-30 PSI range)
- **Calibration Zone**: ±2 PSI around atmospheric pressure (overlap region)
- **Operating Context**: Engine running, stable manifold pressure conditions

**Learning Algorithm**:
```
During overlap zone operation (MAP sensor reading -2 to +2 PSI):
1. Simultaneously read both sensors
2. Calculate instantaneous offset: boost_gauge - can_map
3. Filter for stable conditions (low pressure rate of change)
4. Update learned offset using exponential moving average (1% rate)
5. Track calibration confidence based on sample consistency
6. Bounds check: offset limited to ±5 PSI maximum
```

**Storage Structure**:
```rust
pub struct SensorFusionData {
    map_boost_offset: f32,           // Learned offset: Boost_gauge - CAN_MAP (PSI)
    calibration_confidence: f32,     // Learning confidence (0.0-1.0) 
    overlap_samples: u32,            // Number of calibration data points
    offset_stability: f32,           // Variance metric for offset consistency
    last_calibration: Timestamp,     // When offset was last updated
    
    // Diagnostic data
    max_observed_offset: f32,        // Largest offset seen (drift detection)
    min_observed_offset: f32,        // Smallest offset seen
    calibration_range: f32,          // Range of offsets during learning
}
```

**Storage Size**: ~32 bytes

**Calibration Conditions**:
- **Engine running**: Stable manifold pressure readings
- **Overlap zone**: Both sensors reading -2 to +2 PSI
- **Stable pressure**: Rate of change < 0.5 PSI/second
- **Minimum samples**: 100 readings required for initial confidence

---

### 4. Safety Response Parameters 🚨

**Purpose**: Learn optimal safety system characteristics to provide effective overboost protection while minimizing false triggers and recovery disruption.

**Why Learning is Necessary**:
- **System response timing**: Different pneumatic systems have different dump rates
- **Sensor noise characteristics**: Optimal filtering for each installation
- **Environmental sensitivity**: Temperature affects pneumatic response speed
- **Overboost recovery**: Balance between safety and smooth operation

**Data Sources**:
- **Primary**: Overboost events and recovery timing
- **Secondary**: Pneumatic system response measurements (dome pressure sensors)
- **Control**: Safety intervention effectiveness and timing
- **Context**: Environmental conditions during safety events

**Learning Algorithm**:
```
During overboost events:
1. Record: trigger_pressure, response_time, recovery_pressure, environmental_context
2. Measure: actual_dump_rate, recovery_smoothness, false_trigger_indicators
3. Calculate optimal hysteresis: balance safety margin vs smooth operation
4. Update safety parameters using conservative learning rate (0.1% per event)
5. Validate: ensure learned parameters maintain safety margins
6. Bounds enforcement: Never reduce safety margins below minimums
```

**Storage Structure**:
```rust
pub struct LearnedSafetyData {
    // Overboost hysteresis learning
    overboost_hysteresis: f32,           // Learned optimal hysteresis band (PSI)
    hysteresis_confidence: f32,          // Confidence in learned value
    
    // Response timing optimization  
    trigger_response_time: f32,          // Optimal safety response timing (ms)
    recovery_response_time: f32,         // Optimal recovery timing (ms)
    
    // Environmental adaptation
    temperature_response_factor: f32,    // Temperature effect on response timing
    pressure_response_factor: f32,       // Supply pressure effect on response
    
    // Safety event history for learning
    total_overboost_events: u32,         // Total safety interventions
    false_trigger_count: u32,            // Suspected false positives
    recovery_success_rate: f32,          // Successful recoveries vs total events
    
    // Bounds and validation
    min_safe_hysteresis: f32,            // Never learn below this value
    max_response_time: f32,              // Never learn above this timing
    last_safety_update: Timestamp,       // When safety params were updated
}
```

**Storage Size**: ~64 bytes

**Learning Rate**: 0.1% per safety event (very conservative for safety-critical parameters)

**Safety Constraints**:
- **Minimum hysteresis**: 0.2 PSI (never learn below this safety margin)
- **Maximum response time**: 150ms (safety response must be faster)
- **Bounds validation**: All learned safety parameters validated against hard limits

---

## Learning Lifecycle

### Phase 1: Fresh System (0-10 hours operation)
**Characteristics**:
- **Conservative operation**: Large safety margins, slow responses
- **Active learning**: High learning rates for duty cycle calibration
- **Bootstrap calibration**: Progressive expansion of safe operating envelope
- **User guidance**: System provides recommendations for calibration runs

**Data State**:
- **Duty calibration**: Sparse, low confidence, conservative baselines
- **Environmental factors**: Default values, minimal compensation
- **Sensor fusion**: Uncalibrated, large uncertainty bands  
- **Safety parameters**: Conservative defaults, no learned optimization

### Phase 2: Active Learning (10-100 hours operation)  
**Characteristics**:
- **Expanding envelope**: Gradual increase in boost targets as safety is proven
- **Multi-pass validation**: Require consistent results before accepting learned values
- **Environmental adaptation**: Begin learning compensation factors
- **Confidence building**: Increase learning confidence with consistent results

**Data State**:
- **Duty calibration**: Moderate coverage, building confidence
- **Environmental factors**: Basic compensation factors learned
- **Sensor fusion**: Good calibration in overlap zone
- **Safety parameters**: Some optimization based on system response

### Phase 3: Mature System (100+ hours operation)
**Characteristics**:
- **Stable operation**: Well-learned calibration across operating range
- **Fine tuning**: Small adjustments for long-term trends
- **Environmental mastery**: Good compensation for weather/seasonal changes
- **Predictive behavior**: System anticipates and compensates for known conditions

**Data State**:
- **Duty calibration**: Full coverage, high confidence, stable baselines
- **Environmental factors**: Comprehensive compensation model
- **Sensor fusion**: Excellent cross-calibration, seamless operation
- **Safety parameters**: Optimized for installation, minimal false triggers

### Reset and Recovery Procedures

**User-Initiated Reset** (`reset_learned_data` command):
- **Preserves**: User configuration (aggression, limits, spring pressure)
- **Resets**: All calibration maps, environmental factors, sensor fusion
- **Maintains**: Safety event logs for diagnostic purposes
- **Recovery**: System returns to Phase 1 conservative operation

**Partial Reset Scenarios**:
- **Calibration only**: Reset duty maps, preserve environmental/sensor learning
- **Environmental only**: Reset compensation factors, preserve calibration
- **Safety only**: Reset learned safety parameters, preserve operational data

**Automatic Reset Triggers**:
- **Data corruption**: Checksum failures trigger automatic reset
- **Impossible values**: Learned parameters outside physical bounds
- **Hardware changes**: Major component replacement detected
- **Firmware updates**: Version incompatibility triggers selective reset

---

## Storage Requirements

### Total Storage Allocation (SD Card - No Size Constraints)
```
Duty Calibration Maps:    18,720 bytes (780 points × 24 bytes)
Environmental Compensation:   48 bytes  
Sensor Fusion Data:          32 bytes
Safety Parameters:           64 bytes
Learning Metadata:           64 bytes
Data Integrity (checksums):  32 bytes
---------------------------------------------------------
Total Learned Data:      18,960 bytes (~19KB)
```

### SD Card Storage Structure
```
User Configuration:        1KB  (user_config.json)
Learned Calibration:      19KB  (calibration_maps.bin)
Environmental Data:        1KB  (environmental.json)
Sensor Fusion:             1KB  (sensor_fusion.json)
Safety Parameters:         1KB  (safety_params.json)
Backups (5 versions):    115KB  (automatic rolling backups)
Diagnostic Logs:          50KB  (safety events, system health)
---------------------------------------------------------
Total Storage Used:      188KB  (0.18% of 8GB card)
```

**Storage Benefits**:
- **No size constraints**: Full learning capability enabled
- **High resolution**: Smoother control and better transient response  
- **Growth room**: Easy to add more learned parameters in future
- **User accessibility**: Human-readable config files
- **Automatic backups**: Multiple versions preserved for safety

### Persistence Strategy

**Write Frequency Management**:
- **Duty calibration**: Write every 50 learning updates or 10 minutes (whichever first)
- **Environmental factors**: Write every 1 hour of operation or 5% change
- **Sensor fusion**: Write every 100 calibration samples 
- **Safety parameters**: Write immediately after each safety event
- **Metadata**: Write every 10 minutes of operation

**SD Card Wear Optimization**:
- **Debounced writes**: 5-10 second delay to batch rapid changes
- **Change detection**: Only write when learned values actually change
- **Atomic operations**: Write to temp file, then rename (crash-safe)
- **Built-in wear leveling**: SD card controller manages wear automatically
- **Write combining**: Batch related parameter updates together

**Data Integrity**:
- **Checksums**: Each learned data category has independent checksum
- **Redundant storage**: Critical safety parameters stored in multiple locations
- **Corruption recovery**: Automatic reset of corrupted sections
- **Validation**: All learned parameters validated against physical bounds on load

---

## Integration with System Architecture

### Control Loop Integration
```rust
// Level 2: Precise Boost Delivery uses learned data
let learned_baseline_duty = lookup_learned_duty(rpm, boost_target, &environmental_factors);
let environmental_correction = apply_environmental_compensation(&environmental_factors);
let final_baseline = learned_baseline_duty * environmental_correction;
```

### Sensor Fusion Integration  
```rust
// Seamless sensor transition using learned offset
let fused_pressure = if raw_pressure < 2.0 {
    can_map_pressure + learned_map_offset  // Use CAN MAP in vacuum range
} else {
    boost_gauge_pressure                   // Use boost gauge above atmospheric
};
```

### Safety System Integration
```rust
// Apply learned hysteresis for optimal safety response
let trigger_threshold = user_overboost_limit;
let recovery_threshold = user_overboost_limit - learned_hysteresis;
```

### Backup/Restore Compatibility
- **Version compatibility**: Learned data tagged with firmware version
- **Selective restore**: Users can restore learned data independently from config
- **Cross-platform**: Environmental factors transfer between installations
- **Hardware-specific**: Duty calibration may require re-learning on hardware changes

---

*This document serves as the authoritative specification for all RumbleDome learned data. All implementation must follow these definitions for consistency and safety.*