# CAN Signals — Gen2 Coyote (S550 Mustang)

## 🏗️ Constraints Layer Document

**🔗 Dependencies:** None (vehicle integration constraints are foundational)

**📤 Impacts:** Changes to available CAN signals require review of:
- **Tier 2**: Architecture.md (control algorithms), Hardware.md (CAN interface)
- **Tier 3**: Implementation.md (CAN processing), TestPlan.md (signal validation)

*Sources: [S550 CAN Database](https://github.com/EricTurner3/s550-canbus/wiki/CAN-Database), [Ford CAN IDs Summary](https://github.com/v-ivanyshyn/parse_can_logs/blob/master/Ford%20CAN%20IDs%20Summary.md)*

## ✅ Confirmed Available Signals

**🔗 T2-CAN-001**: **Engine RPM Signal**  
**Decision Type**: 🔗 **Direct Derivation** - Required for torque-following control  
**Derived From**: T1-TORQUE-001 (ECU Cooperation Philosophy)  
**AI Traceability**: Drives control loop timing, learning system indexing

- **RPM** 
  - **ID**: 0x109 (HS3 bus)
  - **Encoding**: `(b0<<8 + b1) / 4`
  - **Units**: RPM
  - **Status**: Ready for implementation

**🚧 T2-CAN-002**: **Ford S550 Torque Signal A (0x167)**  
**Status**: 🚧 **TBD - Vehicle Testing Required**  
**Decision Type**: ⚠️ **Engineering Decision** (when signal type determined)  
**Derived From**: T1-TORQUE-001 (ECU Cooperation Philosophy)  
**Research Required**: Vehicle testing to determine if this represents desired or actual torque  
**AI Generation Impact**: BLOCKS torque-following control until signal type resolved  
**Provisional Implementation**: Mock torque values for desktop simulation  
**Resolution Timeline**: Phase 1 vehicle integration testing

- **Engine Load/Torque**
  - **ID**: 0x167 (HS1 & HS3 buses) 
  - **Encoding**: `((b1-128)<<8 + b2) / 4`
  - **Units**: TBD (likely Nm or %)
  - **Status**: ⚠️ Need to determine if this is desired or actual torque

**🔗 T2-CAN-003**: **Manifold Absolute Pressure (CAN MAP)**  
**Decision Type**: 🔗 **Direct Derivation** - Required for sensor fusion safety  
**Derived From**: T1-SAFETY-002 (Defense in Depth Strategy)  
**Engineering Rationale**: CAN MAP provides redundant manifold pressure measurement for sensor fusion with boost gauge  
**AI Traceability**: Drives sensor fusion algorithms, safety monitoring, pressure validation

- **Manifold Absolute Pressure (MAP)**
  - **ID**: 0x167 (HS1 & HS3 buses)
  - **Encoding**: `((b5-25)<<8 + b6 - 128) / 5` 
  - **Units**: TBD (likely kPa → convert to PSI)
  - **Status**: Ready for implementation


**🚧 T2-CAN-004**: **Ford S550 Torque Signal B (0x43E)**  
**Status**: 🚧 **TBD - Signal Differentiation Required**  
**Decision Type**: ⚠️ **Engineering Decision** (when signal role determined)  
**Research Required**: Compare 0x167 vs 0x43E behavior to identify desired vs actual torque signals  
**Engineering Hypothesis**: May represent engine load percentage vs absolute torque  
**AI Generation Impact**: Affects torque signal selection logic  
**Resolution Method**: Simultaneous logging during acceleration/deceleration events  

- **Engine Load (Alternative)**
  - **ID**: 0x43E (HS1 & HS3 buses)
  - **Encoding**: `(b5<<8 + b6) / 72 - 140`
  - **Units**: % load
  - **Status**: Backup torque signal option

## 🚧 TBD Research Requirements

**🚧 T2-CAN-005**: **Desired vs Actual Torque Signal Identification**  
**Status**: 🚧 **TBD - Signal Differentiation Testing**  
**Decision Type**: ⚠️ **Engineering Decision**  
**Research Required**: Vehicle testing to identify which CAN signals represent ECU desired torque vs actual delivered torque  
**Critical for**: T1-TORQUE-001 (ECU Cooperation Philosophy) implementation  
**Test Methods**:
  1. Simultaneous logging of 0x167 and 0x43E during acceleration
  2. Search for additional CAN IDs with separate desired/actual signals
  3. Cross-reference with HPTuners torque channels if available
**Fallback Strategy**: PID-based torque signals if CAN separation not available

**🔬 T2-CAN-006**: **CAN Message Update Frequencies**  
**Status**: 🔬 **RESEARCH - Frequency Analysis Required**  
**Decision Type**: 🔗 **Direct Derivation** (from control loop requirements)  
**Derived From**: T2-CONTROL-001 (100Hz control loop - Architecture.md)  
**Research Required**: Measure actual CAN bus message rates on Ford S550  
**Minimum Requirement**: 20-50Hz for smooth torque-following control  
**AI Generation Impact**: Affects control loop timing and buffer sizing  
**Test Method**: CAN sniffer frequency analysis across RPM range

**🔬 T2-CAN-007**: **Signal Scaling and Accuracy Validation**  
**Status**: 🔬 **RESEARCH - Calibration Validation Required**  
**Decision Type**: ⚠️ **Engineering Decision**  
**Research Required**: 
  - Verify MAP signal produces reasonable pressure values vs boost gauge
  - Confirm torque signal units (Nm vs % load) and scaling accuracy
  - Test signal linearity and accuracy across operating range
**AI Generation Impact**: Drives signal processing algorithms, validation thresholds
**Validation Method**: Cross-reference with known boost/torque measurements

## Verification Plan

### Priority Tests
1. **Signal Accuracy Validation**
   - Compare 0x167 MAP values with known boost gauge readings
   - Verify RPM (0x109) matches dashboard/ECU readings

2. **Torque Signal Analysis** 
   - Simultaneously log 0x167 and 0x43E during acceleration to see differences
   - Cross-reference with HPTuners VCM Scanner torque channels if available
   - Test behavior during torque-limiting events (traction control, knock, etc.)

3. **Update Frequency Testing**
   - Measure actual CAN message rates for all signals
   - Verify sufficient frequency for control loop requirements


### Fallback Strategy
If desired/actual torque separation isn't available via CAN:
- **Option A**: Use HPTuners PID method for torque signals
- **Option B**: Single torque signal with torque rate-of-change derivative for enhanced response  
- **Option C**: Manifold pressure-based control with CAN RPM/load assistance

## Implementation Status

**✅ Ready for AI Code Generation:**
- **T2-CAN-001**: Engine RPM Signal (0x109) - Complete traceability  
- **T2-CAN-003**: CAN MAP Signal (0x167) - Complete traceability

**🚧 TBD - Blocks AI Generation Until Resolved:**
- **T2-CAN-002**: Torque Signal A identification (desired vs actual)
- **T2-CAN-004**: Torque Signal B identification (load vs torque)  
- **T2-CAN-005**: Desired/Actual torque separation strategy

**🔬 Research Required - Affects Implementation Details:**
- **T2-CAN-006**: Message frequency validation (control loop timing)
- **T2-CAN-007**: Signal accuracy validation (processing algorithms)

**📋 Resolution Roadmap:**
1. **Phase 1 Vehicle Testing**: Resolve T2-CAN-002, T2-CAN-004, T2-CAN-005
2. **Phase 1 Frequency Analysis**: Resolve T2-CAN-006  
3. **Phase 1 Calibration**: Resolve T2-CAN-007
4. **Result**: 100% CAN integration specifications ready for AI code generation

## HAL Integration Notes
- **Platform-specific decoding**: Encapsulate Ford S550 signal parsing in HAL layer
- **Future platforms**: GM, Mopar support through additional CAN protocol implementations
- **Graceful degradation**: System should work with subset of available signals
