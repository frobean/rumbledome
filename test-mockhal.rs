#!/usr/bin/env rust-script
//! Simple test to verify MockHal functionality

use rumbledome_hal::{MockHal, HalTrait, TimeProvider, PwmControl};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing MockHal implementation...");
    
    // Create MockHal instance
    let mut hal = MockHal::new();
    println!("✅ MockHal created successfully");
    
    // Test initialization
    hal.init()?;
    println!("✅ HAL initialized");
    
    // Test self-test
    let test_result = hal.self_test()?;
    println!("✅ Self-test completed: {:?}", test_result.overall_status);
    
    // Test time functions
    let time_us = hal.now_us();
    let time_ms = hal.now_ms();
    println!("✅ Time functions: {}μs, {}ms", time_us, time_ms);
    
    // Test PWM control
    hal.set_duty_cycle(25.0)?;
    println!("✅ PWM duty cycle set to 25%");
    
    let current_duty = hal.get_current_duty();
    println!("✅ Current duty cycle: {:.1}%", current_duty);
    
    // Test emergency shutdown
    hal.emergency_shutdown()?;
    println!("✅ Emergency shutdown executed");
    
    println!("\n🎉 All MockHal tests passed!");
    println!("🔗 MockHal is ready for RumbleDome core development");
    
    Ok(())
}