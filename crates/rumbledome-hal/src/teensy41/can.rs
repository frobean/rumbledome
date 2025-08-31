//! CAN bus implementation for Teensy 4.1
//! 
//! Provides CAN bus functionality for reading Ford Gen2 Coyote ECU signals
//! using the i.MX RT1062 FlexCAN modules.

use crate::traits::CanBus;
use crate::types::{CanMessage, CanErrorStats};
use crate::error::HalError;

use teensy4_bsp::hal;
use hal::can::{self, Filter, Id, Frame};
use heapless::spsc::{Queue, Producer, Consumer};

/// CAN message queue size
const CAN_RX_QUEUE_SIZE: usize = 32;

/// Teensy 4.1 CAN bus implementation
pub struct Teensy41Can {
    /// CAN peripheral instance
    can: can::Can<can::module::_1>,
    
    /// Receive message queue
    rx_queue: Queue<CanMessage, CAN_RX_QUEUE_SIZE>,
    
    /// Queue producer (for interrupt handler)
    rx_producer: Producer<'static, CanMessage, CAN_RX_QUEUE_SIZE>,
    
    /// Queue consumer (for application)
    rx_consumer: Consumer<'static, CanMessage, CAN_RX_QUEUE_SIZE>,
    
    /// Error statistics
    error_stats: CanErrorStats,
    
    /// Connection status
    connected: bool,
    
    /// Last successful message timestamp
    last_message_ms: Option<u64>,
}

impl Teensy41Can {
    /// Create new CAN bus controller
    pub fn new() -> Result<Self, HalError> {
        
        // Initialize CAN peripheral at 500 kbps
        let can = can::Can::new(
            unsafe { can::module::_1::new() },
            can::ClockSource::Peripheral(24_000_000), // 24 MHz peripheral clock
            can::BitRate::B500K,
        ).map_err(|e| HalError::can_error(format!("CAN init failed: {:?}", e)))?;
        
        // Set up message filters for Ford Gen2 Coyote signals
        // These are speculative and need to be verified with real CAN logs
        let filters = [
            // ⚠ SPECULATIVE CAN IDs - need verification
            Filter::accept_id(Id::Standard(0x201)), // RPM
            Filter::accept_id(Id::Standard(0x202)), // MAP
            Filter::accept_id(Id::Standard(0x203)), // Desired torque
            Filter::accept_id(Id::Standard(0x204)), // Actual torque
            Filter::accept_id(Id::Standard(0x205)), // TPS
            Filter::accept_id(Id::Standard(0x206)), // Drive mode
        ];
        
        can.set_filters(&filters)
            .map_err(|e| HalError::can_error(format!("Filter setup failed: {:?}", e)))?;
        
        // Create message queue for async reception
        let queue = Queue::new();
        let (rx_producer, rx_consumer) = queue.split();
        
        log::info!("CAN bus initialized at 500 kbps with {} filters", filters.len());
        
        Ok(Self {
            can,
            rx_queue: Queue::new(), // This will be replaced by proper static allocation
            rx_producer,
            rx_consumer,
            error_stats: CanErrorStats::default(),
            connected: false,
            last_message_ms: None,
        })
    }
    
    /// Process received CAN frame and convert to CanMessage
    fn process_rx_frame(&mut self, frame: Frame) -> Option<CanMessage> {
        let id = match frame.id() {
            Id::Standard(id) => id as u32,
            Id::Extended(id) => id,
        };
        
        let data = frame.data().to_vec();
        
        Some(CanMessage {
            id,
            data,
            extended: matches!(frame.id(), Id::Extended(_)),
            rtr: frame.is_remote_transmission_request(),
        })
    }
    
    /// Update connection status based on message activity
    fn update_connection_status(&mut self, current_time_ms: u64) {
        if let Some(last_msg) = self.last_message_ms {
            // Consider connected if we received a message within last 1000ms
            self.connected = (current_time_ms - last_msg) < 1000;
        } else {
            self.connected = false;
        }
    }
    
    /// Handle CAN error conditions
    fn handle_can_error(&mut self, error: can::Error) {
        match error {
            can::Error::BusOff => {
                self.error_stats.bus_off_count += 1;
                self.connected = false;
                log::error!("CAN bus off error");
            },
            can::Error::ErrorPassive => {
                log::warn!("CAN error passive state");
            },
            can::Error::ErrorActive => {
                log::debug!("CAN error active state");
            },
            can::Error::Transmit => {
                self.error_stats.tx_errors += 1;
                log::warn!("CAN transmit error");
            },
            can::Error::Receive => {
                self.error_stats.rx_errors += 1;
                log::warn!("CAN receive error");
            },
        }
        
        self.error_stats.last_error_ms = Some(0); // TODO: Get current time
    }
}

impl CanBus for Teensy41Can {
    fn send(&mut self, message: &CanMessage) -> Result<(), HalError> {
        let id = if message.extended {
            Id::Extended(message.id)
        } else {
            Id::Standard(message.id as u16)
        };
        
        let frame = if message.rtr {
            Frame::new_remote(id, message.data.len() as u8)
        } else {
            Frame::new_data(id, &message.data)
        };
        
        self.can.transmit(&frame)
            .map_err(|e| HalError::can_error(format!("CAN transmit failed: {:?}", e)))?;
        
        log::trace!("CAN message sent: ID={:03X}, len={}", message.id, message.data.len());
        
        Ok(())
    }
    
    fn receive(&mut self) -> Result<Option<CanMessage>, HalError> {
        // Try to receive from hardware
        match self.can.receive() {
            Ok(frame) => {
                if let Some(message) = self.process_rx_frame(frame) {
                    self.last_message_ms = Some(0); // TODO: Get current timestamp
                    log::trace!("CAN message received: ID={:03X}, len={}", 
                        message.id, message.data.len());
                    return Ok(Some(message));
                }
            },
            Err(can::Error::WouldBlock) => {
                // No message available, check queue
            },
            Err(e) => {
                self.handle_can_error(e);
                return Err(HalError::can_error(format!("CAN receive error: {:?}", e)));
            }
        }
        
        // Check message queue
        if let Some(message) = self.rx_consumer.dequeue() {
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
    
    fn get_error_stats(&self) -> CanErrorStats {
        self.error_stats.clone()
    }
    
    fn reset(&mut self) -> Result<(), HalError> {
        // Reset CAN peripheral
        self.can.reset()
            .map_err(|e| HalError::can_error(format!("CAN reset failed: {:?}", e)))?;
        
        // Clear error statistics
        self.error_stats = CanErrorStats::default();
        self.connected = false;
        
        log::info!("CAN bus reset");
        
        Ok(())
    }
}

/// CAN message parser for Ford Gen2 Coyote signals
pub struct CoyoteCanParser {
    /// Last parsed engine data
    pub last_data: CoyoteEngineData,
    
    /// Data validity flags
    pub data_valid: CoyoteDataValidity,
}

/// Parsed engine data from CAN
#[derive(Debug, Clone, Default)]
pub struct CoyoteEngineData {
    /// Engine RPM
    pub rpm: u16,
    
    /// Manifold absolute pressure (kPa)
    pub map_kpa: f32,
    
    /// ECU desired torque (Nm)
    pub desired_torque: f32,
    
    /// ECU actual torque (Nm)
    pub actual_torque: f32,
    
    /// Throttle position (%)
    pub throttle_position: Option<f32>,
    
    /// Drive mode
    pub drive_mode: Option<crate::types::DriveMode>,
    
    /// Last update timestamp
    pub timestamp_ms: u64,
}

/// Data validity tracking
#[derive(Debug, Clone, Default)]
pub struct CoyoteDataValidity {
    pub rpm_valid: bool,
    pub map_valid: bool,
    pub desired_torque_valid: bool,
    pub actual_torque_valid: bool,
    pub throttle_valid: bool,
    pub drive_mode_valid: bool,
}

impl CoyoteCanParser {
    /// Create new CAN parser
    pub fn new() -> Self {
        Self {
            last_data: CoyoteEngineData::default(),
            data_valid: CoyoteDataValidity::default(),
        }
    }
    
    /// Parse CAN message and update engine data
    pub fn parse_message(&mut self, message: &CanMessage, timestamp_ms: u64) -> bool {
        let mut updated = false;
        
        match message.id {
            // ⚠ SPECULATIVE - These CAN IDs need verification
            0x201 => {
                if message.data.len() >= 2 {
                    self.last_data.rpm = u16::from_le_bytes([message.data[0], message.data[1]]);
                    self.data_valid.rpm_valid = true;
                    updated = true;
                }
            },
            
            0x202 => {
                if message.data.len() >= 2 {
                    let raw = u16::from_le_bytes([message.data[0], message.data[1]]);
                    self.last_data.map_kpa = (raw as f32) * 0.1; // Scale factor TBD
                    self.data_valid.map_valid = true;
                    updated = true;
                }
            },
            
            0x203 => {
                if message.data.len() >= 2 {
                    let raw = u16::from_le_bytes([message.data[0], message.data[1]]);
                    self.last_data.desired_torque = (raw as f32) * 0.1 - 1000.0; // Scale/offset TBD
                    self.data_valid.desired_torque_valid = true;
                    updated = true;
                }
            },
            
            0x204 => {
                if message.data.len() >= 2 {
                    let raw = u16::from_le_bytes([message.data[0], message.data[1]]);
                    self.last_data.actual_torque = (raw as f32) * 0.1 - 1000.0; // Scale/offset TBD
                    self.data_valid.actual_torque_valid = true;
                    updated = true;
                }
            },
            
            0x205 => {
                if !message.data.is_empty() {
                    self.last_data.throttle_position = Some((message.data[0] as f32) / 2.55); // 0-255 -> 0-100%
                    self.data_valid.throttle_valid = true;
                    updated = true;
                }
            },
            
            0x206 => {
                if !message.data.is_empty() {
                    self.last_data.drive_mode = match message.data[0] {
                        0 => Some(crate::types::DriveMode::Normal),
                        1 => Some(crate::types::DriveMode::Sport),
                        2 => Some(crate::types::DriveMode::SportPlus),
                        3 => Some(crate::types::DriveMode::Track),
                        _ => None,
                    };
                    self.data_valid.drive_mode_valid = true;
                    updated = true;
                }
            },
            
            _ => {
                // Unknown message ID
                log::trace!("Unknown CAN ID: {:03X}", message.id);
            }
        }
        
        if updated {
            self.last_data.timestamp_ms = timestamp_ms;
        }
        
        updated
    }
    
    /// Check if all critical data is valid
    pub fn has_valid_data(&self) -> bool {
        self.data_valid.rpm_valid &&
        self.data_valid.map_valid &&
        self.data_valid.desired_torque_valid &&
        self.data_valid.actual_torque_valid
    }
    
    /// Get data age in milliseconds
    pub fn data_age_ms(&self, current_time_ms: u64) -> u64 {
        current_time_ms.saturating_sub(self.last_data.timestamp_ms)
    }
}

impl Default for CoyoteCanParser {
    fn default() -> Self {
        Self::new()
    }
}