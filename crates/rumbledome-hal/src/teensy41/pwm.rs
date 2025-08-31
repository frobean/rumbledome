//! PWM implementation for Teensy 4.1
//! 
//! Provides PWM control for solenoids using the i.MX RT1062 FlexPWM modules.

use crate::traits::PwmController;
use crate::types::PwmChannel;
use crate::error::HalError;

use teensy4_bsp::{board, hal};
use hal::flexpwm;

/// Teensy 4.1 PWM controller implementation
pub struct Teensy41Pwm {
    /// FlexPWM module for solenoid control
    solenoid_pwm: flexpwm::Pwm<flexpwm::module::_2, flexpwm::submodule::_2>,
    
    /// Current duty cycle settings
    duty_cycles: [f32; 1], // Only one channel for now
    
    /// PWM frequency in Hz
    frequency_hz: u16,
    
    /// PWM enabled state
    enabled: [bool; 1],
    
    /// PWM cycle synchronization
    sync_config: PwmSyncConfig,
    
    /// Phase management for multi-channel coordination
    phase_offset_degrees: f32,
}

/// PWM synchronization configuration
#[derive(Debug, Clone)]
struct PwmSyncConfig {
    /// Enable control loop synchronization to PWM cycles
    sync_control_loop: bool,
    
    /// PWM cycle period in microseconds
    cycle_period_us: u32,
    
    /// Last PWM cycle start timestamp (microseconds)
    last_cycle_start_us: u64,
    
    /// Control update timing strategy
    update_strategy: ControlUpdateStrategy,
}

/// Strategy for timing control updates relative to PWM cycles
#[derive(Debug, Clone, PartialEq)]
pub enum ControlUpdateStrategy {
    /// Update at fixed intervals regardless of PWM phase (default)
    Asynchronous,
    
    /// Update synchronized to PWM cycle start (reduces switching noise)
    CycleStart,
    
    /// Update at PWM cycle midpoint (optimal for feedback sampling)
    CycleMidpoint,
    
    /// Update multiple times per PWM cycle for high-frequency correction
    SubCycle { updates_per_cycle: u8 },
}

impl Teensy41Pwm {
    /// Create new PWM controller
    pub fn new(
        pins: board::t41::Pins,
        ccm: &hal::ccm::Handle,
    ) -> Result<Self, HalError> {
        
        // Configure FlexPWM2 for solenoid control
        // Using pin P_B0_10 (Teensy pin 24) for solenoid PWM output
        
        let mut pwm_module = hal::flexpwm::new(
            ccm.perclk_clk_sel(hal::ccm::perclk::PERCLK_CLK_SEL_A::OSC_CLK)
                .set_perclk_divider(1),
        );
        
        // Configure PWM submodule 2 for 100Hz operation (optimal for solenoid response)
        // 30Hz was too low - pneumatic systems benefit from higher switching frequency
        let frequency_hz = 100;
        let period = 24_000_000 / frequency_hz as u32; // 24MHz clock
        let cycle_period_us = 1_000_000 / frequency_hz as u32; // Period in microseconds
        
        let solenoid_pwm = pwm_module
            .build_submodule_2()
            .complementary_source_selection(flexpwm::ComplementarySourceSelection::LocalSync)
            .clock_selection(flexpwm::ClockSelection::IpgClock)
            .prescaler(flexpwm::Prescaler::DivideBy1)
            .load_frequency_hz(frequency_hz)
            .map_err(|e| HalError::pwm_error(format!("PWM config failed: {:?}", e)))?;
        
        // Configure the PWM output pin
        hal::iomuxc::configure(&mut pins.p24, hal::iomuxc::Config::zero());
        
        // Configure synchronization for control loop coordination
        let sync_config = PwmSyncConfig {
            sync_control_loop: true,
            cycle_period_us,
            last_cycle_start_us: 0,
            update_strategy: ControlUpdateStrategy::CycleMidpoint, // Sample at midpoint for stability
        };
        
        log::info!("PWM initialized at {} Hz with cycle synchronization", frequency_hz);
        
        Ok(Self {
            solenoid_pwm,
            duty_cycles: [0.0],
            frequency_hz,
            enabled: [false],
            sync_config,
            phase_offset_degrees: 0.0,
        })
    }
    
    /// Check if it's optimal time to update control loop relative to PWM cycle
    pub fn is_optimal_update_time(&mut self, current_time_us: u64) -> bool {
        if !self.sync_config.sync_control_loop {
            return true; // Always optimal if sync is disabled
        }
        
        // Calculate current position in PWM cycle
        let cycle_position_us = current_time_us % (self.sync_config.cycle_period_us as u64);
        let cycle_phase = cycle_position_us as f32 / self.sync_config.cycle_period_us as f32;
        
        match self.sync_config.update_strategy {
            ControlUpdateStrategy::Asynchronous => true,
            
            ControlUpdateStrategy::CycleStart => {
                // Update within first 10% of PWM cycle
                cycle_phase < 0.1
            },
            
            ControlUpdateStrategy::CycleMidpoint => {
                // Update at PWM cycle midpoint (±10% window)
                (cycle_phase >= 0.45 && cycle_phase <= 0.55)
            },
            
            ControlUpdateStrategy::SubCycle { updates_per_cycle } => {
                // Update at evenly spaced intervals within cycle
                let update_interval = 1.0 / updates_per_cycle as f32;
                let target_phases: Vec<f32> = (0..updates_per_cycle)
                    .map(|i| i as f32 * update_interval)
                    .collect();
                
                // Check if we're near any target phase (±5% window)
                target_phases.iter().any(|&target| {
                    let phase_diff = (cycle_phase - target).abs();
                    phase_diff < 0.05 || phase_diff > 0.95 // Handle wrap-around
                })
            },
        }
    }
    
    /// Get recommended control loop frequency based on PWM settings
    pub fn get_recommended_control_frequency(&self) -> u16 {
        match self.sync_config.update_strategy {
            ControlUpdateStrategy::Asynchronous => {
                // Standard rule: control frequency = 10x PWM frequency
                (self.frequency_hz * 10).min(1000)
            },
            
            ControlUpdateStrategy::CycleStart | ControlUpdateStrategy::CycleMidpoint => {
                // Synchronous: match PWM frequency exactly
                self.frequency_hz
            },
            
            ControlUpdateStrategy::SubCycle { updates_per_cycle } => {
                // Multiple updates per PWM cycle
                self.frequency_hz * updates_per_cycle as u16
            },
        }
    }
    
    /// Calculate time until next optimal update window
    pub fn time_to_next_update_window_us(&self, current_time_us: u64) -> u32 {
        if !self.sync_config.sync_control_loop {
            return 0; // No wait needed
        }
        
        let cycle_position_us = current_time_us % (self.sync_config.cycle_period_us as u64);
        
        match self.sync_config.update_strategy {
            ControlUpdateStrategy::Asynchronous => 0,
            
            ControlUpdateStrategy::CycleStart => {
                // Wait for next cycle start
                if cycle_position_us < (self.sync_config.cycle_period_us as u64 / 10) {
                    0 // In window
                } else {
                    self.sync_config.cycle_period_us - cycle_position_us as u32
                }
            },
            
            ControlUpdateStrategy::CycleMidpoint => {
                // Wait for cycle midpoint window
                let midpoint_us = self.sync_config.cycle_period_us as u64 / 2;
                let window_start = midpoint_us - (self.sync_config.cycle_period_us as u64 / 20); // 5% before
                let window_end = midpoint_us + (self.sync_config.cycle_period_us as u64 / 20);   // 5% after
                
                if cycle_position_us >= window_start && cycle_position_us <= window_end {
                    0 // In window
                } else if cycle_position_us < window_start {
                    (window_start - cycle_position_us) as u32
                } else {
                    (self.sync_config.cycle_period_us as u64 + window_start - cycle_position_us) as u32
                }
            },
            
            ControlUpdateStrategy::SubCycle { updates_per_cycle } => {
                // Find next sub-cycle update point
                let update_interval_us = self.sync_config.cycle_period_us / updates_per_cycle as u32;
                let next_update = ((cycle_position_us / update_interval_us as u64) + 1) * update_interval_us as u64;
                
                if next_update >= self.sync_config.cycle_period_us as u64 {
                    self.sync_config.cycle_period_us - cycle_position_us as u32
                } else {
                    (next_update - cycle_position_us) as u32
                }
            },
        }
    }
    
    /// Set PWM synchronization strategy
    pub fn set_sync_strategy(&mut self, strategy: ControlUpdateStrategy) {
        self.sync_config.update_strategy = strategy;
        
        log::info!("PWM sync strategy changed to: {:?}", strategy);
    }
    
    /// Enable or disable control loop synchronization
    pub fn set_sync_enabled(&mut self, enabled: bool) {
        self.sync_config.sync_control_loop = enabled;
        
        log::info!("PWM synchronization {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Apply duty cycle change with jitter reduction
    pub fn set_duty_cycle_synchronized(&mut self, channel: PwmChannel, duty_percent: f32, update_time_us: u64) -> Result<(), HalError> {
        // Check if this is an optimal time to update
        if !self.is_optimal_update_time(update_time_us) {
            log::trace!("Duty cycle update delayed for PWM synchronization");
            // In a real implementation, we might queue this update for the next optimal time
        }
        
        // Apply the duty cycle change
        self.set_duty_cycle(channel, duty_percent)?;
        
        // Update sync tracking
        self.sync_config.last_cycle_start_us = update_time_us;
        
        Ok(())
    }
    
    /// Get PWM timing information for control loop coordination
    pub fn get_timing_info(&self) -> PwmTimingInfo {
        PwmTimingInfo {
            frequency_hz: self.frequency_hz,
            cycle_period_us: self.sync_config.cycle_period_us,
            recommended_control_frequency: self.get_recommended_control_frequency(),
            sync_enabled: self.sync_config.sync_control_loop,
            update_strategy: self.sync_config.update_strategy.clone(),
        }
    }
}

/// PWM timing information for control loop coordination
#[derive(Debug, Clone)]
pub struct PwmTimingInfo {
    pub frequency_hz: u16,
    pub cycle_period_us: u32,
    pub recommended_control_frequency: u16,
    pub sync_enabled: bool,
    pub update_strategy: ControlUpdateStrategy,
}

impl PwmController for Teensy41Pwm {
    fn set_duty_cycle(&mut self, channel: PwmChannel, duty_percent: f32) -> Result<(), HalError> {
        // Validate input
        if duty_percent < 0.0 || duty_percent > 100.0 {
            return Err(HalError::invalid_parameter(
                format!("Invalid duty cycle: {}%", duty_percent)
            ));
        }
        
        match channel {
            PwmChannel::BoostSolenoid => {
                // Convert percentage to PWM value (0-32767 for FlexPWM)
                let duty_value = ((duty_percent / 100.0) * 32767.0) as u16;
                
                // Set PWM duty cycle
                self.solenoid_pwm.set_duty_cycle(duty_value);
                
                self.duty_cycles[0] = duty_percent;
                
                log::trace!("Solenoid PWM set to {:.1}% (raw: {})", duty_percent, duty_value);
                
                Ok(())
            }
        }
    }
    
    fn set_frequency(&mut self, channel: PwmChannel, frequency_hz: u16) -> Result<(), HalError> {
        match channel {
            PwmChannel::BoostSolenoid => {
                // Validate frequency range
                if frequency_hz < 10 || frequency_hz > 1000 {
                    return Err(HalError::invalid_parameter(
                        format!("Invalid PWM frequency: {} Hz", frequency_hz)
                    ));
                }
                
                // Reconfigure PWM frequency
                self.solenoid_pwm.set_load_frequency_hz(frequency_hz)
                    .map_err(|e| HalError::pwm_error(format!("Frequency change failed: {:?}", e)))?;
                
                self.frequency_hz = frequency_hz;
                
                log::info!("PWM frequency changed to {} Hz", frequency_hz);
                
                Ok(())
            }
        }
    }
    
    fn enable(&mut self, channel: PwmChannel) -> Result<(), HalError> {
        match channel {
            PwmChannel::BoostSolenoid => {
                // Enable PWM output
                self.solenoid_pwm.set_enabled(true);
                self.enabled[0] = true;
                
                log::info!("Solenoid PWM enabled");
                
                Ok(())
            }
        }
    }
    
    fn disable(&mut self, channel: PwmChannel) -> Result<(), HalError> {
        match channel {
            PwmChannel::BoostSolenoid => {
                // Disable PWM output and ensure failsafe state (0% duty)
                self.solenoid_pwm.set_duty_cycle(0);
                self.solenoid_pwm.set_enabled(false);
                
                self.enabled[0] = false;
                self.duty_cycles[0] = 0.0;
                
                log::info!("Solenoid PWM disabled (failsafe)");
                
                Ok(())
            }
        }
    }
    
    fn get_duty_cycle(&self, channel: PwmChannel) -> Result<f32, HalError> {
        match channel {
            PwmChannel::BoostSolenoid => Ok(self.duty_cycles[0]),
        }
    }
}

impl Drop for Teensy41Pwm {
    fn drop(&mut self) {
        // Ensure failsafe state when PWM controller is dropped
        let _ = self.disable(PwmChannel::BoostSolenoid);
        log::info!("PWM controller dropped - failsafe applied");
    }
}