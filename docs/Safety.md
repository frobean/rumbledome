# RumbleDome Safety Requirements

## Critical Safety Principles

### Defense in Depth Strategy
RumbleDome employs multiple independent safety layers to prevent engine damage under all failure scenarios:

1. **Electronic Safety**: Software-based monitoring and response
2. **Pneumatic Safety**: Physical system design ensures safe failure modes
3. **Mechanical Safety**: Spring-loaded wastegate provides final backup

### Fail-Safe Design Philosophy
**Primary Principle**: Any system failure must result in the safest possible state - minimal boost operation.

## Non-Negotiable Safety Invariants

### SY-1: Pneumatic Fail-Safe Operation
- **Requirement**: `duty = 0%` forces full input pressure to lower dome → wastegate forced open → minimal boost
- **Rationale**: Physical system design ensures that total electronic failure results in safe operation
- **Validation**: System behavior must be verified through pneumatic testing with solenoid power removed

### SY-2: High-Authority System Recognition
- **Requirement**: System recognizes that small duty cycle changes can produce large boost changes, requiring more conservative control strategies
- **Rationale**: High-pressure pneumatic system can achieve dangerous boost levels with relatively low duty cycles (15-20%)
- **Implementation**: All control algorithms must account for high system sensitivity

### SY-3: Overboost Response Validation
- **Requirement**: System must verify it can dump upper dome pressure fast enough before allowing higher input pressures
- **Rationale**: High input pressures may prevent adequate overboost response time
- **Testing**: Automated overboost response time validation during system startup and configuration changes

### SY-4: Progressive Calibration Safety
- **Requirement**: Auto-calibration starts at spring+1 psi overboost limits, increases only as system proves safety response
- **Rationale**: Learning system must never exceed proven safe operating limits
- **Implementation**: Calibration system maintains conservative limits until multi-pass validation confirms safe operation

## Configuration & Control Safety

### SY-5: No Raw Duty Cycle Configuration
- **Requirement**: All user-facing configuration is in PSI/kPa; system learns required duty cycles through calibration
- **Rationale**: Prevents user misconfiguration that could result in dangerous duty cycles
- **Enforcement**: User interface must not accept or display raw duty cycle values

### SY-6: Torque Ceiling Enforcement
- **Requirement**: CAN torque error drives control loop; pressure limits provide safety override
- **Rationale**: ECU cooperation prevents harsh interventions while maintaining ultimate safety control
- **Override Logic**: Manifold pressure limits always take precedence over torque-based control requests

### SY-7: Configurable Spring Pressure
- **Requirement**: Wastegate spring pressure must be user-configurable, not hardcoded
- **Rationale**: Different installations have different spring pressures; system must adapt to actual hardware
- **Validation**: Spring pressure setting must be validated against actual system response

### SY-8: Pneumatic System Optimization
- **Requirement**: System recommends optimal input air pressure for control resolution and safety response
- **Rationale**: Prevents dangerous configurations where overboost response time is inadequate
- **Monitoring**: Continuous monitoring of pneumatic system health and response capability

## Fault Handling & Recovery

### SY-9: Critical Fault Response
- **Requirement**: Any sensor/CAN/NVM fault ⇒ duty=0%, visible fault reason, logged timestamp
- **Implementation**: All fault conditions must be detectable within 200ms and result in immediate safe state
- **Recovery**: Manual fault acknowledgment required before system can resume operation

### SY-10: CAN Dependency Management
- **Requirement**: Lost CAN torque signals = fault condition (no fallback to pressure-only mode)
- **Rationale**: System is designed around ECU cooperation; operating without ECU data compromises safety model
- **Timeout**: CAN signal loss detection within 500ms maximum

### SY-11: Learning System Bounds
- **Requirement**: All learned duty cycle adjustments have slew rate limits (maximum change per time period) and absolute bounds (cannot exceed proven safe ranges), and are stored separately from user configuration
- **Rationale**: Prevents learning system from making dangerous adjustments
- **Limits**: Learning adjustments limited to ±5% per hour maximum, ±20% absolute maximum from baseline

### SY-12: Learning Reset Capability
- **Requirement**: Provide command to reset all learned calibration data
- **Rationale**: Enables recovery from corrupted or dangerous learned parameters
- **Preservation**: Reset operation preserves user configuration profiles

## Development & Validation Safety

### SY-13: Core Logic Portability
- **Requirement**: Control logic compiles/runs on desktop without MCU dependencies
- **Rationale**: Enables comprehensive testing of safety-critical algorithms without hardware
- **Coverage**: All safety logic must be testable through desktop simulation

### SY-14: Comprehensive Safety Testing
- **Requirement**: All safety behaviors covered by unit/integration tests; CI must pass before merge
- **Coverage Targets**: 
  - 100% coverage of fault handling paths
  - 100% coverage of overboost response scenarios
  - 100% coverage of learning bounds enforcement
- **Validation**: Automated testing of safety response timing requirements

### SY-15: Progressive Validation Requirements
- **Requirement**: Each calibration phase requires successful completion before advancing to higher boost limits
- **Criteria**: Multi-pass consistency, overboost response validation, system health confirmation
- **Rollback**: Automatic rollback to previous safe limits if validation fails

## Safety Monitoring & Diagnostics

### SY-16: Real-Time Safety Monitoring
- **Overboost Detection**: Continuous monitoring with <100ms response time
- **Sensor Validation**: Plausibility checking of all pressure sensor readings
- **CAN Signal Validation**: Torque signal range and consistency checking
- **System Health**: Pneumatic response time validation and performance monitoring

### SY-17: Safety Event Logging
- **Requirement**: All safety events logged with precise timestamps and system state
- **Retention**: Safety logs preserved across power cycles
- **Analysis**: Logged data sufficient for post-incident analysis and system improvement

### SY-18: Operator Safety Feedback
- **Immediate Feedback**: All safety interventions immediately visible on display
- **System Status**: Continuous indication of safety system health and readiness
- **Warnings**: Proactive warnings for conditions that could compromise safety

## Environmental & Operational Safety

### SY-19: Environmental Adaptation Limits
- **Requirement**: Environmental compensation factors bounded to prevent unsafe adaptation
- **Temperature Range**: Compensation limited to ±10% for temperature variations
- **Altitude Compensation**: Limited to ±15% for altitude variations up to 10,000 feet
- **Supply Pressure**: Compensation tracks input pressure changes but maintains safety margins

### SY-20: Operational State Safety
- **Power-On Safety**: System starts in safe state, requires explicit activation
- **Profile Switching**: Live profile changes validated for safety before activation
- **Calibration Safety**: Calibration mode has additional safety constraints and monitoring
- **Emergency Shutdown**: Manual emergency shutdown capability always available

## Safety Validation Requirements

### SY-21: Hardware-in-Loop Validation
- **Pneumatic Response**: Physical validation of overboost response times
- **Sensor Accuracy**: Validation of pressure sensor accuracy and calibration
- **Solenoid Performance**: Verification of PWM response and duty cycle accuracy

### SY-22: Failure Mode Testing
- **Single Point Failures**: Testing of all single component failures
- **Multiple Failures**: Testing of credible multiple failure scenarios
- **Recovery Testing**: Validation of fault recovery and system restart procedures

### SY-23: Integration Safety Testing
- **ECU Interaction**: Validation that system cooperation does not compromise ECU safety functions
- **CAN Bus Safety**: Verification that CAN communication failures are handled safely
- **User Interface Safety**: Confirmation that all user interface operations maintain safety