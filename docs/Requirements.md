# RumbleDome Requirements

## Functional Requirements

### FR-1: ECU Integration & Cooperation
- **Torque Production Assistant**: System monitors ECU torque requests and delivery, modulating boost to help ECU achieve torque targets
- **Final Torque Respect**: Respond to ECU's final desired_torque (after all safety systems have applied modifications) to avoid conflicts
- **Automatic Safety Integration**: Automatically cooperate with traction control, ABS, clutch protection, and other ECU safety systems without specific programming
- **Torque Target Strategy**: Target actual_torque at configurable percentage below desired_torque ceiling (default ~95%) to prevent harsh ECU interventions
- **Predictable Response**: Provide consistent, repeatable boost response so ECU's driver demand tables remain valid across all operating conditions

### FR-2: Boost-Based Profile Management
- **Boost Pressure Configuration**: All user configuration in pressure units (PSI/kPa), never raw duty cycles or power targets
- **Engine-Specific Profiles**: User configures boost pressure limits based on their specific engine's safe operating parameters
- **Profile Types**: Support multiple boost profiles with distinct use cases:
  - **Valet**: 0-2 psi max (near naturally-aspirated operation for inexperienced drivers)
  - **Daily**: Conservative boost curve for comfortable daily driving power increase  
  - **Aggressive**: Moderate boost curve for spirited driving
  - **Track**: Maximum safe boost curve for experienced drivers/track use
- **Boost vs Power Independence**: Profiles define boost pressure limits; actual power output depends on engine tune, turbo sizing, and other vehicle-specific factors
- **Live Profile Switching**: Safe switching between profiles during operation
- **System Parameters**: Configurable wastegate spring pressure, overboost limits, overboost hysteresis per profile

### FR-3: Auto-Calibration System
- **Progressive Learning**: System learns duty cycle mappings for boost targets through safe, progressive calibration runs
- **Safety Progression**: Start calibration at spring+1 psi overboost limit, gradually increase as system proves safety response
- **Multi-Pass Validation**: Require consistent results across multiple calibration runs before accepting learned values
- **Environmental Adaptation**: Compensate for temperature, altitude, and dome supply pressure variations
- **Calibration Reset**: Provide capability to reset all learned calibration data

### FR-4: Pneumatic System Optimization
- **Input Pressure Analysis**: Calculate optimal air supply pressure for best duty cycle resolution and safety response
- **Real-time Health Monitoring**: Monitor pneumatic system performance and detect suboptimal configurations
- **Optimization Recommendations**: Provide specific recommendations for air supply pressure adjustments
- **Response Time Validation**: Verify that overboost pressure dumps can occur within required timeframes

### FR-5: Safety & Fault Management
- **Overboost Protection**: Immediate duty cut to 0% when manifold pressure exceeds configured limits
- **Progressive Overboost Limits**: Only increase overboost limits as system proves adequate safety response capability  
- **Fault Response**: Any sensor, CAN, or storage fault results in duty=0% with visible fault reason and logged timestamp
- **CAN Dependency**: Lost CAN torque signals treated as fault condition (no fallback to pressure-only mode)
- **High-Authority Recognition**: System recognizes that small duty cycle changes can produce large boost changes

### FR-6: Learning & Adaptation
- **Bounded Learning**: All learned duty cycle adjustments have slew rate limits and absolute bounds
- **Environmental Compensation**: Learn compensation factors for temperature, altitude, and supply pressure variations
- **Confidence Tracking**: Track calibration confidence and data quality metrics
- **Separate Storage**: Learned data stored separately from user configuration

### FR-7: User Interface & Monitoring
- **Real-time Display**: TFT display showing boost gauge, targets, calibration progress, pneumatic health
- **System Status**: Display current profile, system state, fault conditions, calibration progress
- **Configuration Interface**: JSON/CLI protocol for configuration, calibration control, system recommendations
- **Diagnostic Information**: Provide torque signals, pressure readings, learned duty cycles, system health data

## Performance Requirements

### PF-1: Real-time Operation
- **Control Loop Frequency**: 100 Hz minimum control loop execution
- **Response Time**: Overboost detection and response within 100ms maximum
- **CAN Processing**: Process CAN messages with minimal latency

### PF-2: Reliability & Availability
- **Fault Detection**: Detect sensor and communication faults within 200ms
- **Recovery Time**: System ready for operation within 5 seconds of power-on
- **Data Persistence**: Learned calibration data preserved across power cycles

## Platform Requirements

### PL-1: Hardware Abstraction
- **Multi-platform Support**: Support different MCU platforms through HAL abstraction
- **Sensor Flexibility**: Support different pressure sensor types and ranges
- **CAN Adaptability**: Support different vehicle CAN protocols through HAL implementations
- **Component Upgrades**: Enable hardware component upgrades without core logic changes

### PL-2: Development & Testing
- **Desktop Simulation**: All control logic must be testable on desktop without hardware dependencies
- **Comprehensive Testing**: Unit and integration test coverage for all safety-critical functionality
- **Mock Hardware**: Complete mock HAL implementation for testing scenarios

## Constraints & Limitations

### Phase 1 Scope
- Fixed torque target strategy (configurable percentage, not adaptive)
- Manual profile selection (user must select power level explicitly)
- Ford Gen2 Coyote CAN protocol only
- Basic environmental compensation
- Boost pressure configuration only (power output varies by engine setup)

### Future Phase Scope (Phase 2+: "Beyond RumbleDome")
- **Delivery Style Integration**: Drive modes affect boost delivery aggressiveness, not power levels
  - Power Level (Profile): What boost/power you get (user selected)
  - Delivery Style (Drive Mode): How that power is delivered (Normal/Sport+/Track aggressiveness)
- **Advanced Predictive Control**: Throttle position integration for anticipatory boost management
- **Multi-variable Environmental Compensation**: Advanced algorithms for temperature, altitude, humidity
- **Platform Expansion**: Support for additional vehicle CAN protocols and engine platforms