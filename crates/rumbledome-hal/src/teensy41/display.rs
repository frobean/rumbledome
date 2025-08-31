//! Display implementation for Teensy 4.1
//! 
//! Provides ST7735R TFT display control via SPI interface
//! for real-time gauge display and system status.

use crate::traits::DisplayController;
use crate::types::{DisplayMode, GaugeConfig};
use crate::error::HalError;

use teensy4_bsp::hal;
use hal::spi::Spi;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle, Sector},
    text::Text,
};
use st7735_lcd::{ST7735, Orientation};

/// Display dimensions
const DISPLAY_WIDTH: u32 = 128;
const DISPLAY_HEIGHT: u32 = 160;

/// Color definitions
const COLOR_BLACK: Rgb565 = Rgb565::BLACK;
const COLOR_WHITE: Rgb565 = Rgb565::WHITE;
const COLOR_RED: Rgb565 = Rgb565::RED;
const COLOR_GREEN: Rgb565 = Rgb565::GREEN;
const COLOR_BLUE: Rgb565 = Rgb565::BLUE;
const COLOR_YELLOW: Rgb565 = Rgb565::YELLOW;
const COLOR_ORANGE: Rgb565 = Rgb565::new(31, 32, 0); // Orange approximation
const COLOR_GRAY: Rgb565 = Rgb565::new(15, 31, 15);

/// Teensy 4.1 display implementation
pub struct Teensy41Display<SPI> {
    /// ST7735R display driver
    display: ST7735<SPI, hal::gpio::Output, hal::gpio::Output>,
    
    /// Current display mode
    mode: DisplayMode,
    
    /// Display update counter
    frame_counter: u32,
    
    /// Gauge configurations
    boost_gauge: GaugeConfig,
    target_gauge: GaugeConfig,
    
    /// Status message buffer
    status_message: [u8; 64],
    status_timeout: u32,
}

/// Gauge rendering configuration
#[derive(Clone)]
struct GaugeLayout {
    center_x: i32,
    center_y: i32,
    radius: u32,
    start_angle: f32,
    end_angle: f32,
    min_value: f32,
    max_value: f32,
    label: &'static str,
    unit: &'static str,
}

impl<SPI> Teensy41Display<SPI>
where
    SPI: hal::spi::SpiExt,
{
    /// Create new display controller
    pub fn new(
        spi: SPI,
        dc_pin: hal::gpio::Output,
        rst_pin: hal::gpio::Output,
    ) -> Result<Self, HalError> {
        
        // Initialize ST7735 display driver
        let mut display = ST7735::new(spi, dc_pin, rst_pin, true, false, 128, 160);
        
        // Initialize display hardware
        display.init(&mut hal::delay::Delay::new())
            .map_err(|e| HalError::display_error(format!("Display init failed: {:?}", e)))?;
        
        // Set orientation for landscape viewing
        display.set_orientation(&Orientation::Portrait)
            .map_err(|e| HalError::display_error(format!("Orientation failed: {:?}", e)))?;
        
        // Clear display to black
        display.clear(COLOR_BLACK)
            .map_err(|e| HalError::display_error(format!("Clear failed: {:?}", e)))?;
        
        // Configure default gauges
        let boost_gauge = GaugeConfig {
            min_value: -10.0,
            max_value: 30.0,
            warning_threshold: 25.0,
            danger_threshold: 28.0,
            label: "BOOST",
        };
        
        let target_gauge = GaugeConfig {
            min_value: -10.0,
            max_value: 30.0,
            warning_threshold: 25.0,
            danger_threshold: 28.0,
            label: "TARGET",
        };
        
        log::info!("ST7735 display initialized ({}x{})", DISPLAY_WIDTH, DISPLAY_HEIGHT);
        
        Ok(Self {
            display,
            mode: DisplayMode::Gauges,
            frame_counter: 0,
            boost_gauge,
            target_gauge,
            status_message: [0u8; 64],
            status_timeout: 0,
        })
    }
    
    /// Render boost pressure gauge
    fn render_boost_gauge(&mut self, current_psi: f32, target_psi: f32) -> Result<(), HalError> {
        let layout = GaugeLayout {
            center_x: 64,
            center_y: 50,
            radius: 35,
            start_angle: -150.0,
            end_angle: 30.0,
            min_value: self.boost_gauge.min_value,
            max_value: self.boost_gauge.max_value,
            label: "BOOST",
            unit: "PSI",
        };
        
        self.draw_gauge(&layout, current_psi, Some(target_psi))?;
        
        // Draw numerical readout
        let text_style = MonoTextStyle::new(&FONT_8X13, COLOR_WHITE);
        let boost_text = format!("{:.1}", current_psi);
        Text::new(&boost_text, Point::new(45, 90), text_style)
            .draw(&mut self.display)
            .map_err(|e| HalError::display_error(format!("Text draw failed: {:?}", e)))?;
        
        Ok(())
    }
    
    /// Render generic gauge with optional target indicator
    fn draw_gauge(&mut self, layout: &GaugeLayout, value: f32, target: Option<f32>) -> Result<(), HalError> {
        let center = Point::new(layout.center_x, layout.center_y);
        
        // Draw gauge background circle
        Circle::new(center - Point::new(layout.radius as i32, layout.radius as i32), layout.radius * 2)
            .into_styled(PrimitiveStyle::with_stroke(COLOR_GRAY, 2))
            .draw(&mut self.display)
            .map_err(|e| HalError::display_error(format!("Circle draw failed: {:?}", e)))?;
        
        // Draw scale marks
        self.draw_gauge_scale(layout)?;
        
        // Draw current value needle
        let value_angle = self.value_to_angle(layout, value);
        self.draw_needle(layout, value_angle, COLOR_GREEN, 3)?;
        
        // Draw target indicator if provided
        if let Some(target_val) = target {
            let target_angle = self.value_to_angle(layout, target_val);
            self.draw_needle(layout, target_angle, COLOR_YELLOW, 1)?;
        }
        
        // Draw label
        let text_style = MonoTextStyle::new(&FONT_6X10, COLOR_WHITE);
        let label_pos = Point::new(
            layout.center_x - (layout.label.len() as i32 * 3), 
            layout.center_y + 20
        );
        Text::new(layout.label, label_pos, text_style)
            .draw(&mut self.display)
            .map_err(|e| HalError::display_error(format!("Label draw failed: {:?}", e)))?;
        
        Ok(())
    }
    
    /// Draw gauge scale marks and numbers
    fn draw_gauge_scale(&mut self, layout: &GaugeLayout) -> Result<(), HalError> {
        let num_marks = 8;
        let angle_range = layout.end_angle - layout.start_angle;
        let value_range = layout.max_value - layout.min_value;
        
        for i in 0..=num_marks {
            let angle = layout.start_angle + (angle_range * i as f32 / num_marks as f32);
            let value = layout.min_value + (value_range * i as f32 / num_marks as f32);
            
            // Calculate mark positions
            let angle_rad = angle.to_radians();
            let inner_radius = (layout.radius - 8) as f32;
            let outer_radius = (layout.radius - 3) as f32;
            
            let inner_x = layout.center_x + (inner_radius * angle_rad.cos()) as i32;
            let inner_y = layout.center_y + (inner_radius * angle_rad.sin()) as i32;
            let outer_x = layout.center_x + (outer_radius * angle_rad.cos()) as i32;
            let outer_y = layout.center_y + (outer_radius * angle_rad.sin()) as i32;
            
            // Draw scale mark
            Line::new(Point::new(inner_x, inner_y), Point::new(outer_x, outer_y))
                .into_styled(PrimitiveStyle::with_stroke(COLOR_WHITE, 1))
                .draw(&mut self.display)
                .map_err(|e| HalError::display_error(format!("Scale mark failed: {:?}", e)))?;
            
            // Draw numbers for major marks
            if i % 2 == 0 {
                let text_style = MonoTextStyle::new(&FONT_6X10, COLOR_WHITE);
                let text_radius = (layout.radius - 15) as f32;
                let text_x = layout.center_x + (text_radius * angle_rad.cos()) as i32 - 6;
                let text_y = layout.center_y + (text_radius * angle_rad.sin()) as i32 + 3;
                
                let value_text = format!("{:.0}", value);
                Text::new(&value_text, Point::new(text_x, text_y), text_style)
                    .draw(&mut self.display)
                    .map_err(|e| HalError::display_error(format!("Scale text failed: {:?}", e)))?;
            }
        }
        
        Ok(())
    }
    
    /// Draw gauge needle
    fn draw_needle(&mut self, layout: &GaugeLayout, angle: f32, color: Rgb565, thickness: u32) -> Result<(), HalError> {
        let angle_rad = angle.to_radians();
        let needle_length = (layout.radius - 10) as f32;
        
        let end_x = layout.center_x + (needle_length * angle_rad.cos()) as i32;
        let end_y = layout.center_y + (needle_length * angle_rad.sin()) as i32;
        
        Line::new(Point::new(layout.center_x, layout.center_y), Point::new(end_x, end_y))
            .into_styled(PrimitiveStyle::with_stroke(color, thickness))
            .draw(&mut self.display)
            .map_err(|e| HalError::display_error(format!("Needle draw failed: {:?}", e)))?;
        
        // Draw center dot
        Circle::new(Point::new(layout.center_x - 2, layout.center_y - 2), 4)
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(&mut self.display)
            .map_err(|e| HalError::display_error(format!("Center dot failed: {:?}", e)))?;
        
        Ok(())
    }
    
    /// Convert value to angle for gauge display
    fn value_to_angle(&self, layout: &GaugeLayout, value: f32) -> f32 {
        let normalized = (value - layout.min_value) / (layout.max_value - layout.min_value);
        let clamped = normalized.clamp(0.0, 1.0);
        layout.start_angle + clamped * (layout.end_angle - layout.start_angle)
    }
    
    /// Render system status display
    fn render_status_display(&mut self, rpm: u16, map_kpa: f32, throttle: f32) -> Result<(), HalError> {
        // Clear lower portion
        Rectangle::new(Point::new(0, 100), Size::new(DISPLAY_WIDTH, 60))
            .into_styled(PrimitiveStyle::with_fill(COLOR_BLACK))
            .draw(&mut self.display)
            .map_err(|e| HalError::display_error(format!("Clear failed: {:?}", e)))?;
        
        let text_style = MonoTextStyle::new(&FONT_6X10, COLOR_WHITE);
        
        // RPM display
        let rpm_text = format!("RPM: {}", rpm);
        Text::new(&rpm_text, Point::new(5, 115), text_style)
            .draw(&mut self.display)
            .map_err(|e| HalError::display_error(format!("RPM text failed: {:?}", e)))?;
        
        // MAP display
        let map_text = format!("MAP: {:.1}kPa", map_kpa);
        Text::new(&map_text, Point::new(5, 130), text_style)
            .draw(&mut self.display)
            .map_err(|e| HalError::display_error(format!("MAP text failed: {:?}", e)))?;
        
        // Throttle display
        let throttle_text = format!("TPS: {:.0}%", throttle);
        Text::new(&throttle_text, Point::new(5, 145), text_style)
            .draw(&mut self.display)
            .map_err(|e| HalError::display_error(format!("Throttle text failed: {:?}", e)))?;
        
        Ok(())
    }
    
    /// Show temporary status message
    pub fn show_status_message(&mut self, message: &str, timeout_ms: u32) {
        // Copy message to buffer (truncate if too long)
        let msg_bytes = message.as_bytes();
        let copy_len = core::cmp::min(msg_bytes.len(), self.status_message.len() - 1);
        self.status_message[..copy_len].copy_from_slice(&msg_bytes[..copy_len]);
        self.status_message[copy_len] = 0; // Null terminate
        
        self.status_timeout = timeout_ms;
        
        log::debug!("Status message: {} ({}ms)", message, timeout_ms);
    }
    
    /// Update status message timeout
    pub fn update_status_timeout(&mut self, elapsed_ms: u32) {
        if self.status_timeout > 0 {
            self.status_timeout = self.status_timeout.saturating_sub(elapsed_ms);
        }
    }
    
    /// Check if status message is active
    pub fn has_active_status(&self) -> bool {
        self.status_timeout > 0
    }
}

impl<SPI> DisplayController for Teensy41Display<SPI>
where
    SPI: hal::spi::SpiExt,
{
    fn clear(&mut self) -> Result<(), HalError> {
        self.display.clear(COLOR_BLACK)
            .map_err(|e| HalError::display_error(format!("Clear failed: {:?}", e)))?;
        
        self.frame_counter = 0;
        log::trace!("Display cleared");
        
        Ok(())
    }
    
    fn set_display_mode(&mut self, mode: DisplayMode) -> Result<(), HalError> {
        if mode != self.mode {
            self.mode = mode;
            self.clear()?; // Clear when changing modes
            
            log::debug!("Display mode changed to: {:?}", mode);
        }
        
        Ok(())
    }
    
    fn update_gauges(&mut self, boost_psi: f32, target_psi: f32, _duty_cycle: f32) -> Result<(), HalError> {
        match self.mode {
            DisplayMode::Gauges => {
                // Clear gauge area
                Rectangle::new(Point::new(0, 0), Size::new(DISPLAY_WIDTH, 100))
                    .into_styled(PrimitiveStyle::with_fill(COLOR_BLACK))
                    .draw(&mut self.display)
                    .map_err(|e| HalError::display_error(format!("Clear failed: {:?}", e)))?;
                
                // Render boost gauge
                self.render_boost_gauge(boost_psi, target_psi)?;
            },
            DisplayMode::Status => {
                // Status mode doesn't show gauges
            }
        }
        
        self.frame_counter += 1;
        
        Ok(())
    }
    
    fn show_status(&mut self, rpm: u16, map_kpa: f32, throttle: f32) -> Result<(), HalError> {
        self.render_status_display(rpm, map_kpa, throttle)?;
        
        // Show active status message if present
        if self.has_active_status() {
            let text_style = MonoTextStyle::new(&FONT_6X10, COLOR_YELLOW);
            
            // Convert status message buffer to string
            let msg_end = self.status_message.iter().position(|&x| x == 0).unwrap_or(self.status_message.len());
            if let Ok(message) = core::str::from_utf8(&self.status_message[..msg_end]) {
                Text::new(message, Point::new(5, 157), text_style)
                    .draw(&mut self.display)
                    .map_err(|e| HalError::display_error(format!("Status message failed: {:?}", e)))?;
            }
        }
        
        Ok(())
    }
    
    fn show_error(&mut self, error_message: &str) -> Result<(), HalError> {
        // Clear display
        self.clear()?;
        
        // Draw error border
        Rectangle::new(Point::new(2, 2), Size::new(DISPLAY_WIDTH - 4, DISPLAY_HEIGHT - 4))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_RED, 2))
            .draw(&mut self.display)
            .map_err(|e| HalError::display_error(format!("Error border failed: {:?}", e)))?;
        
        // Draw "ERROR" title
        let title_style = MonoTextStyle::new(&FONT_8X13, COLOR_RED);
        Text::new("ERROR", Point::new(40, 25), title_style)
            .draw(&mut self.display)
            .map_err(|e| HalError::display_error(format!("Error title failed: {:?}", e)))?;
        
        // Draw error message (wrapped if needed)
        let msg_style = MonoTextStyle::new(&FONT_6X10, COLOR_WHITE);
        let max_chars_per_line = 20;
        let mut y_pos = 50;
        
        for chunk in error_message.chars().collect::<Vec<_>>().chunks(max_chars_per_line) {
            let line: String = chunk.iter().collect();
            Text::new(&line, Point::new(10, y_pos), msg_style)
                .draw(&mut self.display)
                .map_err(|e| HalError::display_error(format!("Error message failed: {:?}", e)))?;
            
            y_pos += 12;
            if y_pos > 140 { break; } // Don't overflow display
        }
        
        log::error!("Error displayed: {}", error_message);
        
        Ok(())
    }
    
    fn get_display_mode(&self) -> DisplayMode {
        self.mode
    }
}