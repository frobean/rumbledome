# RumbleDome Simulation Engine Requirements

📖 **Related Documentation:**
- [TestPlan.md](TestPlan.md) - Testing framework that uses this simulation engine
- [Architecture.md](Architecture.md) - Control algorithms that this simulation validates
- [Physics.md](Physics.md) - Real-world physics constraints that simulation must model

## Simulation Engine Purpose

**🔗 T2-SIM-001**: **Physics-Based Control Algorithm Validation**  
**Derived From**: T1-SAFETY-002 (Defense in Depth) + T3-BUILD-001 (Desktop Testing Requirements)  
**Decision Type**: 🔗 **Direct Derivation** - Safe algorithm development requires realistic physics simulation  
**AI Traceability**: Drives simulation accuracy requirements, validation criteria, physics modeling fidelity

The simulation engine provides **realistic physics-based validation** of RumbleDome control algorithms without risking actual turbocharger hardware. It must accurately model vehicle dynamics, ECU behavior, and turbocharger physics to enable confident algorithm development.

**🔗 T2-SIM-014**: **Test Mule Scenario Coverage Strategy**  
**Derived From**: FR-6 (Learning & Adaptation) + robust validation requirements across diverse installations  
**Decision Type**: ⚠️ **Engineering Decision** - Coverage over precision approach to scenario validation  
**Engineering Rationale**: Learning algorithms adapt to wide ranges of conditions; simulation should stress-test adaptation capability across realistic behavioral envelope rather than model one specific setup perfectly  
**AI Traceability**: Drives scenario matrix design, test mule configuration, learning robustness validation

**Coverage Over Precision Philosophy**:
- **Goal**: Prove RumbleDome works across diverse real-world installations, not perfect simulation of one specific setup
- **Strategy**: Create range of realistic challenges that stress different aspects of learning algorithms
- **Validation**: If system handles worst-case scenarios, it will work with typical equipment
- **Benefit**: Confidence across installation variations without requiring perfect simulation accuracy

**Test Mule Configuration Matrix**:
```rust
pub enum TestMuleProfile {
    // Well-behaved system - validates precision and fine-tuning
    Conservative {
        feed_pressure_variation: ±1.0,    // Good regulator (±2.5%)
        turbo_response_time: 0.8,         // Small responsive turbo  
        engine_response: Predictable,     // Consistent torque delivery
        environmental_drift: Minimal,     // Stable conditions
    },
    
    // Typical system - validates real-world performance
    Realistic {
        feed_pressure_variation: ±2.5,    // Standard regulator (±6%)
        turbo_response_time: 1.5,         // Medium turbo
        engine_response: Variable,        // RPM-dependent response zones
        environmental_drift: Gradual,     // Normal temperature/load changes
    },
    
    // Challenging system - validates robustness and adaptation  
    Difficult {
        feed_pressure_variation: ±4.0,    // Poor regulator (±10%)
        turbo_response_time: 2.8,         // Large laggy turbo
        engine_response: Unpredictable,   // Variable torque characteristics
        environmental_drift: Significant, // Altitude/temperature effects
    },
    
    // Extreme system - validates stability and safety under stress
    WorstCase {
        feed_pressure_variation: ±6.0,    // Terrible regulator (±15%)
        turbo_response_time: 4.5,         // Massive SpaceX-class turbo
        engine_response: Chaotic,         // Highly variable behavior
        environmental_drift: Extreme,     // Rapid changing conditions
    },
}
```

**Scenario Coverage Validation Strategy**:
- **Conservative Profile**: Learning system achieves precise control, fine-tunes parameters efficiently
- **Realistic Profile**: System performs well under typical installation conditions, adapts to normal variations
- **Difficult Profile**: Learning algorithms remain stable and effective despite challenging conditions
- **Worst-Case Profile**: System maintains safety and basic functionality even under extreme stress

**Multi-Dimensional Test Coverage**:
```rust
// Generate comprehensive scenario matrix
pub struct ScenarioMatrix {
    turbo_characteristics: [Small, Medium, Large, SpaceX],
    pressure_regulation: [Excellent, Good, Poor, Terrible],
    engine_responsiveness: [Quick, Moderate, Sluggish, Variable],
    environmental_conditions: [Stable, Drifting, Cycling, Chaotic],
    
    // Creates combinations like:
    // - Small turbo + poor regulator + quick engine = precision control with noisy air
    // - Large turbo + good regulator + sluggish engine = lag compensation focus
    // - Medium turbo + terrible regulator + variable engine = general robustness test
}
```

**Learning System Robustness Validation**:
- **Adaptation Range**: System learns effectively across 4:1 variation ranges in key parameters
- **Stability Under Stress**: No oscillation or divergence even with worst-case input variations  
- **Performance Graceful Degradation**: Worst-case scenarios still provide safe, functional boost control
- **Recovery Capability**: System adapts when conditions improve from worst-case to normal

## Non-Happy-Path Validation Requirements

### NHP-1: CAN Bus Jitter and Timing Variations

**🔗 T2-SIM-015**: **Configurable CAN Bus Realism**  
**Derived From**: Real-world CAN bus characteristics + input validation requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Model realistic CAN timing variations to validate input handling robustness  
**Engineering Rationale**: Real CAN buses exhibit timing jitter, dropouts, and processing delays; control algorithms must handle imperfect signal timing  
**AI Traceability**: Validates signal timeout handling, stale data detection, control loop timing robustness

**Configurable CAN Bus Characteristics**:
```rust
pub struct ConfigurableCanBusModel {
    // Message timing configuration
    pub base_message_rate_hz: f32,           // 20-50 Hz configurable
    pub timing_jitter_ms: f32,               // ±0-10ms configurable jitter
    pub message_dropout_rate: f32,           // 0-1% configurable dropout rate
    
    // ECU processing delay configuration
    pub ecu_processing_delay_ms: f32,        // 0-50ms configurable delay
    pub ecu_processing_jitter_ms: f32,       // ±0-10ms processing variation
    
    // Bus loading effects configuration  
    pub bus_utilization: f32,                // 0-95% configurable loading
    pub collision_retry_delay_ms: f32,       // 0-10ms collision handling
    
    // Message staleness configuration
    pub max_message_age_ms: u32,             // Configurable staleness threshold
    pub stale_data_behavior: StaleDataMode,  // Hold/Zero/Error configurable
}

pub enum StaleDataMode {
    HoldLastValue,      // Keep using last good value
    ZeroValue,          // Force to zero/neutral
    ErrorCondition,     // Trigger fault condition
}
```

**CAN Bus Testing Scenarios**:

**Scenario 1: Perfect CAN (Baseline)**
- Jitter: ±0ms, Dropout: 0%, Processing delay: 5ms constant
- **Validates**: Ideal case performance baseline

**Scenario 2: Normal CAN Conditions**  
- Jitter: ±2ms, Dropout: 0.05%, Processing delay: 15ms ±3ms
- **Validates**: Typical real-world CAN bus performance

**Scenario 3: Stressed CAN Bus**
- Jitter: ±5ms, Dropout: 0.2%, Processing delay: 25ms ±8ms  
- **Validates**: Control stability under poor CAN conditions

**Scenario 4: Extreme CAN Degradation**
- Jitter: ±10ms, Dropout: 1.0%, Processing delay: 50ms ±15ms
- **Validates**: Graceful degradation and fault detection

### NHP-2: Sensor Noise and Reading Variations

**🔗 T2-SIM-016**: **Configurable Sensor Realism**  
**Derived From**: Automotive sensor specifications + signal processing requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Model realistic sensor characteristics to validate filtering and learning robustness  
**Engineering Rationale**: Real sensors exhibit noise, drift, and response delays; learning algorithms must distinguish signal from noise  
**AI Traceability**: Validates noise filtering, sensor validation, learning data quality, fault detection

**Configurable Sensor Model**:
```rust
pub struct ConfigurableSensorModel {
    // Accuracy configuration
    pub base_accuracy_percent: f32,          // ±0-5% configurable accuracy
    pub noise_amplitude_psi: f32,            // ±0-0.5 PSI configurable noise
    pub temperature_drift_psi_per_c: f32,    // 0-0.1 PSI/°C configurable drift
    
    // Dynamic response configuration
    pub sensor_response_time_ms: f32,        // 0-50ms configurable lag
    pub filtering_time_constant_ms: f32,     // 0-100ms configurable filtering
    
    // Long-term drift configuration  
    pub zero_drift_psi_per_hour: f32,        // Configurable zero drift rate
    pub span_drift_percent_per_day: f32,     // Configurable span drift rate
    pub drift_enabled: bool,                 // Enable/disable drift simulation
    
    // Fault injection configuration
    pub fault_probability: f32,              // 0-0.01% configurable fault rate
    pub fault_types: Vec<SensorFaultType>,   // Configurable fault modes
}

pub enum SensorFaultType {
    Stuck,              // Sensor reads constant value
    Noisy,              // Excessive noise injection
    Drifted,            // Rapid calibration drift  
    Intermittent,       // Occasional bad readings
    Disconnected,       // Sensor completely fails
}
```

**Sensor-Specific Configurations**:
```rust
pub struct SensorConfiguration {
    // Manifold pressure sensor (critical for control)
    manifold_pressure: ConfigurableSensorModel {
        base_accuracy_percent: 1.0..3.0,    // Configurable ±1-3%
        noise_amplitude_psi: 0.02..0.15,    // Configurable ±0.02-0.15 PSI
        response_time_ms: 5.0..20.0,        // Configurable 5-20ms
    },
    
    // Feed pressure sensor (critical for compensation)
    feed_pressure: ConfigurableSensorModel {
        base_accuracy_percent: 0.5..2.0,    // Better accuracy required
        noise_amplitude_psi: 0.01..0.08,    // Lower noise tolerance
        response_time_ms: 10.0..25.0,       // Slower acceptable
    },
    
    // Dome pressure sensors (less critical)
    dome_pressures: ConfigurableSensorModel {
        base_accuracy_percent: 1.5..4.0,    // Lower accuracy acceptable
        noise_amplitude_psi: 0.05..0.20,    // More noise acceptable
        response_time_ms: 3.0..15.0,        // Faster response needed
    },
}
```

**Sensor Testing Scenarios**:

**Scenario 1: Perfect Sensors (Baseline)**
- Accuracy: ±0.1%, Noise: ±0.01 PSI, No drift
- **Validates**: Ideal sensor performance baseline

**Scenario 2: Good Quality Sensors**
- Accuracy: ±1%, Noise: ±0.05 PSI, Minimal drift
- **Validates**: High-quality sensor installation performance

**Scenario 3: Standard Automotive Sensors**  
- Accuracy: ±2%, Noise: ±0.1 PSI, Normal drift rates
- **Validates**: Typical automotive sensor performance

**Scenario 4: Poor Quality/Aged Sensors**
- Accuracy: ±3%, Noise: ±0.2 PSI, Accelerated drift
- **Validates**: Degraded sensor handling and compensation

**Scenario 5: Sensor Fault Injection**
- Random faults: stuck, noisy, drifted sensors
- **Validates**: Fault detection and graceful degradation

### NHP-3: Combined Input Degradation Testing

**Multi-Layered Degradation Scenarios**:
```rust
pub enum InputDegradationProfile {
    // Realistic combined conditions
    TypicalInstallation {
        can_jitter: ±2ms,
        can_dropouts: 0.05%,
        sensor_noise: ±0.08_PSI,
        sensor_drift: Normal,
    },
    
    // Challenging but realistic conditions  
    DifficultConditions {
        can_jitter: ±5ms,
        can_dropouts: 0.2%,
        sensor_noise: ±0.15_PSI,
        sensor_drift: Accelerated,
    },
    
    // Stress testing conditions
    ExtremeConditions {
        can_jitter: ±10ms,
        can_dropouts: 1.0%,
        sensor_noise: ±0.25_PSI,
        sensor_drift: Severe,
        fault_injection: Enabled,
    },
}
```

**Robustness Validation Requirements**:
- **Signal Timeout Handling**: System maintains safe operation when CAN messages stop
- **Stale Data Detection**: Control algorithms detect and handle aged input data appropriately  
- **Noise Filtering**: Learning system distinguishes between signal and sensor noise
- **Sensor Validation**: System detects sensor failures and maintains functionality with remaining sensors
- **Graceful Degradation**: Performance degrades gradually under input quality reduction, not catastrophically
- **Recovery Capability**: System recovers normal performance when input quality improves

## Interactive Simulation Interface Requirements

### ISI-1: Real-Time Interactive Dashboard

**🔗 T2-SIM-017**: **Interactive Real-Time Simulation Interface**  
**Derived From**: Development workflow requirements + validation methodology needs  
**Decision Type**: ⚠️ **Engineering Decision** - Interactive dashboard interface for real-time simulation control and monitoring  
**Engineering Rationale**: Real-time interaction enables intuitive understanding of system behavior, interactive debugging, and comprehensive fault testing  
**AI Traceability**: Drives simulation interface design, real-time control implementation, live telemetry systems

**Interactive Simulation Architecture**:
```rust
pub struct RumbleDomeSimulator {
    // Continuously running "vehicle"
    physics_engine: VehiclePhysicsEngine,
    rumbledome_core: RumbleDomeCore<SimulationHal>,
    
    // Real-time interactive controls
    driver_controls: DriverControls,
    environmental_controls: EnvironmentalControls,
    fault_injection_controls: FaultControls,
    
    // Live monitoring and metrics
    telemetry: LiveTelemetry,
    metrics_collector: MetricsCollector,
}

// Real-time driver controls - simulate actual driving
pub struct DriverControls {
    pub pedal_position: f32,        // 0-100% throttle input
    pub vehicle_load: f32,          // Hills, headwind, trailer load
    pub aggression_knob: f32,       // RumbleDome aggression setting (0-100%)
    pub scramble_button: bool,      // Emergency full-power override
    pub target_rpm: u16,            // Desired engine speed for testing
}

// Real-time fault injection controls
pub struct FaultControls {
    // CAN bus degradation knobs
    pub can_jitter_ms: f32,         // ±0-10ms live timing jitter
    pub can_dropout_rate: f32,      // 0-5% live message dropout rate
    pub can_processing_delay: f32,  // 0-50ms live ECU processing delay
    pub can_bus_utilization: f32,   // 0-95% bus loading simulation
    
    // Sensor degradation knobs  
    pub sensor_noise_multiplier: f32,    // 0-10x noise amplification
    pub sensor_drift_rate: f32,          // Accelerated drift simulation
    pub sensor_temperature_c: f32,       // Temperature effects on sensors
    pub manifold_sensor_fault: SensorFaultType,  // Live fault injection
    pub feed_pressure_sensor_fault: SensorFaultType,
    pub dome_sensor_faults: [SensorFaultType; 2],
    
    // Air system degradation knobs
    pub regulator_hunting_psi: f32,      // ±0-8 PSI hunting amplitude
    pub feed_pressure_offset_psi: f32,   // ±10 PSI supply pressure variation
    pub dome_leak_rate: f32,             // 0-10% pneumatic leak simulation
    pub compressor_cycling: bool,        // Enable supply pressure cycling
}

pub enum SensorFaultType {
    None,           // Normal operation
    Stuck,          // Sensor reads constant value
    Noisy,          // Excessive noise injection (5-10x normal)
    Drifted,        // Rapid calibration drift
    Intermittent,   // Occasional bad readings
    Disconnected,   // Complete sensor failure
    Slow,           // Degraded response time
}
```

**Real-Time Interaction Flow**:
```rust
impl RumbleDomeSimulator {
    pub fn run_interactive_simulation(&mut self) {
        let mut last_update = Instant::now();
        
        loop {
            // Update controls from UI (sliders, knobs, buttons)
            self.update_controls_from_interface();
            
            // Apply real-time fault injection
            self.apply_fault_conditions();
            
            // Execute one simulation timestep (1ms)
            let dt_ms = last_update.elapsed().as_millis() as f32;
            let physics_state = self.physics_engine.step(dt_ms);
            let control_result = self.rumbledome_core.execute_control_cycle();
            
            // Collect metrics and update telemetry
            self.metrics_collector.record_timestep(&physics_state, &control_result);
            self.telemetry.update_displays();
            
            // Maintain real-time execution
            std::thread::sleep(Duration::from_millis(1));
            last_update = Instant::now();
        }
    }
    
    pub fn apply_fault_conditions(&mut self) {
        let faults = &self.fault_injection_controls;
        
        // Apply CAN bus degradation in real-time
        self.physics_engine.can_bus.set_jitter(faults.can_jitter_ms);
        self.physics_engine.can_bus.set_dropout_rate(faults.can_dropout_rate);
        self.physics_engine.can_bus.set_processing_delay(faults.can_processing_delay);
        
        // Apply sensor faults in real-time
        self.physics_engine.manifold_sensor.set_fault(faults.manifold_sensor_fault);
        self.physics_engine.feed_pressure_sensor.set_fault(faults.feed_pressure_sensor_fault);
        self.physics_engine.manifold_sensor.set_noise_multiplier(faults.sensor_noise_multiplier);
        
        // Apply air system degradation in real-time
        self.physics_engine.air_system.set_regulator_hunting(faults.regulator_hunting_psi);
        self.physics_engine.air_system.set_feed_pressure_offset(faults.feed_pressure_offset_psi);
        self.physics_engine.air_system.set_leak_rate(faults.dome_leak_rate);
    }
}
```

### ISI-2: Live Metrics Collection and Visualization

**🔗 T2-SIM-018**: **Real-Time Metrics Collection and Analysis Framework**  
**Derived From**: Learning validation requirements + performance monitoring needs  
**Decision Type**: ⚠️ **Engineering Decision** - Comprehensive metrics collection for learning validation and system analysis  
**Engineering Rationale**: Real-time metrics enable learning convergence validation, performance trend analysis, and quantitative system verification  
**AI Traceability**: Validates learning effectiveness, system performance improvement, fault tolerance capabilities

**Comprehensive Metrics Collection**:
```rust
pub struct MetricsCollector {
    // Time-series configuration
    pub timeline: Vec<f64>,                    // Timestamps (seconds)
    pub sample_rate_hz: f32,                   // 10-100 Hz configurable
    pub max_history_minutes: u32,              // Rolling window size
    
    // Control performance metrics
    pub torque_error: TimeSeries<f32>,         // Nm - ECU cooperation effectiveness
    pub boost_error: TimeSeries<f32>,          // PSI - boost control precision
    pub duty_cycle: TimeSeries<f32>,           // % - control output
    pub response_time: TimeSeries<f32>,        // ms - system responsiveness
    pub control_stability: TimeSeries<f32>,    // Coefficient of variation
    
    // Learning system metrics
    pub learning_rate: TimeSeries<f32>,        // Parameter change rate
    pub parameter_stability: TimeSeries<f32>,  // Learning convergence measure
    pub confidence_scores: TimeSeries<f32>,    // System confidence in learned data
    pub adaptation_events: Vec<AdaptationEvent>, // Learning event log
    pub parameter_drift: TimeSeries<f32>,      // Parameter change tracking
    
    // System health metrics
    pub can_message_rate: TimeSeries<f32>,     // Hz - CAN bus health
    pub can_timeout_count: TimeSeries<u32>,    // CAN message timeouts
    pub sensor_noise_levels: TimeSeries<f32>,  // RMS sensor noise measurements
    pub pneumatic_health: TimeSeries<f32>,     // 0-1 air system health score
    pub safety_interventions: Vec<SafetyEvent>, // Safety system activation log
    
    // Environmental tracking
    pub feed_pressure: TimeSeries<f32>,        // PSI - air supply variations
    pub temperature_effects: TimeSeries<f32>,  // Environmental compensation
    pub load_conditions: TimeSeries<f32>,      // Vehicle loading changes
    pub environmental_drift: TimeSeries<f32>,  // Baseline drift tracking
}

pub struct TimeSeries<T> {
    pub data: VecDeque<T>,
    pub max_samples: usize,                    // Rolling window size
    pub statistics: TimeSeriesStats<T>,        // Running statistics
}

pub struct TimeSeriesStats<T> {
    pub current: T,
    pub min: T,
    pub max: T,
    pub average: T,
    pub std_deviation: T,
    pub trend: TrendDirection,                 // Rising/Falling/Stable
}

pub struct AdaptationEvent {
    pub timestamp: f64,
    pub parameter_type: String,                // What parameter changed
    pub old_value: f32,
    pub new_value: f32,
    pub trigger_reason: String,                // Why it changed
    pub confidence_level: f32,                 // How confident in the change
}

pub struct SafetyEvent {
    pub timestamp: f64,
    pub event_type: SafetyEventType,
    pub trigger_value: f32,                    // What value triggered it
    pub response_time_ms: f32,                 // How quickly system responded
    pub recovery_time_ms: Option<f32>,         // How long to recover (if applicable)
}
```

**Real-Time Dashboard Visualization**:
```rust
pub struct LiveTelemetryDisplay {
    // Main control performance dashboard
    pub control_dashboard: ControlPerformanceDashboard,
    pub learning_dashboard: LearningSystemDashboard,  
    pub health_dashboard: SystemHealthDashboard,
    pub environmental_dashboard: EnvironmentalDashboard,
}

pub struct ControlPerformanceDashboard {
    // Real-time graphs (last 60 seconds)
    pub torque_error_graph: RealTimeGraph,
    pub boost_error_graph: RealTimeGraph,
    pub response_time_graph: RealTimeGraph,
    pub duty_cycle_graph: RealTimeGraph,
    
    // Current status indicators
    pub current_torque_gap: f32,               // Nm
    pub current_boost_error: f32,              // PSI
    pub average_response_time: f32,            // ms
    pub control_stability_score: f32,          // 0-1
    
    // Performance trend indicators
    pub performance_trend: TrendIndicator,     // ↑↓→ arrows
    pub improvement_rate: f32,                 // % per minute
}

pub struct LearningSystemDashboard {
    // Learning progress graphs (last 10 minutes)  
    pub parameter_stability_graph: RealTimeGraph,
    pub learning_rate_graph: RealTimeGraph,
    pub confidence_score_graph: RealTimeGraph,
    pub adaptation_frequency_graph: RealTimeGraph,
    
    // Learning status indicators
    pub convergence_status: ConvergenceStatus, // Converging/Converged/Diverging
    pub learning_effectiveness: f32,           // 0-1 score
    pub recent_adaptations: Vec<AdaptationEvent>,
    
    // Learning event log (scrolling text)
    pub adaptation_log: Vec<String>,           // "08:32 - Adapted low-RPM duty scaling (+3%)"
}

pub struct SystemHealthDashboard {
    // System health graphs (last 30 minutes)
    pub can_health_graph: RealTimeGraph,
    pub sensor_noise_graph: RealTimeGraph,  
    pub pneumatic_health_graph: RealTimeGraph,
    pub fault_rate_graph: RealTimeGraph,
    
    // Current health status
    pub can_bus_status: HealthStatus,          // Good/Warning/Critical
    pub sensor_status: HealthStatus,
    pub pneumatic_status: HealthStatus,
    pub overall_health_score: f32,             // 0-1
    
    // Event counters
    pub safety_events_count: u32,
    pub can_timeouts_count: u32,
    pub sensor_warnings_count: u32,
}
```

### ISI-3: Advanced Learning Analysis and Validation

**🔗 T2-SIM-019**: **Learning System Analysis and Validation Framework**  
**Derived From**: FR-6 (Learning & Adaptation) + quantitative validation requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Advanced analytics for learning system validation and performance measurement  
**Engineering Rationale**: Quantitative analysis required to prove learning convergence, measure improvement, and validate adaptation effectiveness  
**AI Traceability**: Provides quantitative proof of learning system effectiveness, convergence validation, performance improvement measurement

**Learning Analysis Framework**:
```rust
pub struct LearningAnalyzer {
    pub convergence_detector: ConvergenceAnalyzer,
    pub performance_tracker: PerformanceTracker,
    pub anomaly_detector: AnomalyDetector,
    pub effectiveness_analyzer: EffectivenessAnalyzer,
}

impl LearningAnalyzer {
    pub fn analyze_learning_progress(&self, metrics: &MetricsCollector) -> LearningReport {
        LearningReport {
            // Convergence analysis
            convergence_status: self.detect_parameter_convergence(&metrics.parameter_stability),
            convergence_quality: self.calculate_convergence_quality(),
            time_to_convergence: self.estimate_convergence_time(),
            stability_score: self.measure_parameter_stability(),
            
            // Performance improvement analysis  
            performance_improvement: self.measure_performance_trend(&metrics.torque_error),
            control_consistency: self.measure_control_stability(&metrics.boost_error),
            response_improvement: self.measure_response_consistency(&metrics.response_time),
            overall_effectiveness: self.calculate_overall_effectiveness(),
            
            // Learning system effectiveness
            adaptation_frequency: self.count_recent_adaptations(),
            learning_efficiency: self.measure_learning_efficiency(),
            parameter_utilization: self.analyze_parameter_usage(),
            confidence_progression: self.analyze_confidence_trends(),
            
            // Robustness validation
            fault_tolerance: self.analyze_fault_tolerance(),
            recovery_capability: self.analyze_recovery_performance(),
            graceful_degradation: self.validate_graceful_degradation(),
        }
    }
    
    pub fn detect_learning_issues(&self, metrics: &MetricsCollector) -> Vec<LearningIssue> {
        let mut issues = Vec::new();
        
        // Parameter oscillation detection
        if self.detect_parameter_oscillation(&metrics.parameter_stability) {
            issues.push(LearningIssue::ParameterOscillation {
                amplitude: self.measure_oscillation_amplitude(),
                frequency: self.measure_oscillation_frequency(),
            });
        }
        
        // Learning stagnation detection
        if self.detect_learning_stagnation(&metrics.learning_rate) {
            issues.push(LearningIssue::LearningStagnation {
                duration: self.measure_stagnation_duration(),
                performance_plateau: self.measure_performance_plateau(),
            });
        }
        
        // Performance regression detection
        if self.detect_performance_regression(&metrics.torque_error) {
            issues.push(LearningIssue::PerformanceRegression {
                regression_magnitude: self.measure_regression_severity(),
                regression_rate: self.measure_regression_rate(),
            });
        }
        
        // Confidence degradation detection
        if self.detect_confidence_degradation(&metrics.confidence_scores) {
            issues.push(LearningIssue::ConfidenceDegradation {
                confidence_loss: self.measure_confidence_loss(),
                affected_parameters: self.identify_affected_parameters(),
            });
        }
        
        issues
    }
}

pub enum LearningIssue {
    ParameterOscillation { amplitude: f32, frequency: f32 },
    LearningStagnation { duration: f32, performance_plateau: f32 },
    PerformanceRegression { regression_magnitude: f32, regression_rate: f32 },
    ConfidenceDegradation { confidence_loss: f32, affected_parameters: Vec<String> },
    ExcessiveAdaptation { adaptation_rate: f32 },
    InsuffientAdaptation { improvement_rate: f32 },
}
```

### ISI-4: Exportable Reports and Analysis

**🔗 T2-SIM-020**: **Comprehensive Simulation Reporting and Export System**  
**Derived From**: Documentation requirements + validation reporting needs  
**Decision Type**: ⚠️ **Engineering Decision** - Comprehensive reporting system for simulation results and learning validation  
**Engineering Rationale**: Detailed reports required for system validation documentation, performance verification, and learning system proof  
**AI Traceability**: Generates quantitative proof of system performance, learning effectiveness, and fault tolerance capabilities

**Comprehensive Reporting Framework**:
```rust
pub struct SimulationReporter {
    pub csv_exporter: CsvExporter,
    pub json_exporter: JsonExporter,
    pub html_report_generator: HtmlReportGenerator,
    pub plot_generator: PlotGenerator,
}

impl SimulationReporter {
    pub fn generate_comprehensive_report(&self, session: &SimulationSession) -> ComprehensiveReport {
        ComprehensiveReport {
            executive_summary: self.generate_executive_summary(session),
            session_details: self.generate_session_summary(session),
            performance_analysis: self.analyze_performance_metrics(session),
            learning_analysis: self.analyze_learning_effectiveness(session),
            fault_tolerance_analysis: self.analyze_fault_tolerance(session),
            recommendations: self.generate_recommendations(session),
            detailed_metrics: self.export_detailed_metrics(session),
        }
    }
    
    fn generate_executive_summary(&self, session: &SimulationSession) -> ExecutiveSummary {
        ExecutiveSummary {
            // High-level results
            overall_performance_grade: self.calculate_overall_grade(session),
            learning_convergence_achieved: session.learning_converged,
            safety_validation_passed: session.safety_events.is_empty(),
            fault_tolerance_validated: session.fault_tolerance_score > 0.8,
            
            // Key performance indicators
            torque_error_improvement: session.calculate_torque_error_improvement(),
            control_stability_score: session.calculate_stability_score(),
            learning_effectiveness_score: session.calculate_learning_effectiveness(),
            
            // Critical findings
            critical_issues: session.identify_critical_issues(),
            performance_bottlenecks: session.identify_bottlenecks(),
            optimization_opportunities: session.identify_optimizations(),
        }
    }
    
    fn analyze_learning_effectiveness(&self, session: &SimulationSession) -> LearningEffectivenessReport {
        LearningEffectivenessReport {
            convergence_analysis: ConvergenceAnalysis {
                convergence_achieved: session.learning_converged,
                convergence_time_minutes: session.convergence_time,
                convergence_quality_score: session.calculate_convergence_quality(),
                parameter_stability_score: session.calculate_parameter_stability(),
            },
            
            performance_improvement: PerformanceImprovementAnalysis {
                initial_performance: session.initial_performance_metrics,
                final_performance: session.final_performance_metrics,
                improvement_magnitude: session.calculate_improvement_magnitude(),
                improvement_rate: session.calculate_improvement_rate(),
                improvement_consistency: session.calculate_improvement_consistency(),
            },
            
            adaptation_effectiveness: AdaptationEffectivenessAnalysis {
                total_adaptations: session.adaptation_events.len(),
                beneficial_adaptations: session.count_beneficial_adaptations(),
                harmful_adaptations: session.count_harmful_adaptations(),
                adaptation_accuracy: session.calculate_adaptation_accuracy(),
                learning_efficiency: session.calculate_learning_efficiency(),
            },
            
            robustness_validation: RobustnessValidationAnalysis {
                noise_tolerance: session.validate_noise_tolerance(),
                fault_recovery: session.validate_fault_recovery(),
                environmental_adaptation: session.validate_environmental_adaptation(),
                graceful_degradation: session.validate_graceful_degradation(),
            },
        }
    }
}

pub struct ComprehensiveReport {
    pub executive_summary: ExecutiveSummary,
    pub session_details: SessionSummary,
    pub performance_analysis: PerformanceAnalysisReport,
    pub learning_analysis: LearningEffectivenessReport,
    pub fault_tolerance_analysis: FaultToleranceReport,
    pub recommendations: Vec<Recommendation>,
    pub detailed_metrics: MetricsExport,
}
```

**Interactive Testing Workflow Integration**:
```rust
pub enum InteractiveTestScenario {
    // Real-time testing scenarios
    HillClimbTest {
        initial_grade: f32,
        final_grade: f32,
        ramp_duration_seconds: f32,
        fault_injection: Option<FaultControls>,
    },
    
    FaultRecoveryTest {
        fault_sequence: Vec<FaultInjectionStep>,
        recovery_validation: RecoveryValidationCriteria,
    },
    
    LearningValidationTest {
        learning_duration_minutes: f32,
        scenario_variations: Vec<ScenarioVariation>,
        convergence_criteria: ConvergenceCriteria,
    },
    
    StressTest {
        stress_factors: StressFactors,
        duration_minutes: f32,
        degradation_limits: DegradationLimits,
    },
}
```

**This comprehensive framework provides:**
- **Real-time interactive control** of all simulation parameters
- **Live visualization** of system performance and learning progress  
- **Quantitative validation** of learning convergence and effectiveness
- **Comprehensive fault injection** and robustness testing
- **Detailed reporting** with exportable analysis and recommendations
- **Interactive testing workflows** for systematic validation scenarios

## Core Simulation Requirements

### SR-1: Physics Accuracy Requirements

**🔗 T2-SIM-002**: **Turbocharger Physics Modeling**  
**Derived From**: T2-CONTROL-003 (3-Level Control Hierarchy) + real-world turbo behavior  
**Decision Type**: ⚠️ **Engineering Decision** - Balance simulation accuracy with computational complexity  
**Engineering Rationale**: Control algorithms must experience realistic turbo lag, boost response, and wastegate dynamics  
**AI Traceability**: Drives turbo inertia modeling, compressor maps, shaft speed calculations

**🔗 T2-SIM-010**: **Parametric Turbocharger Modeling by Size Categories**  
**Derived From**: Control algorithm validation requirements + development simplicity  
**Decision Type**: ⚠️ **Engineering Decision** - Simplified turbo modeling focused on control response characteristics  
**Engineering Rationale**: Control algorithms need realistic turbo lag and boost response patterns without complex thermodynamics; parametric approach by turbo size creates different control challenges  
**AI Traceability**: Drives turbo response modeling, control algorithm stress testing, learning system validation

**Turbocharger Size Categories**:
```rust
pub enum TurboSize {
    Small,    // Stock-ish: quick spool, limited capacity (EcoBoost style)
    Medium,   // Mild upgrade: balanced response (GT28 style)  
    Large,    // Big single: laggy but powerful (GT35+ style)
    SpaceX,   // Ridiculous: science experiment territory
}

pub struct TurboCharacteristics {
    spool_time_constant: f32,        // seconds to 63% boost response
    max_boost_capacity: f32,         // PSI at redline  
    low_rpm_efficiency: f32,         // boost capability <3000 RPM
    inertia_factor: f32,             // resistance to boost changes
    response_curve_type: SpoolCurve, // Exponential, Linear, S-curve
}
```

**Size Category Specifications**:

**Small Turbo (Control Precision Testing)**:
- Spool time: 0.6-1.0s, Max boost: 8-12 PSI, Low RPM efficiency: 90%
- Control challenge: Limited headroom, requires precise control, easy to hit ceiling
- Learning focus: Fine-tuning, steady-state optimization

**Medium Turbo (Balanced Testing)**:  
- Spool time: 1.0-1.5s, Max boost: 15-20 PSI, Low RPM efficiency: 70%
- Control challenge: Good balance of response and power, realistic daily driver
- Learning focus: General algorithm validation, typical use cases

**Large Turbo (Lag Compensation Testing)**:
- Spool time: 2.0-3.0s, Max boost: 25-35 PSI, Low RPM efficiency: 30% 
- Control challenge: Significant lag, overshoot potential, requires prediction
- Learning focus: Lag compensation, overshoot prevention, high-performance scenarios

**SpaceX Turbo (Extreme Condition Testing)**:
- Spool time: 3.0-5.0s, Max boost: 40+ PSI, Low RPM efficiency: 10%
- Control challenge: Extreme lag, massive overshoot risk, nearly uncontrollable
- Learning focus: Edge case handling, algorithm stability under extreme conditions

**Simplified Response Modeling**:
```rust
// Core boost response without complex thermodynamics
fn update_boost_response(&mut self, exhaust_energy: f32, dt_s: f32) -> f32 {
    let target_boost = self.calculate_rpm_limited_boost(exhaust_energy);
    let response_rate = (target_boost - self.current_boost) / self.spool_time_constant;
    let inertia_limited = response_rate * (1.0 - self.inertia_factor);
    self.current_boost += inertia_limited * dt_s;
    self.current_boost.clamp(0.0, self.max_boost_capacity)
}
```

**Control Algorithm Validation Benefits**:
- **Different Response Personalities**: Each size creates unique control challenges
- **Stress Testing Range**: From precision control (small) to extreme lag handling (SpaceX)
- **Learning Algorithm Coverage**: Validates adaptation across turbo characteristics  
- **Realistic Behavior**: Captures essential turbo lag without unnecessary complexity

**Manifold Pressure Dynamics**:
- **Volume Effects**: 2-4L manifold volume creates realistic pressure wave propagation
- **Flow Conservation**: Mass flow in (compressor) = mass flow out (engine consumption + leakage)
- **Temperature Effects**: Gas temperature affects pressure relationships (ideal gas law)
- **Pressure Oscillation**: Realistic damping of pressure waves and resonances

**Wastegate Physics**:
- **Actuator Response**: Spring pressure + dome pressure = net force on wastegate
- **Flow Characteristics**: Position-dependent exhaust bypass flow (0-100% open)
- **Pneumatic Dynamics**: **Simplified lumped delay model** - aggregate 50-80ms response time
- **Mechanical Limits**: Physical stops, spring preload, actuator travel limits

**🔗 T2-SIM-008**: **Pneumatic System Modeling Strategy**  
**Derived From**: Control algorithm validation requirements + development velocity priorities  
**Decision Type**: ⚠️ **Engineering Decision** - Simplified pneumatic model for initial development  
**Engineering Rationale**: Pneumatic plumbing complexity (line sizes, dome volumes, flow rates) distills to aggregate response time for control algorithm validation  
**AI Traceability**: Drives pneumatic response modeling, wastegate position dynamics, safety response timing

**🔗 T2-SIM-012**: **Air Management System Integration with Primary Control Loop**  
**Derived From**: Architecture.md T2-CONTROL-009 (Adaptive Pneumatic System) + primary control loop requirements  
**Decision Type**: 🔗 **Direct Implementation** - Implement existing feed pressure specifications in simulation  
**Engineering Rationale**: Air management is integral control actuator affecting duty cycle scaling, control resolution, and safety authority  
**AI Traceability**: Implements documented feed pressure compensation, control authority modeling, pneumatic system health

**Reference Implementation of Existing Specifications**:
- **Feed Pressure Control Authority**: Implements Architecture.md "Control Resolution ∝ 1 / Input Air Pressure"
- **Optimal Pressure Calculation**: Uses documented `Nominal Feed Pressure = Spring Pressure + Safety Margin + (Target Boost × Scaling Factor)`
- **Dynamic Compensation**: Implements `compensated_duty_cycle = base_duty_cycle × (nominal_feed_pressure / actual_feed_pressure)`  
- **Safety Monitoring**: Implements Safety.md SY-19 feed pressure fault detection

**Air Management Control Loop Integration**:
```rust
// Implements Architecture.md T2-CONTROL-009 specifications
pub struct AirManagementSystem {
    // Feed pressure monitoring (Architecture.md specification)
    pub feed_pressure_psi: f32,
    pub nominal_feed_pressure_psi: f32,      // Calculated per Architecture.md formula
    
    // Control characteristics (Architecture.md Control Resolution formula)
    pub control_resolution_factor: f32,      // Inversely proportional to feed pressure
    pub control_authority_available: bool,   // Safety.md SY-19 validation
    
    // Dome pressures (Architecture.md sensor specification)
    pub upper_dome_pressure_psi: f32,       // Wastegate closing force validation
    pub lower_dome_pressure_psi: f32,       // Wastegate opening force validation
    
    // System health (Architecture.md fault detection)
    pub pneumatic_health_status: PneumaticHealth, // Good/Degraded/Failed
}

// Feed pressure compensation (Architecture.md documented algorithm)  
impl AirManagementSystem {
    fn calculate_pressure_compensation(&self) -> f32 {
        self.nominal_feed_pressure_psi / self.feed_pressure_psi
    }
    
    fn validate_control_authority(&self) -> Result<(), PneumaticFault> {
        // Implements Safety.md SY-19 requirements
        if self.feed_pressure_psi < (self.spring_pressure_psi + SAFETY_MARGIN_PSI) {
            return Err(PneumaticFault::InsufficientOpeningAuthority);
        }
        if self.feed_pressure_psi > self.calculate_max_useful_pressure() {
            return Err(PneumaticFault::ExcessivePressureCompression);
        }
        Ok(())
    }
}
```

**Integration with RumbleDome Control Loop**:
```rust  
// Primary control loop with air management (per Architecture.md)
impl RumbleDomeCore {
    fn execute_control_cycle(&mut self) -> Result<(), CoreError> {
        // 1. Read air system state
        let air_state = self.hal.read_air_management_system()?;
        
        // 2. Validate pneumatic authority (Safety.md SY-19)
        air_state.validate_control_authority()?;
        
        // 3. Calculate base control output  
        let base_duty = self.calculate_torque_following_duty()?;
        
        // 4. Apply documented feed pressure compensation
        let compensation = air_state.calculate_pressure_compensation();
        let compensated_duty = base_duty * compensation;
        
        // 5. Update learning with pressure-normalized data (Architecture.md)
        self.learned_data.update_pressure_normalized(base_duty, &air_state)?;
        
        Ok(())
    }
}
```

**Simulation Implementation Requirements**:
- **Reference Architecture.md**: All pneumatic calculations use documented formulas and constants
- **Implement Safety.md SY-19**: Feed pressure fault detection with documented thresholds
- **Dome Pressure Dynamics**: Simple first-order lag representing 50-80ms pneumatic response  
- **Control Authority Modeling**: Feed pressure effects on control resolution per documented inverse relationship
- **Pressure Compensation**: Real-time compensation using documented algorithm

**🔗 T2-SIM-013**: **Realistic Air Pressure Regulation Simulation**  
**Derived From**: Real-world pneumatic regulator characteristics + Architecture.md compensation requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Model realistic pressure variation to validate compensation algorithms  
**Engineering Rationale**: Real regulators exhibit noise, hunting, and drift; simulation must include these variations to test documented compensation algorithms  
**AI Traceability**: Validates Architecture.md pressure compensation, learning system adaptation, control stability under realistic conditions

**Realistic Air Pressure Behavior Modeling**:
```rust
pub struct RealisticAirPressureSimulation {
    nominal_feed_pressure: f32,    // Target regulated pressure (per Architecture.md calculation)
    
    // Pressure variation sources
    regulator_hunting_amplitude: f32,  // ±1.5 PSI typical regulator hunting
    supply_variation_amplitude: f32,   // ±2.0 PSI compressor cycling effects
    flow_noise_amplitude: f32,         // ±0.8 PSI random demand variations
    
    // Variation timing characteristics
    hunting_frequency_hz: f32,         // 2.0 Hz typical regulator hunting
    supply_cycle_period_s: f32,        // 20s compressor cycling
}

impl RealisticAirPressureSimulation {
    fn update_pressure_with_realistic_noise(&mut self, time_s: f32) -> f32 {
        // Regulator hunting around setpoint
        let hunting = (time_s * self.hunting_frequency_hz * 2.0 * PI).sin() 
                     * self.regulator_hunting_amplitude;
        
        // Compressor cycling - supply pressure variations
        let supply_cycle = triangle_wave(time_s / self.supply_cycle_period_s) 
                          * self.supply_variation_amplitude;
        
        // Random flow demand effects and sensor noise
        let flow_noise = random_gaussian() * self.flow_noise_amplitude;
        
        let actual_pressure = self.nominal_feed_pressure + hunting + supply_cycle + flow_noise;
        
        // Realistic regulator bounds (can't go negative or excessive)
        actual_pressure.clamp(
            self.nominal_feed_pressure * 0.75,  // 25% below nominal worst case
            self.nominal_feed_pressure * 1.25   // 25% above nominal worst case
        )
    }
}
```

**Pressure Variation Testing Scenarios**:

**Scenario 1: Normal Regulator Operation**
- Nominal: 40 PSI, Variation: ±2 PSI, Hunting: 2 Hz
- **Tests**: Compensation algorithm stability, learning adaptation to noise

**Scenario 2: Degraded Regulator Performance**  
- Nominal: 35 PSI, Variation: ±4 PSI, Hunting: 1 Hz with larger amplitude
- **Tests**: Control stability under poor regulation, safety authority maintenance

**Scenario 3: Supply Pressure Cycling**
- Large slow variations (±3 PSI over 30s) simulating compressor cycling
- **Tests**: Long-term compensation tracking, learning system adaptation

**Compensation Algorithm Validation Requirements**:
- **Dynamic Scaling**: `compensated_duty = base_duty × (nominal_pressure / actual_pressure)` maintains consistent control
- **Noise Rejection**: Control output remains stable despite ±2 PSI pressure variations
- **Learning Adaptation**: Learned parameters adapt to average pressure over time, ignore short-term noise
- **Safety Authority**: Overboost protection remains effective despite pressure variations
- **Control Stability**: No hunting or oscillation induced by pressure noise

**Validation Against Documented Behavior**:
- **Low Feed Pressure**: Reduced control authority, safety fault conditions (Safety.md)
- **High Feed Pressure**: Compressed control band, hunting risk (Architecture.md)  
- **Optimal Feed Pressure**: Balanced control resolution and authority (Architecture.md calculation)
- **Pressure Variations**: Dynamic compensation maintains consistent control despite realistic regulator noise (Architecture.md)
- **Noise Immunity**: Learning system distinguishes between pressure variations and actual performance changes

### SR-2: ECU Behavior Modeling

**🔗 T2-SIM-003**: **Realistic ECU Torque Management Simulation**  
**Derived From**: T2-ECU-001 (Torque Production Assistant) + real Ford PCM behavior  
**Decision Type**: ⚠️ **Engineering Decision** - ECU behavior must reflect actual Ford calibration strategies  
**Engineering Rationale**: RumbleDome cooperation logic requires realistic ECU torque climbing and limiting behavior  
**AI Traceability**: Drives driver demand table simulation, torque curve modeling, safety intervention logic

**🔗 T2-SIM-009**: **Scenario-Based Torque Gap Modeling with Temporal Dynamics**  
**Derived From**: T2-ECU-001 (Torque Production Assistant) + control algorithm validation requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Focus on torque gap patterns rather than detailed engine physics  
**Engineering Rationale**: RumbleDome responds to torque gaps regardless of their origin; scenario-based modeling provides realistic control challenges without engine combustion complexity  
**AI Traceability**: Drives scenario design, temporal dynamics modeling, torque gap evolution patterns

**Scenario-Based Approach**:
- **Focus**: Model realistic torque gap patterns (desired vs actual) rather than detailed torque production physics
- **Benefit**: Simpler implementation while providing comprehensive control algorithm validation
- **Coverage**: All scenarios where RumbleDome boost assistance is beneficial

**Critical Requirement: Temporal Coherence**:
- **Time-Series Evolution**: Torque gaps must evolve realistically over time with proper derivatives
- **Calculus-Based Dynamics**: Include torque gap rates (dGap/dt) and acceleration (d²Gap/dt²) 
- **Physical Consistency**: Vehicle dynamics, load changes, and ECU responses must be temporally coherent
- **Control Challenge Creation**: Realistic time evolution creates proper control algorithm validation scenarios

**Dynamic Torque Gap Scenarios**:

**Hill Climb Evolution**:
```
t=0s:    Level ground, gap=5Nm (normal)
t=10s:   Grade increases, vehicle slows, ECU demands more torque
t=20s:   Desired=350Nm, Actual=280Nm (power-limited), gap=70Nm
t=30s:   RumbleDome assists, actual torque increases
t=40s:   Gap closes to 20Nm, vehicle maintains speed
```

**Acceleration Tip-In Evolution**:
```
t=0s:    Cruise, gap=0Nm
t=0.1s:  Pedal input, desired jumps to 300Nm, actual=150Nm (turbo lag)
t=0.5s:  Turbo spooling, actual=200Nm, gap=100Nm
t=1.0s:  Turbo spooled, actual=290Nm, gap=10Nm
t=2.0s:  Steady state, gap=5Nm
```

**Environmental Power Loss Evolution**:
```
t=0s:    Cool morning, gap=5Nm (normal)
t=600s:  Engine warming, intake temps rising
t=1200s: Hot day conditions, max power reduces
t=1800s: Desired=280Nm, Actual=250Nm (heat soak), gap=30Nm
```

**Torque Gap Dynamics Modeling**:
- **Gap Rate Calculation**: `torque_gap_rate = d(desired_torque - actual_torque)/dt`
- **ECU Climbing Rate**: How aggressively ECU increases torque demand over time
- **Load Change Rate**: External load variations (grade changes, wind, traffic)
- **Vehicle Response Rate**: Speed/acceleration changes affecting load dynamics
- **Environmental Rate**: Temperature, altitude changes over drive cycle

**🔗 T2-SIM-011**: **Engine Response Characteristics for Realistic Torque Gap Evolution**  
**Derived From**: T2-SIM-009 (Scenario-Based Torque Gap Modeling) + Coyote engine characteristics  
**Decision Type**: ⚠️ **Engineering Decision** - Model engine response speed to create realistic torque gap dynamics  
**Engineering Rationale**: Engine response speed affects how quickly torque gaps close; fast-revving Coyote creates different gap evolution patterns than sluggish engines  
**AI Traceability**: Drives torque gap time evolution, learning algorithm stress testing, reactive control validation

**Engine Response Speed Impact**:
- **Purpose**: Create realistic torque gap evolution patterns for RumbleDome to react to
- **Not For Prediction**: Engine response affects gap dynamics, not predictive control algorithms
- **Coyote Characteristics**: Rev-happy engine with RPM-dependent responsiveness

**Coyote Response Characteristics**:
```rust
pub struct EngineResponseModel {
    // RPM-dependent torque response characteristics
    low_rpm_response_time: f32,      // 0.3s response below 3000 RPM (sluggish)
    high_rpm_response_time: f32,     // 0.1s response above 4000 RPM (happy)
    transition_rpm_range: (u16, u16), // (3000, 4000) RPM transition zone
    
    // Simple torque delivery modeling
    current_actual_torque: f32,
    torque_response_time_constant: f32, // Varies by RPM zone
}
```

**RPM-Dependent Response Zones**:
- **Below 3000 RPM**: Slower torque response (0.3s time constant)
  - Creates persistent torque gaps requiring sustained RumbleDome assistance
  - Learning challenge: System must provide consistent help for sluggish response
  
- **3000-4000 RPM Transition**: Medium response (0.2s time constant)  
  - Creates moderate torque gap evolution
  - Learning challenge: Balanced assistance requirements
  
- **Above 4000 RPM Power Band**: Fast torque response (0.1s time constant)
  - Creates rapidly closing torque gaps  
  - Learning challenge: Light touch required, engine handles most gaps quickly

**Torque Gap Evolution Examples**:
```rust
// Same 100 Nm torque request, different RPM zones:

// Low RPM scenario (2500 RPM):
t=0ms:   desired=300, actual=200, gap=100 Nm
t=200ms: desired=300, actual=220, gap=80 Nm  // Gap persists
// RumbleDome learning: "Need sustained assistance at low RPM"

// High RPM scenario (5000 RPM): 
t=0ms:   desired=300, actual=200, gap=100 Nm  
t=200ms: desired=300, actual=285, gap=15 Nm  // Gap closes quickly
// RumbleDome learning: "Light assistance needed in power band"
```

**Implementation Approach**:
- **Simple First-Order Response**: `actual_torque += (desired_torque - actual_torque) / time_constant * dt`
- **RPM-Zone Lookup**: Time constant varies by current RPM zone
- **No Prediction**: Engine model only affects torque gap evolution rate, not predictive control
- **Torque-Following Focus**: Creates realistic scenarios for reactive gap response

**Control Algorithm Benefits**:
- **Realistic Gap Dynamics**: Different evolution patterns challenge learning algorithms appropriately
- **RPM-Zone Learning**: System learns different assistance strategies for different engine response zones  
- **Stress Testing Range**: From persistent gaps (low RPM) to rapid resolution (high RPM)
- **Torque-Following Validation**: Tests reactive response to various gap closure rates

**Required Scenario Characteristics**:
- **Realistic Time Constants**: ECU response (10-50ms), engine response (0.1-0.3s), turbo lag (0.5-2s), load changes (1-10s)
- **Proper Derivatives**: Smooth torque gap evolution without artificial step functions
- **Physics Feedback**: Vehicle dynamics affect load, which affects torque gaps
- **Engine Response Integration**: RPM-dependent torque delivery affects gap evolution patterns
- **Control Algorithm Stress**: Scenarios must challenge rate limiting, learning, safety systems

**Temporal Dynamics Implementation Requirements**:
```rust
// Required simulation interface for temporal coherence
pub trait DynamicTorqueScenario {
    /// Update scenario with 1ms timestep for proper derivative calculation
    fn step(&mut self, dt_ms: f32) -> ScenarioState;
    
    /// Get current torque signals with realistic time evolution
    fn get_torque_signals(&self) -> (f32, f32);  // (desired, actual)
    
    /// Get dynamics for control algorithm rate limiting validation
    fn get_dynamics(&self) -> ScenarioDynamics {
        torque_gap_rate: f32,      // dGap/dt (Nm/s)
        load_change_rate: f32,     // External load dynamics
        ecu_climb_rate: f32,       // ECU aggressiveness
        vehicle_response_rate: f32, // Speed/acceleration feedback
    }
}
```

**Load Recognition**:
- **Vehicle Load Calculation**: Grade, headwind, rolling resistance, acceleration demand
- **Power Demand vs Available**: ECU recognizes when hitting physical power limits
- **Torque Request Scaling**: Reduce targets when sustained power demand exceeds capability
- **Recovery Behavior**: Resume normal torque climbing when load conditions improve

### SR-3: Environmental Baseline Drift Modeling

**🔗 T2-SIM-004**: **Environmental Effects as Learning System Challenge**  
**Derived From**: FR-6 (Learning & Adaptation) + environmental compensation requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Environmental effects are slow background changes handled by learning system  
**Engineering Rationale**: ECU manages environmental compensation; RumbleDome sees net effects as baseline parameter drift requiring learning adaptation  
**AI Traceability**: Drives learning system validation, baseline drift scenarios, adaptation rate testing

**Simplified Environmental Approach**:
- **ECU Responsibility**: Temperature, altitude, humidity compensation handled by ECU fueling/timing
- **RumbleDome Perspective**: Environmental effects appear as slow baseline drift in learned parameters
- **Learning Challenge**: System must track gradually changing baselines without oscillation
- **Timescale Separation**: Environmental changes (minutes to hours) vs control decisions (milliseconds)

**Environmental Baseline Drift Patterns**:
- **Power Baseline Drift**: Gradual changes in engine power output (simulates altitude/temperature effects)
- **Boost Efficiency Drift**: Slow changes in duty-cycle-to-boost relationships (simulates turbo/intercooler effects)
- **Learning System Response**: Validation that learning tracks drift without instability

**Drift Scenario Examples**:
```
Mountain Ascent (2 hours):
t=0min:    45% duty → 8 PSI boost (sea level baseline)
t=60min:   45% duty → 7.2 PSI boost (altitude effects)
t=120min:  52% duty → 8 PSI boost (learning adapted)

Hot Day Progression (4 hours):
t=0hr:     Cool morning, baseline stable
t=2hr:     Engine warming, slight power reduction
t=4hr:     Hot afternoon, 5% power loss
Learning:  Duty baselines drift upward 3-7% over day
```

**Environmental Validation Requirements**:
- **Drift Tracking**: Learning system adapts to 1-10% baseline changes over hours
- **Stability**: No oscillation or hunting during slow environmental changes  
- **Convergence Rate**: Appropriate response speed (not too fast, not too slow)
- **Bounded Adaptation**: Drift compensation stays within safety limits

### SR-4: Vehicle Dynamics Integration

**🔗 T2-SIM-005**: **Comprehensive Vehicle Load Simulation**  
**Derived From**: T2-SIM-003 (ECU Behavior Modeling) + realistic driving scenario requirements  
**Decision Type**: ⚠️ **Engineering Decision** - Vehicle dynamics must create realistic load conditions for ECU  
**Engineering Rationale**: ECU torque climbing behavior depends heavily on opposing load forces  
**AI Traceability**: Drives grade simulation, rolling resistance, aerodynamic modeling

**Vehicle Load Forces**:
- **Rolling Resistance**: Speed-dependent rolling resistance (tires, bearings, driveline)
- **Aerodynamic Drag**: Speed-squared drag force with realistic Cd values
- **Grade Resistance**: Hill climbing forces based on vehicle weight and grade angle
- **Acceleration Demand**: Inertial forces during acceleration/deceleration events

**Transmission Modeling**:
- **Gear Ratios**: Realistic gear ratios for torque multiplication/RPM relationships
- **Shift Logic**: Basic automatic transmission shift point simulation
- **Torque Converter**: Slip characteristics affecting power delivery (if applicable)
- **Final Drive**: Differential ratio effects on wheel torque

**Vehicle Parameters**:
- **Mass**: 1800-2200 kg typical (Ford Mustang GT range)
- **Frontal Area**: 2.3 m² typical
- **Drag Coefficient**: 0.35-0.40 typical
- **Rolling Resistance**: 0.008-0.012 coefficient range
- **Grade Capability**: -15% to +15% grade simulation

## Simulation Fidelity Requirements  

### SF-1: Accuracy Targets

**🔗 T2-SIM-006**: **Simulation Accuracy Specifications**  
**Derived From**: Control algorithm validation requirements + acceptable error margins  
**Decision Type**: ⚠️ **Engineering Decision** - Balance accuracy with computational performance  
**Engineering Rationale**: Simulation must be accurate enough to validate control algorithms without perfect precision  

**Turbocharger Response Accuracy**:
- **Boost Response Time**: Within ±20% of actual turbo lag characteristics
- **Steady-State Boost**: Within ±5% of target boost pressure
- **Transient Response**: Rise time and overshoot within ±15% of actual behavior
- **Wastegate Position**: Actuator position accuracy within ±10%

**ECU Behavior Accuracy**:
- **Torque Request Timing**: ECU torque climbing behavior within ±10% of actual rates
- **Load Response**: Power limiting behavior under load within ±15% of actual
- **Safety Interventions**: Overboost/knock protection timing within ±50ms of actual
- **Steady-State Accuracy**: Final torque achievement within ±5% of actual ECU

**Environmental Effects**:
- **Temperature Compensation**: Power/efficiency changes within ±10% of actual
- **Altitude Effects**: Power loss with elevation within ±15% of actual
- **Pressure/Temperature**: Thermodynamic calculations within ±5% accuracy

### SF-2: Performance Requirements

**Computational Performance**:
- **Real-Time Factor**: Simulate 1 second of vehicle operation in <100ms computation time
- **Time Step**: 1ms maximum time step for adequate control loop resolution
- **Memory Usage**: <500MB RAM for complete simulation state
- **Deterministic**: Identical inputs produce identical outputs (repeatable testing)

**Simulation Stability**:
- **Numerical Stability**: No divergence or oscillation in physics calculations
- **Long Duration**: Stable operation for simulated drive cycles up to 8 hours
- **Extreme Conditions**: Stable behavior at temperature/altitude/load extremes
- **Fault Recovery**: Graceful handling of sensor failures and system faults

## Validation Requirements

### VR-1: Simulation Validation Criteria

**🔗 T2-SIM-007**: **Simulation-to-Reality Correlation**  
**Derived From**: T1-SAFETY-002 (Defense in Depth) + algorithm validation requirements  
**Decision Type**: 🔗 **Direct Derivation** - Simulation must correlate with reality for valid algorithm testing  
**AI Traceability**: Drives validation test procedures, correlation metrics, acceptance criteria

**Correlation with Real Hardware**:
- **Boost Response Curves**: Simulated vs actual boost response within ±15% RMS error
- **Turbo Lag Characteristics**: Spool time correlation within ±20%
- **ECU Torque Behavior**: Torque climbing rates within ±15% of actual vehicle
- **Wastegate Response**: Pneumatic response times within ±25% of actual hardware

**Drive Cycle Validation**:
- **Fuel Economy**: Simulated fuel consumption within ±10% of actual (when available)
- **Performance Metrics**: 0-60 times, quarter-mile within ±5% of actual vehicle
- **Boost Patterns**: Boost pressure traces during real drive cycles within ±15% correlation
- **Temperature Rise**: Simulated heat soak behavior within ±20% of actual

### VR-2: Test Scenario Coverage

**Mandatory Test Scenarios**:
- **Normal Operation**: City/highway driving with various aggression settings
- **Performance Operation**: Wide-open-throttle acceleration from various RPM points
- **Environmental Extremes**: Hot day (40°C), cold morning (-10°C), high altitude (2000m)
- **Load Conditions**: Hill climbing (5%, 10%, 15% grades), trailer towing simulation
- **Fault Conditions**: Sensor failures, overboost conditions, CAN timeouts

**Learning Validation Scenarios**:
- **Convergence Testing**: Multi-hour drive cycles with learning system active
- **Stability Analysis**: Extended operation to verify learning doesn't oscillate
- **Performance Improvement**: Measurable improvement in torque following over time
- **Environmental Adaptation**: Learning system adapts to temperature/altitude changes

### VR-3: Acceptance Criteria

**Algorithm Validation Requirements**:
- **Safety Response**: All safety scenarios (overboost, faults) result in safe system state
- **Torque Following**: ECU cooperation algorithms achieve <10 Nm average torque error
- **Learning Convergence**: Learned parameters stabilize within 2 hours simulated driving
- **Performance Consistency**: Control performance degrades <5% across environmental conditions

**Simulation Quality Gates**:
- **Physics Realism**: All simulated responses within specified accuracy targets
- **Numerical Stability**: No computational divergence or oscillation over 8-hour simulation
- **Repeatability**: Identical test runs produce identical results (±1% variation)
- **Coverage**: Test scenarios exercise all control algorithm code paths

## Implementation Guidelines

### IG-1: Modular Architecture

**Component Isolation**:
- **Physics Engine**: Separate module for all vehicle/turbo physics
- **ECU Simulator**: Independent ECU behavior modeling
- **Environmental Model**: Isolated environmental effects simulation
- **Test Harness**: Separate framework for scenario execution and analysis

**Interface Standards**:
- **HAL Compatibility**: Simulation engine implements same HAL interfaces as real hardware
- **Configuration**: All simulation parameters configurable via external files
- **Data Logging**: Comprehensive logging of all simulation state for analysis
- **Repeatability**: Simulation state can be saved/restored for debugging

### IG-2: Development Process

**Incremental Development**:
1. **Basic Physics**: Start with simplified turbo and manifold dynamics
2. **ECU Behavior**: Add realistic ECU torque management behavior  
3. **Environmental**: Add temperature, altitude, and load effects
4. **Validation**: Correlate with real hardware data when available
5. **Refinement**: Improve accuracy based on validation results

**Testing Strategy**:
- **Unit Testing**: Each physics component tested independently
- **Integration Testing**: Complete simulation scenarios with known expected outcomes
- **Correlation Testing**: Compare simulation to real hardware when available
- **Regression Testing**: Automated testing to prevent simulation behavior changes

### IG-3: Configuration Management

**Parameter Organization**:
- **Vehicle Config**: Mass, drag coefficient, gear ratios, engine specifications
- **Turbo Config**: Inertia, efficiency maps, wastegate characteristics
- **ECU Config**: Torque curves, calibration aggressiveness, safety limits
- **Environment Config**: Temperature ranges, altitude effects, weather conditions

**Calibration Data**:
- **Measured Data**: Use real vehicle data when available for parameter validation
- **Literature Values**: Engineering handbooks for standard automotive parameters  
- **Tunable Parameters**: Clearly marked parameters that may need adjustment
- **Version Control**: Track simulation parameter changes and their validation status

## Success Criteria

**Primary Objectives**:
- **Safe Algorithm Development**: RumbleDome control algorithms validated without hardware risk
- **Learning System Validation**: Quantitative proof that learning converges and improves performance
- **Realistic Testing**: Simulation scenarios representative of real-world operating conditions
- **Development Velocity**: Rapid iteration on control algorithms with immediate feedback

**Quality Metrics**:
- **Correlation Accuracy**: Simulation matches real hardware within specified tolerances
- **Algorithm Coverage**: All control logic tested across full operating envelope
- **Safety Validation**: All fault conditions tested and verified safe
- **Performance Validation**: Control performance meets specifications in simulation

This simulation engine transforms RumbleDome development from **experimental hardware testing** to **validated algorithm development** with quantitative performance metrics and comprehensive safety validation.