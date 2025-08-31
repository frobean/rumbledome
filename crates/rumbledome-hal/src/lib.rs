//! Hardware Abstraction Layer for RumbleDome
//! 
//! Provides platform-independent traits for all hardware interfaces.
//! Supports multiple platforms through feature-gated implementations:
//! - `mock`: Mock implementations for testing (default)
//! - `teensy41`: Teensy 4.1 hardware implementation

pub mod traits;
pub mod types;
pub mod error;

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "teensy41")]
pub mod teensy41;

pub use traits::*;
pub use types::*;
pub use error::*;

#[cfg(feature = "mock")]
pub use mock::MockHal;

#[cfg(feature = "teensy41")]
pub use teensy41::Teensy41Hal;