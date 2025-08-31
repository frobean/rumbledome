//! Time provider implementation for Teensy 4.1

use crate::traits::TimeProvider;
use cortex_m::peripheral::{DWT, SYST};

/// Teensy 4.1 time provider using DWT cycle counter
pub struct Teensy41Time {
    /// System clock frequency (600 MHz for Teensy 4.1)
    clock_freq: u32,
    
    /// Startup timestamp for relative time calculation
    startup_cycles: u32,
}

impl Teensy41Time {
    /// Create new time provider
    pub fn new() -> Self {
        // Enable DWT cycle counter for precise timing
        unsafe {
            let dwt = &*DWT::PTR;
            dwt.ctrl.modify(|r| r | 1);
        }
        
        let startup_cycles = DWT::cycle_count();
        
        Self {
            clock_freq: 600_000_000, // 600 MHz
            startup_cycles,
        }
    }
    
    /// Get current cycle count
    fn current_cycles(&self) -> u32 {
        DWT::cycle_count()
    }
    
    /// Convert cycles to milliseconds
    fn cycles_to_ms(&self, cycles: u32) -> u64 {
        ((cycles as u64) * 1000) / (self.clock_freq as u64)
    }
    
    /// Convert cycles to microseconds
    fn cycles_to_us(&self, cycles: u32) -> u64 {
        ((cycles as u64) * 1_000_000) / (self.clock_freq as u64)
    }
}

impl TimeProvider for Teensy41Time {
    fn now_ms(&self) -> u64 {
        let current = self.current_cycles();
        let elapsed_cycles = current.wrapping_sub(self.startup_cycles);
        self.cycles_to_ms(elapsed_cycles)
    }
    
    fn delay_ms(&mut self, ms: u32) {
        let start_cycles = self.current_cycles();
        let delay_cycles = (ms as u64 * self.clock_freq as u64) / 1000;
        
        while self.current_cycles().wrapping_sub(start_cycles) < delay_cycles as u32 {
            cortex_m::asm::nop();
        }
    }
    
    fn timestamp_us(&self) -> u64 {
        let current = self.current_cycles();
        let elapsed_cycles = current.wrapping_sub(self.startup_cycles);
        self.cycles_to_us(elapsed_cycles)
    }
}