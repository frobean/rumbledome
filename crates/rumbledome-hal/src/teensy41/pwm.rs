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
        
        // Configure PWM submodule 2 for 30Hz operation
        let frequency_hz = 30;
        let period = 24_000_000 / frequency_hz as u32; // 24MHz clock
        
        let solenoid_pwm = pwm_module
            .build_submodule_2()
            .complementary_source_selection(flexpwm::ComplementarySourceSelection::LocalSync)
            .clock_selection(flexpwm::ClockSelection::IpgClock)
            .prescaler(flexpwm::Prescaler::DivideBy1)
            .load_frequency_hz(frequency_hz)
            .map_err(|e| HalError::pwm_error(format!("PWM config failed: {:?}", e)))?;
        
        // Configure the PWM output pin
        hal::iomuxc::configure(&mut pins.p24, hal::iomuxc::Config::zero());
        
        log::info!("PWM initialized at {} Hz", frequency_hz);
        
        Ok(Self {
            solenoid_pwm,
            duty_cycles: [0.0],
            frequency_hz,
            enabled: [false],
        })
    }
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