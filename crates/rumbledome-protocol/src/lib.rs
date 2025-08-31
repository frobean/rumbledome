//! RumbleDome JSON/CLI Protocol Definitions
//! 
//! Defines the communication protocol between RumbleDome and external
//! configuration tools, desktop simulator, and diagnostic interfaces.

pub mod messages;
pub mod error;

pub use messages::*;
pub use error::*;