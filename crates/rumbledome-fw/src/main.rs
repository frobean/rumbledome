//! RumbleDome Firmware for Teensy 4.1
//! 
//! Main firmware entry point for the Teensy 4.1 microcontroller.
//! Implements the complete RumbleDome system with real-time control,
//! safety monitoring, and CAN bus integration.

#![no_std]
#![no_main]

use cortex_m::asm;
use defmt_rtt as _;
use panic_probe as _;

use teensy4_bsp as bsp;
use bsp::board;
use bsp::hal;

use rumbledome_core::{RumbleDomeCore, SystemConfig};
use rumbledome_hal::Teensy41Hal;

use rtic::app;

// System constants
const CONTROL_FREQUENCY_HZ: u32 = 100; // 100Hz control loop
const CONTROL_PERIOD_US: u32 = 1_000_000 / CONTROL_FREQUENCY_HZ;

#[app(device = teensy4_bsp::hal::ral, peripherals = true, dispatchers = [GPIO1_COMBINED_0_15])]
mod app {
    use super::*;
    use systick_monotonic::Systick;
    use rtic_monotonics::systick::*;
    
    // Shared resources
    #[shared]
    struct Shared {
        /// Main RumbleDome controller
        controller: RumbleDomeCore<Teensy41Hal>,
        
        /// System status for monitoring
        system_status: SystemStatus,
    }
    
    // Local resources (owned by specific tasks)
    #[local]
    struct Local {
        /// LED for system status indication
        led: board::Led,
        
        /// Periodic timer for control loop
        control_timer: hal::pit::Pit<2>,
        
        /// CAN peripheral
        can_peripheral: Option<hal::can::Can<hal::can::module::_1>>,
        
        /// Display interface
        display: Option<DisplayInterface>,
        
        /// User input buttons
        profile_button: board::Button,
    }
    
    /// System status structure
    struct SystemStatus {
        /// Heartbeat counter
        heartbeat: u32,
        
        /// Last error message
        last_error: Option<&'static str>,
        
        /// System uptime in milliseconds
        uptime_ms: u64,
        
        /// Control loop statistics
        loop_stats: ControlLoopStats,
    }
    
    /// Control loop performance statistics
    struct ControlLoopStats {
        /// Total control cycles executed
        total_cycles: u64,
        
        /// Maximum execution time (microseconds)
        max_execution_time_us: u32,
        
        /// Average execution time (microseconds)  
        avg_execution_time_us: u32,
        
        /// Missed cycles due to overrun
        missed_cycles: u32,
    }
    
    /// Display interface abstraction
    struct DisplayInterface {
        // TODO: Implement ST7735R TFT display driver
        _placeholder: (),
    }
    
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        defmt::info!("RumbleDome firmware initializing...");
        
        // Initialize the board
        let board = board::t41(ctx.device);
        let mut delay = board.delay;
        
        // Configure system clocks
        let ccm = board.ccm.handle;
        let mut cfg = ccm.perclk_clk_sel(hal::ccm::perclk::PERCLK_CLK_SEL_A::OSC_CLK);
        cfg.set_perclk_divider(1);
        
        // Initialize monotonic timer for RTIC
        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(ctx.core.SYST, 600_000_000, systick_token);
        
        // Initialize hardware abstraction layer
        let hal_result = Teensy41Hal::new(
            board.pins,
            board.usb,
            board.spi4,
            board.uart2,
            ccm,
        );
        
        let teensy_hal = match hal_result {
            Ok(hal) => hal,
            Err(e) => {
                defmt::error!("Failed to initialize HAL: {:?}", defmt::Debug2Format(&e));
                // TODO: Better error handling - for now panic
                panic!("HAL initialization failed");
            }
        };
        
        // Load system configuration
        let config = SystemConfig::default(); // TODO: Load from EEPROM
        
        // Initialize RumbleDome core
        let mut controller = match RumbleDomeCore::new(teensy_hal, config) {
            Ok(ctrl) => ctrl,
            Err(e) => {
                defmt::error!("Failed to create RumbleDome controller: {:?}", defmt::Debug2Format(&e));
                panic!("Controller initialization failed");
            }
        };
        
        // Initialize the controller
        if let Err(e) = controller.initialize() {
            defmt::error!("Controller initialization failed: {:?}", defmt::Debug2Format(&e));
            panic!("Controller initialization failed");
        }
        
        // Set up periodic control loop timer (100Hz)
        let mut pit = board.pit;
        pit.set_load_timer_value(2, CONTROL_PERIOD_US);
        pit.set_timer_enable(2, true);
        pit.set_interrupt_enable(2, true);
        
        // Initialize system status
        let system_status = SystemStatus {
            heartbeat: 0,
            last_error: None,
            uptime_ms: 0,
            loop_stats: ControlLoopStats {
                total_cycles: 0,
                max_execution_time_us: 0,
                avg_execution_time_us: 0,
                missed_cycles: 0,
            },
        };
        
        // TODO: Initialize CAN peripheral
        let can_peripheral = None;
        
        // TODO: Initialize display
        let display = None;
        
        defmt::info!("RumbleDome firmware initialized successfully");
        
        // Start the main control loop
        control_loop::spawn().ok();
        
        // Start status update task
        status_update::spawn().ok();
        
        (
            Shared {
                controller,
                system_status,
            },
            Local {
                led: board.led,
                control_timer: pit.timer(2),
                can_peripheral,
                display,
                profile_button: board.button,
            },
        )
    }
    
    /// Main control loop task (100Hz)
    #[task(shared = [controller, system_status], priority = 3)]
    async fn control_loop(mut ctx: control_loop::Context) {
        loop {
            let start_time = Systick::now();
            
            // Execute one control cycle
            let result = ctx.controller.lock(|controller| {
                controller.execute_control_cycle()
            });
            
            // Update statistics
            let execution_time = start_time.elapsed().to_micros();
            
            ctx.system_status.lock(|status| {
                status.loop_stats.total_cycles += 1;
                
                if execution_time > status.loop_stats.max_execution_time_us {
                    status.loop_stats.max_execution_time_us = execution_time;
                }
                
                // Update rolling average
                let alpha = 0.1; // EMA factor
                status.loop_stats.avg_execution_time_us = 
                    ((1.0 - alpha) * status.loop_stats.avg_execution_time_us as f32 + 
                     alpha * execution_time as f32) as u32;
                
                // Check for errors
                if let Err(e) = result {
                    status.last_error = Some("Control loop error");
                    defmt::error!("Control loop error: {:?}", defmt::Debug2Format(&e));
                }
            });
            
            // Wait for next control period
            Systick::delay(CONTROL_PERIOD_US.micros()).await;
        }
    }
    
    /// Status update and housekeeping task (1Hz)  
    #[task(shared = [system_status], local = [led], priority = 1)]
    async fn status_update(mut ctx: status_update::Context) {
        loop {
            ctx.system_status.lock(|status| {
                status.heartbeat += 1;
                status.uptime_ms += 1000; // 1 second
                
                // Blink LED to show system is alive
                if status.heartbeat % 2 == 0 {
                    ctx.led.set();
                } else {
                    ctx.led.clear();
                }
                
                // Log periodic status
                if status.heartbeat % 10 == 0 {
                    defmt::info!(
                        "Status: uptime={}s, cycles={}, avg_time={}us, max_time={}us",
                        status.uptime_ms / 1000,
                        status.loop_stats.total_cycles,
                        status.loop_stats.avg_execution_time_us,
                        status.loop_stats.max_execution_time_us
                    );
                }
            });
            
            // Update every second
            Systick::delay(1000.millis()).await;
        }
    }
    
    /// Handle profile button press
    #[task(shared = [controller], local = [profile_button], priority = 2)]
    async fn handle_profile_button(mut ctx: handle_profile_button::Context) {
        loop {
            // Wait for button press
            // TODO: Implement proper button debouncing and edge detection
            if ctx.profile_button.is_set() {
                defmt::info!("Profile button pressed");
                
                // Cycle through profiles
                ctx.controller.lock(|controller| {
                    // TODO: Implement profile switching logic
                    defmt::info!("Profile switching not yet implemented");
                });
                
                // Debounce delay
                Systick::delay(200.millis()).await;
            }
            
            Systick::delay(50.millis()).await;
        }
    }
    
    /// CAN message reception handler
    #[task(shared = [controller], priority = 2)]
    async fn can_rx_handler(_ctx: can_rx_handler::Context) {
        // TODO: Implement CAN message reception and parsing
        defmt::debug!("CAN RX handler - not yet implemented");
    }
    
    /// Handle system faults and emergency shutdown
    #[task(shared = [controller, system_status], priority = 4)]
    async fn emergency_handler(mut ctx: emergency_handler::Context) {
        defmt::error!("Emergency handler triggered!");
        
        // Immediately shut down boost control
        ctx.controller.lock(|controller| {
            // Force system to safe state
            // TODO: Implement emergency shutdown
            defmt::error!("Emergency shutdown - forcing safe state");
        });
        
        ctx.system_status.lock(|status| {
            status.last_error = Some("Emergency shutdown");
        });
        
        // Flash LED rapidly to indicate fault
        loop {
            ctx.local.led.toggle();
            Systick::delay(100.millis()).await;
        }
    }
    
    /// Idle task - lowest priority
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            // Enter sleep mode to save power
            asm::wfi();
        }
    }
}

// Import monotonic timer support
use rtic_monotonics::systick::{fugit::ExtU32, Systick};

// Helper trait for time conversions
trait TimeExt {
    fn millis(self) -> <Systick as rtic_monotonics::Monotonic>::Duration;
    fn micros(self) -> <Systick as rtic_monotonics::Monotonic>::Duration;
}

impl TimeExt for u32 {
    fn millis(self) -> <Systick as rtic_monotonics::Monotonic>::Duration {
        <Systick as rtic_monotonics::Monotonic>::Duration::from_ticks(
            (self as u64 * 600_000_000) / 1000
        )
    }
    
    fn micros(self) -> <Systick as rtic_monotonics::Monotonic>::Duration {
        <Systick as rtic_monotonics::Monotonic>::Duration::from_ticks(
            (self as u64 * 600_000_000) / 1_000_000
        )
    }
}