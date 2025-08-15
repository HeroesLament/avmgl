//! OTM8009A Display Driver Module
//! 
//! This module provides support for the OTM8009A display controller
//! used in the STM32F769I-DISCO board.

pub mod defs;
pub mod driver;
pub mod nifs;

// Re-export the main types and functions
pub use driver::OTM8009ADriver;
pub use defs::*;

#[cfg(feature = "nifs")]
pub use nifs::*;