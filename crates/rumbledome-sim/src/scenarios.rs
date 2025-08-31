//! Test scenarios for RumbleDome simulation
//! 
//! Defines various test scenarios to validate system behavior
//! under different operating conditions.

use crate::engine_sim::EngineCommand;
use rumbledome_hal::DriveMode;
use serde::{Deserialize, Serialize};

/// Test scenario definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    /// Scenario name
    pub name: String,
    
    /// Scenario description
    pub description: String,
    
    /// Expected duration (seconds)
    pub duration: u64,
    
    /// Scenario events
    pub events: Vec<ScenarioEvent>,
    
    /// Success criteria
    pub success_criteria: Vec<SuccessCriterion>,
}

/// Timed event within a scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioEvent {
    /// Time to execute event (milliseconds from start)
    pub time_ms: u64,
    
    /// Command to execute
    pub command: ScenarioCommand,
    
    /// Event description
    pub description: String,
}

/// Commands available in scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScenarioCommand {
    /// Set throttle position
    SetThrottle(f32),
    
    /// Set RPM directly
    SetRpm(u16),
    
    /// Change drive mode
    SetDriveMode(DriveMode),
    
    /// Start calibration
    StartCalibration { rpm: u16, boost: f32 },
    
    /// Switch profile
    SwitchProfile(String),
    
    /// Apply torque limit
    ApplyTorqueLimit(f32),
    
    /// Reset to idle
    ResetToIdle,
    
    /// Force overboost condition
    ForceOverboost,
    
    /// Simulate knock
    SimulateKnock,
    
    /// Wait for specified time
    Wait(u64),
}

/// Success criteria for scenario validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    /// Time range to check (ms)
    pub time_range: (u64, u64),
    
    /// Condition to check
    pub condition: SuccessCondition,
    
    /// Criterion description
    pub description: String,
}

/// Conditions for success validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuccessCondition {
    /// System state equals expected
    SystemState(String),
    
    /// Boost pressure within range
    BoostPressureRange(f32, f32),
    
    /// RPM within range
    RpmRange(u16, u16),
    
    /// Torque error below threshold
    TorqueErrorBelow(f32),
    
    /// No overboost events
    NoOverboostEvents,
    
    /// Calibration completed
    CalibrationComplete,
    
    /// Duty cycle within range
    DutyCycleRange(f32, f32),
}

/// Scenario execution runner
pub struct ScenarioRunner {
    /// Current event index
    current_event: usize,
    
    /// Events that have been executed
    executed_events: Vec<ScenarioEvent>,
    
    /// Start time of scenario
    start_time: Option<u64>,
}

impl ScenarioRunner {
    pub fn new() -> Self {
        Self {
            current_event: 0,
            executed_events: Vec::new(),
            start_time: None,
        }
    }
    
    /// Update scenario and return commands to execute
    pub fn update_scenario(&mut self, scenario: &TestScenario, current_time_ms: u64) -> Vec<EngineCommand> {
        if self.start_time.is_none() {
            self.start_time = Some(current_time_ms);
        }
        
        let elapsed = current_time_ms - self.start_time.unwrap();
        let mut commands = Vec::new();
        
        // Check for events to execute
        while self.current_event < scenario.events.len() {
            let event = &scenario.events[self.current_event];
            
            if elapsed >= event.time_ms {
                // Execute this event
                if let Some(engine_cmd) = self.convert_scenario_command(&event.command) {
                    commands.push(engine_cmd);
                }
                
                log::info!("Scenario event at {}ms: {}", elapsed, event.description);
                self.executed_events.push(event.clone());
                self.current_event += 1;
            } else {
                break;
            }
        }
        
        commands
    }
    
    /// Convert scenario command to engine command
    fn convert_scenario_command(&self, cmd: &ScenarioCommand) -> Option<EngineCommand> {
        match cmd {
            ScenarioCommand::SetThrottle(throttle) => Some(EngineCommand::SetThrottle(*throttle)),
            ScenarioCommand::SetRpm(rpm) => Some(EngineCommand::SetRpm(*rpm)),
            ScenarioCommand::SetDriveMode(mode) => Some(EngineCommand::SetDriveMode(*mode)),
            ScenarioCommand::ApplyTorqueLimit(limit) => Some(EngineCommand::ApplyTorqueLimit(*limit)),
            ScenarioCommand::ResetToIdle => Some(EngineCommand::ResetToIdle),
            ScenarioCommand::ForceOverboost => Some(EngineCommand::ForceOverboost),
            ScenarioCommand::SimulateKnock => Some(EngineCommand::SimulateKnock),
            
            // These commands need to be handled by the main simulation loop
            ScenarioCommand::StartCalibration { .. } => None,
            ScenarioCommand::SwitchProfile(_) => None,
            ScenarioCommand::Wait(_) => None,
        }
    }
    
    /// Reset scenario runner for new scenario
    pub fn reset(&mut self) {
        self.current_event = 0;
        self.executed_events.clear();
        self.start_time = None;
    }
}

impl TestScenario {
    /// Idle system test
    pub fn idle_test() -> Self {
        Self {
            name: "Idle Test".to_string(),
            description: "System idle behavior validation".to_string(),
            duration: 30,
            events: vec![
                ScenarioEvent {
                    time_ms: 0,
                    command: ScenarioCommand::ResetToIdle,
                    description: "Initialize to idle state".to_string(),
                },
                ScenarioEvent {
                    time_ms: 5000,
                    command: ScenarioCommand::SetRpm(800),
                    description: "Maintain idle RPM".to_string(),
                },
            ],
            success_criteria: vec![
                SuccessCriterion {
                    time_range: (5000, 30000),
                    condition: SuccessCondition::SystemState("Idle".to_string()),
                    description: "System should remain in idle state".to_string(),
                },
                SuccessCriterion {
                    time_range: (5000, 30000),
                    condition: SuccessCondition::BoostPressureRange(-1.0, 1.0),
                    description: "Boost pressure should be near zero".to_string(),
                },
            ],
        }
    }
    
    /// Wide-open throttle run test
    pub fn wot_run() -> Self {
        Self {
            name: "WOT Run".to_string(),
            description: "Full throttle boost control test".to_string(),
            duration: 15,
            events: vec![
                ScenarioEvent {
                    time_ms: 0,
                    command: ScenarioCommand::ResetToIdle,
                    description: "Start at idle".to_string(),
                },
                ScenarioEvent {
                    time_ms: 2000,
                    command: ScenarioCommand::SetRpm(2500),
                    description: "Increase to 2500 RPM".to_string(),
                },
                ScenarioEvent {
                    time_ms: 3000,
                    command: ScenarioCommand::SetThrottle(100.0),
                    description: "Full throttle application".to_string(),
                },
                ScenarioEvent {
                    time_ms: 4000,
                    command: ScenarioCommand::SetRpm(4000),
                    description: "Reach 4000 RPM".to_string(),
                },
                ScenarioEvent {
                    time_ms: 8000,
                    command: ScenarioCommand::SetRpm(6000),
                    description: "High RPM operation".to_string(),
                },
                ScenarioEvent {
                    time_ms: 12000,
                    command: ScenarioCommand::SetThrottle(0.0),
                    description: "Release throttle".to_string(),
                },
            ],
            success_criteria: vec![
                SuccessCriterion {
                    time_range: (5000, 10000),
                    condition: SuccessCondition::SystemState("Armed".to_string()),
                    description: "System should be armed during WOT".to_string(),
                },
                SuccessCriterion {
                    time_range: (6000, 10000),
                    condition: SuccessCondition::BoostPressureRange(5.0, 12.0),
                    description: "Should achieve reasonable boost pressure".to_string(),
                },
                SuccessCriterion {
                    time_range: (0, 15000),
                    condition: SuccessCondition::NoOverboostEvents,
                    description: "No overboost events should occur".to_string(),
                },
            ],
        }
    }
    
    /// Overboost detection and recovery test
    pub fn overboost_test() -> Self {
        Self {
            name: "Overboost Test".to_string(),
            description: "Safety system overboost detection and recovery".to_string(),
            duration: 20,
            events: vec![
                ScenarioEvent {
                    time_ms: 0,
                    command: ScenarioCommand::SetRpm(3000),
                    description: "Set operating RPM".to_string(),
                },
                ScenarioEvent {
                    time_ms: 2000,
                    command: ScenarioCommand::SetThrottle(100.0),
                    description: "Full throttle".to_string(),
                },
                ScenarioEvent {
                    time_ms: 5000,
                    command: ScenarioCommand::ForceOverboost,
                    description: "Force overboost condition".to_string(),
                },
                ScenarioEvent {
                    time_ms: 12000,
                    command: ScenarioCommand::ResetToIdle,
                    description: "Allow recovery".to_string(),
                },
            ],
            success_criteria: vec![
                SuccessCriterion {
                    time_range: (6000, 10000),
                    condition: SuccessCondition::SystemState("OverboostCut".to_string()),
                    description: "System should detect and respond to overboost".to_string(),
                },
                SuccessCriterion {
                    time_range: (6000, 10000),
                    condition: SuccessCondition::DutyCycleRange(0.0, 0.1),
                    description: "Duty cycle should be cut to 0%".to_string(),
                },
                SuccessCriterion {
                    time_range: (15000, 20000),
                    condition: SuccessCondition::SystemState("Armed".to_string()),
                    description: "System should recover after overboost cleared".to_string(),
                },
            ],
        }
    }
    
    /// Auto-calibration test scenario
    pub fn calibration_test() -> Self {
        Self {
            name: "Calibration Test".to_string(),
            description: "Auto-calibration system validation".to_string(),
            duration: 45,
            events: vec![
                ScenarioEvent {
                    time_ms: 0,
                    command: ScenarioCommand::StartCalibration { rpm: 4000, boost: 8.0 },
                    description: "Start calibration at 4000 RPM, 8 PSI".to_string(),
                },
                ScenarioEvent {
                    time_ms: 2000,
                    command: ScenarioCommand::SetRpm(4000),
                    description: "Set target RPM".to_string(),
                },
                ScenarioEvent {
                    time_ms: 3000,
                    command: ScenarioCommand::SetThrottle(100.0),
                    description: "Full throttle for calibration".to_string(),
                },
                // Multiple calibration runs
                ScenarioEvent {
                    time_ms: 12000,
                    command: ScenarioCommand::SetThrottle(0.0),
                    description: "End run 1".to_string(),
                },
                ScenarioEvent {
                    time_ms: 15000,
                    command: ScenarioCommand::SetThrottle(100.0),
                    description: "Start run 2".to_string(),
                },
                ScenarioEvent {
                    time_ms: 25000,
                    command: ScenarioCommand::SetThrottle(0.0),
                    description: "End run 2".to_string(),
                },
                ScenarioEvent {
                    time_ms: 28000,
                    command: ScenarioCommand::SetThrottle(100.0),
                    description: "Start run 3".to_string(),
                },
                ScenarioEvent {
                    time_ms: 38000,
                    command: ScenarioCommand::SetThrottle(0.0),
                    description: "End run 3".to_string(),
                },
            ],
            success_criteria: vec![
                SuccessCriterion {
                    time_range: (5000, 45000),
                    condition: SuccessCondition::SystemState("Calibrating".to_string()),
                    description: "System should be in calibration mode".to_string(),
                },
                SuccessCriterion {
                    time_range: (40000, 45000),
                    condition: SuccessCondition::CalibrationComplete,
                    description: "Calibration should complete successfully".to_string(),
                },
                SuccessCriterion {
                    time_range: (0, 45000),
                    condition: SuccessCondition::NoOverboostEvents,
                    description: "No overboost during calibration".to_string(),
                },
            ],
        }
    }
    
    /// Torque cooperation test
    pub fn torque_cooperation_test() -> Self {
        Self {
            name: "Torque Cooperation".to_string(),
            description: "ECU torque cooperation validation".to_string(),
            duration: 25,
            events: vec![
                ScenarioEvent {
                    time_ms: 0,
                    command: ScenarioCommand::SetRpm(3000),
                    description: "Operating RPM".to_string(),
                },
                ScenarioEvent {
                    time_ms: 2000,
                    command: ScenarioCommand::SetThrottle(75.0),
                    description: "Partial throttle".to_string(),
                },
                ScenarioEvent {
                    time_ms: 8000,
                    command: ScenarioCommand::ApplyTorqueLimit(300.0),
                    description: "Simulate traction control intervention".to_string(),
                },
                ScenarioEvent {
                    time_ms: 12000,
                    command: ScenarioCommand::ApplyTorqueLimit(600.0),
                    description: "Remove torque limit".to_string(),
                },
                ScenarioEvent {
                    time_ms: 15000,
                    command: ScenarioCommand::SetThrottle(100.0),
                    description: "Full throttle".to_string(),
                },
                ScenarioEvent {
                    time_ms: 20000,
                    command: ScenarioCommand::SetThrottle(0.0),
                    description: "Release throttle".to_string(),
                },
            ],
            success_criteria: vec![
                SuccessCriterion {
                    time_range: (3000, 25000),
                    condition: SuccessCondition::SystemState("Armed".to_string()),
                    description: "System should remain armed".to_string(),
                },
                SuccessCriterion {
                    time_range: (9000, 11000),
                    condition: SuccessCondition::TorqueErrorBelow(50.0),
                    description: "Should respond to torque limit".to_string(),
                },
                SuccessCriterion {
                    time_range: (0, 25000),
                    condition: SuccessCondition::NoOverboostEvents,
                    description: "No overboost during cooperation test".to_string(),
                },
            ],
        }
    }
    
    /// Profile switching test
    pub fn profile_switching_test() -> Self {
        Self {
            name: "Profile Switching".to_string(),
            description: "Test switching between boost profiles".to_string(),
            duration: 30,
            events: vec![
                ScenarioEvent {
                    time_ms: 0,
                    command: ScenarioCommand::SwitchProfile("daily".to_string()),
                    description: "Start with daily profile".to_string(),
                },
                ScenarioEvent {
                    time_ms: 2000,
                    command: ScenarioCommand::SetRpm(4000),
                    description: "Set operating RPM".to_string(),
                },
                ScenarioEvent {
                    time_ms: 3000,
                    command: ScenarioCommand::SetThrottle(100.0),
                    description: "Full throttle".to_string(),
                },
                ScenarioEvent {
                    time_ms: 8000,
                    command: ScenarioCommand::SetThrottle(0.0),
                    description: "Release throttle".to_string(),
                },
                ScenarioEvent {
                    time_ms: 10000,
                    command: ScenarioCommand::SwitchProfile("aggressive".to_string()),
                    description: "Switch to aggressive profile".to_string(),
                },
                ScenarioEvent {
                    time_ms: 12000,
                    command: ScenarioCommand::SetThrottle(100.0),
                    description: "Full throttle with aggressive profile".to_string(),
                },
                ScenarioEvent {
                    time_ms: 18000,
                    command: ScenarioCommand::SetThrottle(0.0),
                    description: "Release throttle".to_string(),
                },
                ScenarioEvent {
                    time_ms: 20000,
                    command: ScenarioCommand::SwitchProfile("valet".to_string()),
                    description: "Switch to valet mode".to_string(),
                },
                ScenarioEvent {
                    time_ms: 22000,
                    command: ScenarioCommand::SetThrottle(100.0),
                    description: "Full throttle with valet profile".to_string(),
                },
                ScenarioEvent {
                    time_ms: 28000,
                    command: ScenarioCommand::SetThrottle(0.0),
                    description: "Release throttle".to_string(),
                },
            ],
            success_criteria: vec![
                SuccessCriterion {
                    time_range: (5000, 7000),
                    condition: SuccessCondition::BoostPressureRange(6.0, 9.0),
                    description: "Daily profile boost range".to_string(),
                },
                SuccessCriterion {
                    time_range: (14000, 16000),
                    condition: SuccessCondition::BoostPressureRange(8.0, 12.0),
                    description: "Aggressive profile higher boost".to_string(),
                },
                SuccessCriterion {
                    time_range: (24000, 26000),
                    condition: SuccessCondition::BoostPressureRange(0.0, 3.0),
                    description: "Valet profile limited boost".to_string(),
                },
            ],
        }
    }
}

impl Default for ScenarioRunner {
    fn default() -> Self {
        Self::new()
    }
}