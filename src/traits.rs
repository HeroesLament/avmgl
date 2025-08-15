//! Core traits for the OTM8009A display driver
//! 
//! These traits define the hardware abstraction layer for the display driver.

extern crate alloc;

use alloc::vec::Vec;

/// DSI (Display Serial Interface) trait for sending commands to the display
pub trait DsiInterface {
    type Error;
    
    /// Send a DCS (Display Command Set) command to the display
    fn send_dcs_command(&mut self, nb_params: usize, params: &[u8]) -> Result<(), Self::Error>;
    
    /// Delay for the specified number of milliseconds
    fn delay_ms(&mut self, ms: u32);
    
    /// Check if the DSI interface is ready
    fn is_ready(&self) -> bool;
    
    /// Reset the DSI interface
    fn reset(&mut self) -> Result<(), Self::Error>;
}

/// LTDC (LCD-TFT Display Controller) trait for managing display layers
pub trait LtdcInterface {
    type Error;
    
    /// Configure a display layer
    fn configure_layer(&mut self, layer: u8, config: LayerConfig) -> Result<(), Self::Error>;
    
    /// Enable the LTDC controller
    fn enable(&mut self) -> Result<(), Self::Error>;
    
    /// Disable the LTDC controller
    fn disable(&mut self) -> Result<(), Self::Error>;
    
    /// Set the framebuffer address for a specific layer
    fn set_framebuffer(&mut self, layer: u8, address: u32) -> Result<(), Self::Error>;
    
    /// Get display dimensions
    fn get_dimensions(&self) -> (u16, u16);
}

/// Framebuffer trait for pixel manipulation
pub trait FramebufferInterface {
    /// Fill a rectangular region with a color
    fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: u16);
    
    /// Set a single pixel
    fn set_pixel(&mut self, x: u16, y: u16, color: u16);
    
    /// Clear the entire framebuffer with a color
    fn clear(&mut self, color: u16);
    
    /// Get framebuffer dimensions
    fn get_dimensions(&self) -> (u16, u16);
    
    /// Get pointer to framebuffer data
    fn get_buffer_ptr(&self) -> *const u16;
    
    /// Get framebuffer size in bytes
    fn get_buffer_size(&self) -> usize;
}

/// Platform-specific interface trait
pub trait PlatformInterface {
    type Error;
    
    /// Initialize platform-specific hardware
    fn init_platform(&mut self) -> Result<(), Self::Error>;
    
    /// Get platform information
    fn get_platform_info(&self) -> &'static str;
    
    /// Enter low power mode
    fn enter_low_power(&mut self) -> Result<(), Self::Error>;
    
    /// Exit low power mode
    fn exit_low_power(&mut self) -> Result<(), Self::Error>;
}

/// Layer configuration for LTDC
#[derive(Debug, Clone)]
pub struct LayerConfig {
    pub layer: u8,
    pub window_x0: u16,
    pub window_x1: u16,
    pub window_y0: u16,
    pub window_y1: u16,
    pub pixel_format: PixelFormat,
    pub alpha: u8,
    pub red_blue_swap: bool,
    pub framebuffer_address: u32,
    pub framebuffer_pitch: u16,
}

/// Supported pixel formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Argb8888,
    Rgb888,
    Rgb565,
    Argb1555,
    Argb4444,
    L8,
    Al44,
    Al88,
}

/// Color conversion utilities
pub mod color {
    /// Convert RGB888 to RGB565
    pub fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
        let r5 = (r >> 3) as u16;
        let g6 = (g >> 2) as u16;
        let b5 = (b >> 3) as u16;
        
        (r5 << 11) | (g6 << 5) | b5
    }
    
    /// Convert RGB565 to RGB888
    pub fn rgb565_to_rgb888(color: u16) -> (u8, u8, u8) {
        let r = ((color >> 11) & 0x1F) as u8;
        let g = ((color >> 5) & 0x3F) as u8;
        let b = (color & 0x1F) as u8;
        
        // Scale to 8-bit
        let r8 = (r << 3) | (r >> 2);
        let g8 = (g << 2) | (g >> 4);
        let b8 = (b << 3) | (b >> 2);
        
        (r8, g8, b8)
    }
    
    /// Common colors in RGB565 format
    pub const BLACK: u16 = 0x0000;
    pub const WHITE: u16 = 0xFFFF;
    pub const RED: u16 = 0xF800;
    pub const GREEN: u16 = 0x07E0;
    pub const BLUE: u16 = 0x001F;
    pub const YELLOW: u16 = 0xFFE0;
    pub const CYAN: u16 = 0x07FF;
    pub const MAGENTA: u16 = 0xF81F;
}