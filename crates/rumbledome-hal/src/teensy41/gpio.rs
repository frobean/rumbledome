//! GPIO implementation for Teensy 4.1
//! 
//! Provides button input and LED output control using the
//! i.MX RT1062 GPIO modules.

use crate::traits::GpioController;
use crate::types::{ButtonState, LedState, GpioPin};
use crate::error::HalError;

use teensy4_bsp::{hal, board};
use hal::gpio::{Input, Output, GPIO};

/// Button debounce time in milliseconds
const BUTTON_DEBOUNCE_MS: u32 = 20;

/// LED blink patterns
const BLINK_FAST_MS: u32 = 200;
const BLINK_SLOW_MS: u32 = 1000;
const BLINK_HEARTBEAT_ON_MS: u32 = 100;
const BLINK_HEARTBEAT_OFF_MS: u32 = 900;

/// Teensy 4.1 GPIO implementation
pub struct Teensy41Gpio {
    /// Mode selection button (Pin 2)
    mode_button: GPIO<Input, 2>,
    
    /// Profile selection button (Pin 3) 
    profile_button: GPIO<Input, 3>,
    
    /// Emergency stop button (Pin 4)
    estop_button: GPIO<Input, 4>,
    
    /// Status LED (Pin 13 - onboard)
    status_led: GPIO<Output, 13>,
    
    /// Error LED (Pin 12)
    error_led: GPIO<Output, 12>,
    
    /// Activity LED (Pin 11)
    activity_led: GPIO<Output, 11>,
    
    /// Button states and debouncing
    button_states: [ButtonInputState; 3],
    
    /// LED states and timing
    led_states: [LedOutputState; 3],
    
    /// Last update timestamp
    last_update_ms: u32,
}

/// Button input state tracking
#[derive(Clone, Copy)]
struct ButtonInputState {
    /// Current debounced state
    current_state: ButtonState,
    
    /// Raw input state
    raw_state: bool,
    
    /// Debounce timer
    debounce_timer_ms: u32,
    
    /// Press event flag
    pressed_event: bool,
    
    /// Release event flag  
    released_event: bool,
}

/// LED output state tracking
#[derive(Clone, Copy)]
struct LedOutputState {
    /// Current LED state
    state: LedState,
    
    /// Blink timer
    blink_timer_ms: u32,
    
    /// Current physical output state
    output_state: bool,
    
    /// Blink phase (for heartbeat pattern)
    blink_phase: bool,
}

impl ButtonInputState {
    fn new() -> Self {
        Self {
            current_state: ButtonState::Released,
            raw_state: false,
            debounce_timer_ms: 0,
            pressed_event: false,
            released_event: false,
        }
    }
    
    /// Update button state with debouncing
    fn update(&mut self, raw_input: bool, elapsed_ms: u32) {
        // Update debounce timer
        if self.debounce_timer_ms > 0 {
            self.debounce_timer_ms = self.debounce_timer_ms.saturating_sub(elapsed_ms);
        }
        
        // Check for state change
        if raw_input != self.raw_state {
            self.raw_state = raw_input;
            self.debounce_timer_ms = BUTTON_DEBOUNCE_MS;
        }
        
        // Update debounced state when timer expires
        if self.debounce_timer_ms == 0 {
            let new_state = if self.raw_state { 
                ButtonState::Pressed 
            } else { 
                ButtonState::Released 
            };
            
            // Detect state transitions
            match (self.current_state, new_state) {
                (ButtonState::Released, ButtonState::Pressed) => {
                    self.pressed_event = true;
                },
                (ButtonState::Pressed, ButtonState::Released) => {
                    self.released_event = true;
                },
                _ => {}
            }
            
            self.current_state = new_state;
        }
    }
    
    /// Check and clear press event
    fn take_press_event(&mut self) -> bool {
        if self.pressed_event {
            self.pressed_event = false;
            true
        } else {
            false
        }
    }
    
    /// Check and clear release event  
    fn take_release_event(&mut self) -> bool {
        if self.released_event {
            self.released_event = false;
            true
        } else {
            false
        }
    }
}

impl LedOutputState {
    fn new() -> Self {
        Self {
            state: LedState::Off,
            blink_timer_ms: 0,
            output_state: false,
            blink_phase: false,
        }
    }
    
    /// Update LED state and blink timing
    fn update(&mut self, elapsed_ms: u32) -> bool {
        let previous_output = self.output_state;
        
        match self.state {
            LedState::Off => {
                self.output_state = false;
            },
            LedState::On => {
                self.output_state = true;
            },
            LedState::BlinkSlow => {
                self.blink_timer_ms = self.blink_timer_ms.saturating_sub(elapsed_ms);
                if self.blink_timer_ms == 0 {
                    self.output_state = !self.output_state;
                    self.blink_timer_ms = BLINK_SLOW_MS;
                }
            },
            LedState::BlinkFast => {
                self.blink_timer_ms = self.blink_timer_ms.saturating_sub(elapsed_ms);
                if self.blink_timer_ms == 0 {
                    self.output_state = !self.output_state;
                    self.blink_timer_ms = BLINK_FAST_MS;
                }
            },
            LedState::Heartbeat => {
                self.blink_timer_ms = self.blink_timer_ms.saturating_sub(elapsed_ms);
                if self.blink_timer_ms == 0 {
                    if self.blink_phase {
                        // Currently on phase, switch to off
                        self.output_state = false;
                        self.blink_timer_ms = BLINK_HEARTBEAT_OFF_MS;
                        self.blink_phase = false;
                    } else {
                        // Currently off phase, switch to on
                        self.output_state = true;
                        self.blink_timer_ms = BLINK_HEARTBEAT_ON_MS;
                        self.blink_phase = true;
                    }
                }
            }
        }
        
        // Return true if output state changed
        self.output_state != previous_output
    }
    
    /// Set new LED state
    fn set_state(&mut self, new_state: LedState) {
        if new_state != self.state {
            self.state = new_state;
            
            // Initialize blink timing for new state
            match new_state {
                LedState::Off => {
                    self.output_state = false;
                },
                LedState::On => {
                    self.output_state = true;
                },
                LedState::BlinkSlow => {
                    self.blink_timer_ms = BLINK_SLOW_MS;
                    self.output_state = true;
                },
                LedState::BlinkFast => {
                    self.blink_timer_ms = BLINK_FAST_MS;
                    self.output_state = true;
                },
                LedState::Heartbeat => {
                    self.blink_timer_ms = BLINK_HEARTBEAT_ON_MS;
                    self.output_state = true;
                    self.blink_phase = true;
                }
            }
        }
    }
}

impl Teensy41Gpio {
    /// Create new GPIO controller
    pub fn new(pins: board::t41::Pins) -> Result<Self, HalError> {
        
        // Configure button inputs with pull-ups
        let mode_button = pins.p2.into_gpio().into_input()
            .into_pullup()
            .map_err(|e| HalError::gpio_error(format!("Mode button config failed: {:?}", e)))?;
        
        let profile_button = pins.p3.into_gpio().into_input()
            .into_pullup()
            .map_err(|e| HalError::gpio_error(format!("Profile button config failed: {:?}", e)))?;
        
        let estop_button = pins.p4.into_gpio().into_input()
            .into_pullup()
            .map_err(|e| HalError::gpio_error(format!("E-stop button config failed: {:?}", e)))?;
        
        // Configure LED outputs
        let mut status_led = pins.p13.into_gpio().into_output();
        let mut error_led = pins.p12.into_gpio().into_output();
        let mut activity_led = pins.p11.into_gpio().into_output();
        
        // Initialize LEDs to off state
        status_led.set_low();
        error_led.set_low();
        activity_led.set_low();
        
        log::info!("GPIO initialized: 3 buttons, 3 LEDs");
        
        Ok(Self {
            mode_button,
            profile_button,
            estop_button,
            status_led,
            error_led,
            activity_led,
            button_states: [ButtonInputState::new(); 3],
            led_states: [LedOutputState::new(); 3],
            last_update_ms: 0,
        })
    }
    
    /// Update all GPIO states (call regularly from main loop)
    pub fn update(&mut self, current_time_ms: u32) -> Result<(), HalError> {
        let elapsed_ms = current_time_ms.saturating_sub(self.last_update_ms);
        self.last_update_ms = current_time_ms;
        
        // Update button states
        self.button_states[0].update(!self.mode_button.is_set(), elapsed_ms);
        self.button_states[1].update(!self.profile_button.is_set(), elapsed_ms);
        self.button_states[2].update(!self.estop_button.is_set(), elapsed_ms);
        
        // Update LED states and physical outputs
        if self.led_states[0].update(elapsed_ms) {
            if self.led_states[0].output_state {
                self.status_led.set_high();
            } else {
                self.status_led.set_low();
            }
        }
        
        if self.led_states[1].update(elapsed_ms) {
            if self.led_states[1].output_state {
                self.error_led.set_high();
            } else {
                self.error_led.set_low();
            }
        }
        
        if self.led_states[2].update(elapsed_ms) {
            if self.led_states[2].output_state {
                self.activity_led.set_high();
            } else {
                self.activity_led.set_low();
            }
        }
        
        Ok(())
    }
    
    /// Get button index for GPIO pin
    fn get_button_index(&self, pin: GpioPin) -> Result<usize, HalError> {
        match pin {
            GpioPin::ModeButton => Ok(0),
            GpioPin::ProfileButton => Ok(1),
            GpioPin::EmergencyStop => Ok(2),
            _ => Err(HalError::invalid_parameter("Invalid button pin"))
        }
    }
    
    /// Get LED index for GPIO pin
    fn get_led_index(&self, pin: GpioPin) -> Result<usize, HalError> {
        match pin {
            GpioPin::StatusLed => Ok(0),
            GpioPin::ErrorLed => Ok(1),
            GpioPin::ActivityLed => Ok(2),
            _ => Err(HalError::invalid_parameter("Invalid LED pin"))
        }
    }
}

impl GpioController for Teensy41Gpio {
    fn read_button(&mut self, pin: GpioPin) -> Result<ButtonState, HalError> {
        let index = self.get_button_index(pin)?;
        Ok(self.button_states[index].current_state)
    }
    
    fn button_pressed(&mut self, pin: GpioPin) -> Result<bool, HalError> {
        let index = self.get_button_index(pin)?;
        Ok(self.button_states[index].take_press_event())
    }
    
    fn button_released(&mut self, pin: GpioPin) -> Result<bool, HalError> {
        let index = self.get_button_index(pin)?;
        Ok(self.button_states[index].take_release_event())
    }
    
    fn set_led(&mut self, pin: GpioPin, state: LedState) -> Result<(), HalError> {
        let index = self.get_led_index(pin)?;
        self.led_states[index].set_state(state);
        
        log::trace!("LED {:?} set to {:?}", pin, state);
        
        Ok(())
    }
    
    fn get_led_state(&self, pin: GpioPin) -> Result<LedState, HalError> {
        let index = self.get_led_index(pin)?;
        Ok(self.led_states[index].state)
    }
}

/// Button event handler for easy integration
pub struct ButtonEventHandler {
    /// Callback functions for each button
    mode_callback: Option<fn()>,
    profile_callback: Option<fn()>,
    estop_callback: Option<fn()>,
}

impl ButtonEventHandler {
    /// Create new button event handler
    pub fn new() -> Self {
        Self {
            mode_callback: None,
            profile_callback: None,
            estop_callback: None,
        }
    }
    
    /// Set callback for mode button press
    pub fn on_mode_button(&mut self, callback: fn()) {
        self.mode_callback = Some(callback);
    }
    
    /// Set callback for profile button press
    pub fn on_profile_button(&mut self, callback: fn()) {
        self.profile_callback = Some(callback);
    }
    
    /// Set callback for emergency stop button press
    pub fn on_estop_button(&mut self, callback: fn()) {
        self.estop_callback = Some(callback);
    }
    
    /// Process button events (call after GPIO update)
    pub fn process_events(&self, gpio: &mut Teensy41Gpio) -> Result<(), HalError> {
        if gpio.button_pressed(GpioPin::ModeButton)? {
            if let Some(callback) = self.mode_callback {
                callback();
            }
        }
        
        if gpio.button_pressed(GpioPin::ProfileButton)? {
            if let Some(callback) = self.profile_callback {
                callback();
            }
        }
        
        if gpio.button_pressed(GpioPin::EmergencyStop)? {
            if let Some(callback) = self.estop_callback {
                callback();
            }
        }
        
        Ok(())
    }
}

impl Default for ButtonEventHandler {
    fn default() -> Self {
        Self::new()
    }
}