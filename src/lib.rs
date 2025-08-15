//! OTM8009A Display Driver for AtomVM
//! 
//! This crate provides OTM8009A display controller support for AtomVM,
//! implemented in Rust using avmnif-rs.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

// Enable alloc for collections when not using std
#[cfg(not(test))]
extern crate alloc;

// Module declarations
pub mod otm8009a;
pub mod common;
pub mod traits;

#[cfg(test)]
pub mod testing;

// Re-exports
pub use otm8009a::*;
pub use common::*;
pub use traits::*;