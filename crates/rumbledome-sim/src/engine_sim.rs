//! Engine and turbo system simulation
//! 
//! Provides realistic simulation of engine behavior, turbo response,
//! and ECU torque management for testing RumbleDome control logic.

use rumbledome_core::SystemState;
use rumbledome_hal::DriveMode;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Engine and turbo system simulator
pub struct EngineSimulator {
    /// Engine state
    engine: EngineState,
    
    /// Turbo system state
    turbo: TurboState,
    
    /// ECU simulation
    ecu: EcuSimulator,
    
    /// Physics parameters
    physics: PhysicsParameters,
}

/// Current engine operating state
#[derive(Debug, Clone)]
struct EngineState {
    /// Engine RPM
    rpm: u16,
    
    /// Throttle position (0-100%)
    throttle_position: f32,
    
    /// Engine load (0-100%)
    load: f32,
    
    /// Coolant temperature (°C)
    coolant_temp: f32,
    
    /// Intake air temperature (°C)  
    intake_temp: f32,
    
    /// Current drive mode
    drive_mode: Option<DriveMode>,
}

/// Turbo system state
#[derive(Debug, Clone)]
struct TurboState {
    /// Current boost pressure (PSI gauge)
    boost_pressure: f32,
    
    /// Turbo speed (RPM)
    turbo_rpm: u32,
    
    /// Wastegate position (0-100%, 0=closed, 100=fully open)
    wastegate_position: f32,
    
    /// Compressor efficiency
    compressor_efficiency: f32,
    
    /// Heat soak factor
    heat_soak: f32,
}

/// ECU simulation for torque management
#[derive(Debug, Clone)]
struct EcuSimulator {
    /// Desired torque output (Nm)
    desired_torque: f32,
    
    /// Actual torque output (Nm)
    actual_torque: f32,
    
    /// Torque request from driver (pedal position)
    driver_torque_request: f32,
    
    /// ECU torque ceiling (safety limit)
    torque_ceiling: f32,
    
    /// Ignition timing advance (degrees)
    ignition_timing: f32,
    
    /// Fuel enrichment factor
    fuel_enrichment: f32,
}

/// Physics simulation parameters
#[derive(Debug, Clone)]
struct PhysicsParameters {
    /// Engine displacement (L)
    displacement: f32,
    
    /// Turbo size factor (affects spool and response)
    turbo_size: f32,
    
    /// Wastegate spring pressure (PSI)
    spring_pressure: f32,
    
    /// Intercooler efficiency
    intercooler_efficiency: f32,
    
    /// Ambient temperature (°C)
    ambient_temp: f32,
    
    /// Atmospheric pressure (PSI absolute)
    atmospheric_pressure: f32,
}

/// Commands to control the engine simulation
#[derive(Debug, Clone)]
pub enum EngineCommand {
    /// Set throttle position (0-100%)
    SetThrottle(f32),
    
    /// Set engine RPM directly (for testing)
    SetRpm(u16),
    
    /// Set drive mode
    SetDriveMode(DriveMode),
    
    /// Apply external torque limit (safety systems)
    ApplyTorqueLimit(f32),
    
    /// Simulate engine knock (reduces timing)
    SimulateKnock,
    
    /// Reset to idle conditions
    ResetToIdle,
    
    /// Start WOT run
    StartWotRun,
    
    /// Simulate overboost condition
    ForceOverboost,
}

impl EngineSimulator {
    /// Create new engine simulator with realistic Gen2 Coyote parameters
    pub fn new() -> Self {
        Self {
            engine: EngineState {
                rpm: 800,           // Idle RPM
                throttle_position: 0.0,
                load: 10.0,         // Light idle load
                coolant_temp: 90.0, // Operating temperature
                intake_temp: 25.0,  // Ambient + some heat
                drive_mode: Some(DriveMode::Normal),
            },
            turbo: TurboState {
                boost_pressure: 0.0,
                turbo_rpm: 50000,   // Idle turbo speed
                wastegate_position: 100.0, // Fully open at idle
                compressor_efficiency: 0.65,
                heat_soak: 1.0,
            },
            ecu: EcuSimulator {
                desired_torque: 50.0,  // Light idle torque
                actual_torque: 50.0,
                driver_torque_request: 0.0,
                torque_ceiling: 600.0, // Gen2 Coyote limit
                ignition_timing: 15.0, // Base timing
                fuel_enrichment: 1.0,
            },
            physics: PhysicsParameters {
                displacement: 5.0,     // 5.0L Coyote
                turbo_size: 1.0,       // Normalized turbo size
                spring_pressure: 5.0,  // 5 PSI spring
                intercooler_efficiency: 0.85,
                ambient_temp: 20.0,
                atmospheric_pressure: 14.7,
            },
        }
    }
    
    /// Update engine simulation for one time step
    pub fn update(&mut self, dt: Duration, system_state: &SystemState) {
        let dt_sec = dt.as_secs_f32();
        
        // Update ECU torque management
        self.update_ecu_simulation(dt_sec);
        
        // Update turbo system physics
        self.update_turbo_physics(dt_sec);
        
        // Update engine state based on current conditions
        self.update_engine_state(dt_sec, system_state);
        
        // Apply realistic delays and filtering
        self.apply_system_dynamics(dt_sec);
    }
    
    /// Apply command to engine simulator
    pub fn apply_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::SetThrottle(throttle) => {
                self.engine.throttle_position = throttle.clamp(0.0, 100.0);
                // Convert throttle to torque request
                self.ecu.driver_torque_request = (throttle / 100.0) * self.ecu.torque_ceiling;
            },
            
            EngineCommand::SetRpm(rpm) => {
                self.engine.rpm = rpm;
            },
            
            EngineCommand::SetDriveMode(mode) => {
                self.engine.drive_mode = Some(mode);
            },
            
            EngineCommand::ApplyTorqueLimit(limit) => {
                self.ecu.torque_ceiling = limit.min(600.0);
            },
            
            EngineCommand::SimulateKnock => {
                // Reduce timing advance due to knock
                self.ecu.ignition_timing = (self.ecu.ignition_timing - 5.0).max(5.0);
            },
            
            EngineCommand::ResetToIdle => {
                self.engine.throttle_position = 0.0;
                self.ecu.driver_torque_request = 0.0;
                self.ecu.ignition_timing = 15.0;
            },
            
            EngineCommand::StartWotRun => {
                self.engine.throttle_position = 100.0;
                self.ecu.driver_torque_request = self.ecu.torque_ceiling;
            },
            
            EngineCommand::ForceOverboost => {
                // Simulate stuck wastegate or other overboost condition
                self.turbo.wastegate_position = 0.0; // Force wastegate closed
            },
        }
    }
    
    /// Update ECU torque management simulation
    fn update_ecu_simulation(&mut self, dt: f32) {
        // Calculate desired torque based on driver request and current conditions
        let mut base_torque = self.ecu.driver_torque_request;
        
        // Apply drive mode modulation
        match self.engine.drive_mode {
            Some(DriveMode::Normal) => {
                // Standard response
            },
            Some(DriveMode::Sport) => {
                // More aggressive response
                base_torque *= 1.1;
            },
            Some(DriveMode::SportPlus) => {
                // Very aggressive response
                base_torque *= 1.2;
            },
            Some(DriveMode::Track) => {
                // Maximum response
                base_torque *= 1.3;
            },
            None => {}
        }
        
        // Apply RPM-based torque curve (realistic for Coyote)
        let rpm_factor = if self.engine.rpm < 2000 {
            0.6 // Low-end torque reduction
        } else if self.engine.rpm < 4500 {
            1.0 // Peak torque band
        } else if self.engine.rpm < 6500 {
            0.9 // High RPM power band
        } else {
            0.7 // Over-rev protection
        };
        
        base_torque *= rpm_factor;
        
        // Apply torque ceiling
        self.ecu.desired_torque = base_torque.min(self.ecu.torque_ceiling);
        
        // Simulate ECU achieving desired torque (with some lag)
        let torque_error = self.ecu.desired_torque - self.ecu.actual_torque;
        let torque_rate = 200.0; // Nm/s response rate
        
        self.ecu.actual_torque += (torque_error * torque_rate * dt).clamp(-50.0 * dt, 50.0 * dt);
        self.ecu.actual_torque = self.ecu.actual_torque.max(0.0);
    }
    
    /// Update turbo system physics simulation
    fn update_turbo_physics(&mut self, dt: f32) {
        // Exhaust energy drives turbo (simplified model)
        let exhaust_energy = self.ecu.actual_torque * (self.engine.rpm as f32 / 1000.0);
        
        // Target turbo RPM based on exhaust energy
        let target_turbo_rpm = (50000.0 + exhaust_energy * 100.0).min(200000.0);
        
        // Turbo spool dynamics (lag simulation)
        let spool_rate = 50000.0; // RPM/s
        let turbo_error = target_turbo_rpm - self.turbo.turbo_rpm as f32;
        let turbo_delta = (turbo_error * spool_rate * dt / 100000.0).clamp(-5000.0 * dt, 5000.0 * dt);
        
        self.turbo.turbo_rpm = ((self.turbo.turbo_rpm as f32 + turbo_delta) as u32).max(30000);
        
        // Boost pressure from turbo speed and wastegate position
        let max_boost_potential = ((self.turbo.turbo_rpm as f32 - 50000.0) / 10000.0).max(0.0);
        let wastegate_factor = (100.0 - self.turbo.wastegate_position) / 100.0;
        
        let target_boost = (max_boost_potential * wastegate_factor).min(25.0); // Mechanical limit
        
        // Boost pressure dynamics
        let boost_rate = 10.0; // PSI/s
        let boost_error = target_boost - self.turbo.boost_pressure;
        self.turbo.boost_pressure += (boost_error * boost_rate * dt).clamp(-5.0 * dt, 5.0 * dt);
        self.turbo.boost_pressure = self.turbo.boost_pressure.max(0.0);
        
        // Update compressor efficiency based on operating point
        self.turbo.compressor_efficiency = if self.turbo.boost_pressure < 5.0 {
            0.6 // Low efficiency at low boost
        } else if self.turbo.boost_pressure < 15.0 {
            0.75 // Good efficiency in normal range
        } else {
            0.65 // Efficiency drops at high boost
        };
    }
    
    /// Update engine state based on current conditions
    fn update_engine_state(&mut self, dt: f32, _system_state: &SystemState) {
        // Engine load based on throttle and boost
        let boost_factor = 1.0 + (self.turbo.boost_pressure / 14.7);
        self.engine.load = (self.engine.throttle_position * boost_factor).min(100.0);
        
        // Intake air temperature (affected by boost and intercooler)
        let compression_heating = self.turbo.boost_pressure * 8.0; // °C per PSI
        let intercooled_temp = self.physics.ambient_temp + 
            (compression_heating * (1.0 - self.physics.intercooler_efficiency));
        
        // Exponential moving average for temperature
        let temp_rate = 2.0; // °C/s
        let temp_error = intercooled_temp - self.engine.intake_temp;
        self.engine.intake_temp += (temp_error * temp_rate * dt).clamp(-10.0 * dt, 10.0 * dt);
        
        // Coolant temperature (simplified)
        let target_coolant = 90.0 + (self.engine.load / 10.0);
        let coolant_error = target_coolant - self.engine.coolant_temp;
        self.engine.coolant_temp += (coolant_error * 0.5 * dt).clamp(-2.0 * dt, 2.0 * dt);
    }
    
    /// Apply realistic system dynamics and filtering
    fn apply_system_dynamics(&mut self, _dt: f32) {
        // Add some realistic noise to sensor readings
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Small amount of noise on RPM (±10 RPM)
        let rpm_noise: i16 = rng.gen_range(-10..=10);
        self.engine.rpm = ((self.engine.rpm as i16 + rpm_noise).max(0)) as u16;
        
        // Small noise on boost pressure (±0.1 PSI)
        let boost_noise: f32 = rng.gen_range(-0.1..=0.1);
        self.turbo.boost_pressure += boost_noise;
    }
    
    // Getter methods for simulator state
    
    pub fn get_rpm(&self) -> u16 {
        self.engine.rpm
    }
    
    pub fn get_boost_pressure(&self) -> f32 {
        self.turbo.boost_pressure
    }
    
    pub fn get_throttle_position(&self) -> f32 {
        self.engine.throttle_position
    }
    
    pub fn get_desired_torque(&self) -> f32 {
        self.ecu.desired_torque
    }
    
    pub fn get_actual_torque(&self) -> f32 {
        self.ecu.actual_torque
    }
    
    pub fn get_drive_mode(&self) -> Option<DriveMode> {
        self.engine.drive_mode
    }
    
    pub fn get_wastegate_position(&self) -> f32 {
        self.turbo.wastegate_position
    }
    
    pub fn get_turbo_rpm(&self) -> u32 {
        self.turbo.turbo_rpm
    }
    
    pub fn get_intake_temp(&self) -> f32 {
        self.engine.intake_temp
    }
    
    /// Calculate dome pressure based on duty cycle (for HAL simulation)
    pub fn get_dome_pressure(&self, duty_cycle: f32) -> f32 {
        let input_pressure = 15.0; // Simulated air supply pressure
        
        // Dome pressure calculation:
        // 0% duty = lower dome gets full pressure (wastegate open)
        // 100% duty = upper dome gets full pressure (wastegate closed)
        
        // Upper dome pressure varies with duty cycle
        let upper_dome_pressure = input_pressure * (duty_cycle / 100.0);
        
        // Wastegate position affects actual dome pressures
        let effective_pressure = upper_dome_pressure * (1.0 - self.turbo.wastegate_position / 100.0);
        
        effective_pressure
    }
}

impl Default for EngineSimulator {
    fn default() -> Self {
        Self::new()
    }
}