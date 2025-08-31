//! RumbleDome Desktop Simulator
//! 
//! Interactive desktop simulation environment for testing RumbleDome control logic
//! without hardware dependencies. Supports various test scenarios and real-time
//! monitoring of system behavior.

mod engine_sim;
mod scenarios;
mod ui;
mod config;

use clap::{Arg, Command};
use rumbledome_core::{RumbleDomeCore, SystemConfig};
use rumbledome_hal::{MockHal, SystemInputs};
use std::time::{Duration, Instant};
use tokio::time;
use anyhow::Result;

use engine_sim::EngineSimulator;
use scenarios::{TestScenario, ScenarioRunner};
use ui::SimulatorUI;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Parse command line arguments
    let matches = Command::new("RumbleDome Simulator")
        .version("1.0")
        .author("RumbleDome Team")
        .about("Desktop simulator for RumbleDome electronic boost controller")
        .arg(Arg::new("scenario")
            .short('s')
            .long("scenario")
            .value_name("SCENARIO")
            .help("Run specific test scenario")
            .value_parser([
                "idle", "wot_run", "overboost_test", "calibration_test", 
                "torque_cooperation", "profile_switching", "interactive"
            ])
            .default_value("interactive"))
        .arg(Arg::new("duration")
            .short('d')
            .long("duration")
            .value_name("SECONDS")
            .help("Simulation duration in seconds")
            .value_parser(clap::value_parser!(u64))
            .default_value("60"))
        .arg(Arg::new("frequency")
            .short('f')
            .long("frequency")
            .value_name("HZ")
            .help("Control loop frequency")
            .value_parser(clap::value_parser!(u16))
            .default_value("100"))
        .get_matches();
    
    let scenario_name = matches.get_one::<String>("scenario").unwrap();
    let duration = *matches.get_one::<u64>("duration").unwrap();
    let frequency = *matches.get_one::<u16>("frequency").unwrap();
    
    log::info!("Starting RumbleDome Simulator");
    log::info!("Scenario: {}, Duration: {}s, Frequency: {}Hz", 
        scenario_name, duration, frequency);
    
    if scenario_name == "interactive" {
        run_interactive_simulation(frequency).await
    } else {
        run_scenario_simulation(scenario_name, duration, frequency).await
    }
}

/// Run interactive simulation with TUI
async fn run_interactive_simulation(frequency: u16) -> Result<()> {
    log::info!("Starting interactive simulation");
    
    // Initialize components
    let config = SystemConfig::default();
    let hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, config)?;
    let mut engine_sim = EngineSimulator::new();
    let mut ui = SimulatorUI::new()?;
    
    // Initialize system
    core.initialize()?;
    
    let control_interval = Duration::from_millis(1000 / frequency as u64);
    let mut last_control = Instant::now();
    let mut simulation_time = 0u64;
    
    log::info!("Interactive simulation started - Press 'q' to quit");
    
    loop {
        let now = Instant::now();
        
        // Process UI events
        if ui.should_quit() {
            break;
        }
        
        let ui_commands = ui.process_events()?;
        
        // Apply UI commands to engine simulator
        for command in ui_commands {
            engine_sim.apply_command(command);
        }
        
        // Execute control loop at specified frequency
        if now.duration_since(last_control) >= control_interval {
            // Update engine simulation
            engine_sim.update(control_interval, core.get_state());
            
            // Get current solenoid duty from previous cycle
            let current_duty = ui.get_current_duty();
            
            // Create system inputs from simulation
            let inputs = create_system_inputs(&engine_sim, current_duty, simulation_time);
            
            // Execute control cycle
            match core.execute_control_cycle() {
                Ok(_) => {
                    ui.update_system_status(core.get_state(), &inputs);
                },
                Err(e) => {
                    log::error!("Control cycle error: {}", e);
                    ui.show_error(&e.to_string());
                }
            }
            
            last_control = now;
            simulation_time += control_interval.as_millis() as u64;
        }
        
        // Update UI display
        ui.render()?;
        
        // Small delay to prevent excessive CPU usage
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    ui.cleanup()?;
    log::info!("Interactive simulation ended");
    Ok(())
}

/// Run automated scenario simulation
async fn run_scenario_simulation(scenario_name: &str, duration: u64, frequency: u16) -> Result<()> {
    log::info!("Running scenario: {}", scenario_name);
    
    // Initialize components
    let config = SystemConfig::default();
    let hal = MockHal::new();
    let mut core = RumbleDomeCore::new(hal, config)?;
    let mut engine_sim = EngineSimulator::new();
    let mut scenario_runner = ScenarioRunner::new();
    
    // Initialize system
    core.initialize()?;
    
    // Load test scenario
    let scenario = match scenario_name {
        "idle" => TestScenario::idle_test(),
        "wot_run" => TestScenario::wot_run(),
        "overboost_test" => TestScenario::overboost_test(),
        "calibration_test" => TestScenario::calibration_test(),
        "torque_cooperation" => TestScenario::torque_cooperation_test(),
        "profile_switching" => TestScenario::profile_switching_test(),
        _ => return Err(anyhow::anyhow!("Unknown scenario: {}", scenario_name)),
    };
    
    log::info!("Scenario loaded: {}", scenario.name);
    log::info!("Description: {}", scenario.description);
    
    let control_interval = Duration::from_millis(1000 / frequency as u64);
    let total_cycles = duration * frequency as u64;
    let mut cycle_count = 0u64;
    
    println!("Starting simulation...");
    println!("Scenario: {} ({})", scenario.name, scenario.description);
    println!("Duration: {}s at {}Hz ({} cycles)", duration, frequency, total_cycles);
    println!();
    
    let start_time = Instant::now();
    let mut last_status = Instant::now();
    
    while cycle_count < total_cycles {
        let simulation_time_ms = (cycle_count * 1000) / frequency as u64;
        
        // Update scenario state
        let scenario_commands = scenario_runner.update_scenario(&scenario, simulation_time_ms);
        
        // Apply scenario commands to engine simulator
        for command in scenario_commands {
            engine_sim.apply_command(command);
        }
        
        // Update engine simulation
        engine_sim.update(control_interval, core.get_state());
        
        // Create system inputs from simulation
        let inputs = create_system_inputs(&engine_sim, 0.0, simulation_time_ms);
        
        // Execute control cycle
        match core.execute_control_cycle() {
            Ok(_) => {
                // Control cycle successful
            },
            Err(e) => {
                log::error!("Control cycle error at {}ms: {}", simulation_time_ms, e);
                println!("ERROR at {:.1}s: {}", simulation_time_ms as f32 / 1000.0, e);
            }
        }
        
        // Print status every second
        if last_status.elapsed() >= Duration::from_secs(1) {
            let progress = (cycle_count as f32 / total_cycles as f32) * 100.0;
            let sim_time = simulation_time_ms as f32 / 1000.0;
            let state_desc = core.get_state().description();
            
            println!("[{:5.1}%] {:.1}s - State: {} - RPM: {} - Boost: {:.1} PSI", 
                progress, sim_time, state_desc, 
                engine_sim.get_rpm(), engine_sim.get_boost_pressure());
            
            last_status = Instant::now();
        }
        
        cycle_count += 1;
        
        // Simulate real-time execution
        tokio::time::sleep(control_interval).await;
    }
    
    let elapsed = start_time.elapsed();
    println!();
    println!("Simulation completed!");
    println!("Total time: {:.2}s", elapsed.as_secs_f32());
    println!("Final state: {}", core.get_state().description());
    println!("Cycles executed: {}", cycle_count);
    
    // Generate summary report
    generate_simulation_report(&core, &engine_sim, &scenario)?;
    
    Ok(())
}

/// Create SystemInputs from engine simulator state
fn create_system_inputs(engine_sim: &EngineSimulator, current_duty: f32, timestamp_ms: u64) -> SystemInputs {
    use rumbledome_hal::{SensorReadings, CanData, SystemInputs};
    
    let sensors = SensorReadings {
        dome_input_pressure: 15.0, // Simulate 15 PSI air supply
        upper_dome_pressure: engine_sim.get_dome_pressure(current_duty),
        manifold_pressure_gauge: engine_sim.get_boost_pressure(),
        timestamp_ms,
    };
    
    let can = CanData {
        rpm: engine_sim.get_rpm(),
        map_kpa: (engine_sim.get_boost_pressure() + 14.7) * 6.895, // Convert to kPa absolute
        desired_torque: engine_sim.get_desired_torque(),
        actual_torque: engine_sim.get_actual_torque(),
        throttle_position: Some(engine_sim.get_throttle_position()),
        drive_mode: engine_sim.get_drive_mode(),
        timestamp_ms,
    };
    
    SystemInputs {
        sensors,
        can,
        timestamp_ms,
    }
}

/// Generate simulation summary report
fn generate_simulation_report(
    core: &RumbleDomeCore<MockHal>, 
    engine_sim: &EngineSimulator,
    scenario: &TestScenario
) -> Result<()> {
    println!();
    println!("=== SIMULATION REPORT ===");
    println!("Scenario: {}", scenario.name);
    println!("Final System State: {}", core.get_state().description());
    println!("Final Engine State:");
    println!("  RPM: {}", engine_sim.get_rpm());
    println!("  Boost Pressure: {:.2} PSI", engine_sim.get_boost_pressure());
    println!("  Throttle Position: {:.1}%", engine_sim.get_throttle_position());
    
    // TODO: Add more detailed statistics
    // - Control loop performance metrics  
    // - Safety event counts
    // - Learning system progress
    // - Profile usage statistics
    
    Ok(())
}