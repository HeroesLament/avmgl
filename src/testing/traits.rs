//! Testing-specific trait extensions and utilities
//! 
//! These extend the production traits with testing-specific functionality.

extern crate alloc;

use alloc::vec::Vec; 

use crate::traits::*;

/// Testing extensions for DSI interface
pub trait DsiTestingExt: DsiInterface {
    /// Get command history for verification
    fn get_command_history(&self) -> &[DcsCommand];
    
    /// Clear command history
    fn clear_history(&mut self);
    
    /// Set whether commands should fail
    fn set_should_fail(&mut self, fail: bool);
}

/// Testing extensions for LTDC interface  
pub trait LtdcTestingExt: LtdcInterface {
    /// Get current layer configurations
    fn get_layer_configs(&self) -> &[LayerConfig];
    
    /// Check if LTDC is enabled
    fn is_enabled(&self) -> bool;
}

/// Testing extensions for framebuffer interface
pub trait FramebufferTestingExt: FramebufferInterface {
    /// Get a pixel value for verification
    fn get_pixel(&self, x: u16, y: u16) -> Option<u16>;
    
    /// Verify framebuffer contents
    fn verify_region(&self, x: u16, y: u16, width: u16, height: u16, expected_color: u16) -> bool;
}

#[derive(Debug, Clone)]
pub struct DcsCommand {
    pub nb_params: usize,
    pub params: Vec<u8>,
}

impl FramebufferTestingExt for crate::testing::mocks::MockFramebuffer {
    fn get_pixel(&self, x: u16, y: u16) -> Option<u16> {
        self.get_pixel(x, y)
    }
    
    fn verify_region(&self, x: u16, y: u16, width: u16, height: u16, expected_color: u16) -> bool {
        for row in y..(y + height) {
            for col in x..(x + width) {
                if let Some(pixel) = self.get_pixel(col, row) {
                    if pixel != expected_color {
                        return false;
                    }
                } else {
                    return false; // Out of bounds
                }
            }
        }
        true
    }
}