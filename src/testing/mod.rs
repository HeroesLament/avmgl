//! Testing utilities and mock implementations
//! 
//! This module provides mock implementations of hardware interfaces
//! for testing the OTM8009A display driver without actual hardware.

pub mod footprint;
pub mod mocks;
pub mod nifs;
pub mod traits;

// Re-exports for easy testing
pub use footprint::*;
pub use mocks::*;
pub use nifs::*;
pub use traits::*;
