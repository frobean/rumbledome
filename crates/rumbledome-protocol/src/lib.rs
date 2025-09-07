//! RumbleDome JSON/CLI Protocol Definitions
//! 
//! 🔗 T4-PROTOCOL-001: Protocol Message Definitions
//! Derived From: T3-BUILD-004 (JSON Protocol Specification)
//! Decision Type: 🔗 Direct Derivation - Implementation of protocol specification
//! AI Traceability: Enables configuration management, diagnostic communication, CLI interaction

#![no_std]
#![cfg_attr(not(feature = "std"), no_main)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use rumbledome_core::{SystemConfig, SystemState, SystemStatus, CoreError};
use serde::{Deserialize, Serialize};

// Re-export core types for protocol use
pub use rumbledome_core::*;

/// Protocol message types for RumbleDome communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolMessage {
    /// Request current system status
    GetStatus,
    /// System status response
    Status(SystemStatus),
    /// Update system configuration
    SetConfig(SystemConfig),
    /// Configuration update response
    ConfigUpdated,
    /// Error response
    Error(String),
}

// TODO: Implement full protocol message definitions
// This is a placeholder for the protocol system