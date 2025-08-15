//! Mock implementations for testing
//! 
//! These mocks allow testing the OTM8009A driver logic without
//! requiring actual hardware.

extern crate alloc;

use alloc::{vec, vec::Vec};

use crate::traits::*;
use crate::testing::traits::*;
use crate::otm8009a::defs::{LCD_WIDTH, LCD_HEIGHT};

/// Mock DSI interface for testing
#[derive(Debug)]
pub struct MockDsiInterface {
    pub commands_sent: Vec<DcsCommand>,
    pub delays_requested: Vec<u32>,
    pub should_fail: bool,
    pub is_ready: bool,
}

#[derive(Debug, Clone)]
pub struct DcsCommand {
    pub nb_params: usize,
    pub params: Vec<u8>,
}

#[derive(Debug)]
pub enum MockDsiError {
    NotReady,
    SimulatedFailure,
    CommandTooLong,
}

impl MockDsiInterface {
    pub fn new() -> Self {
        Self {
            commands_sent: Vec::new(),
            delays_requested: Vec::new(),
            should_fail: false,
            is_ready: true,
        }
    }
    
    pub fn set_should_fail(&mut self, fail: bool) {
        self.should_fail = fail;
    }
    
    pub fn set_ready(&mut self, ready: bool) {
        self.is_ready = ready;
    }
    
    pub fn clear_history(&mut self) {
        self.commands_sent.clear();
        self.delays_requested.clear();
    }
    
    pub fn get_last_command(&self) -> Option<&DcsCommand> {
        self.commands_sent.last()
    }
    
    pub fn command_count(&self) -> usize {
        self.commands_sent.len()
    }
}

impl DsiInterface for MockDsiInterface {
    type Error = MockDsiError;
    
    fn send_dcs_command(&mut self, nb_params: usize, params: &[u8]) -> Result<(), Self::Error> {
        if self.should_fail {
            return Err(MockDsiError::SimulatedFailure);
        }
        
        if !self.is_ready {
            return Err(MockDsiError::NotReady);
        }
        
        let param_vec = params.to_vec();
        
        let command = DcsCommand {
            nb_params,
            params: param_vec,
        };
        
        self.commands_sent.push(command);
        Ok(())
    }
    
    fn delay_ms(&mut self, ms: u32) {
        self.delays_requested.push(ms);
    }
    
    fn is_ready(&self) -> bool {
        self.is_ready
    }
    
    fn reset(&mut self) -> Result<(), Self::Error> {
        if self.should_fail {
            return Err(MockDsiError::SimulatedFailure);
        }
        
        self.clear_history();
        self.is_ready = true;
        Ok(())
    }
}

/// Mock LTDC interface for testing
#[derive(Debug)]
pub struct MockLtdcInterface {
    pub layer_configs: Vec<LayerConfig>,
    pub enabled: bool,
    pub framebuffer_addresses: Vec<(u8, u32)>,
    pub should_fail: bool,
    pub dimensions: (u16, u16),
}

#[derive(Debug)]
pub enum MockLtdcError {
    InvalidLayer,
    SimulatedFailure,
    InvalidConfiguration,
}

impl MockLtdcInterface {
    pub fn new() -> Self {
        Self {
            layer_configs: Vec::new(),
            enabled: false,
            framebuffer_addresses: Vec::new(),
            should_fail: false,
            dimensions: (LCD_WIDTH, LCD_HEIGHT),
        }
    }
    
    pub fn set_should_fail(&mut self, fail: bool) {
        self.should_fail = fail;
    }
    
    pub fn get_layer_config(&self, layer: u8) -> Option<&LayerConfig> {
        self.layer_configs.iter().find(|config| config.layer == layer)
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl LtdcInterface for MockLtdcInterface {
    type Error = MockLtdcError;
    
    fn configure_layer(&mut self, layer: u8, config: LayerConfig) -> Result<(), Self::Error> {
        if self.should_fail {
            return Err(MockLtdcError::SimulatedFailure);
        }
        
        if layer > 7 {
            return Err(MockLtdcError::InvalidLayer);
        }
        
        // Remove existing config for this layer
        self.layer_configs.retain(|c| c.layer != layer);
        
        // Add new config
        self.layer_configs.push(config);
        Ok(())
    }
    
    fn enable(&mut self) -> Result<(), Self::Error> {
        if self.should_fail {
            return Err(MockLtdcError::SimulatedFailure);
        }
        
        self.enabled = true;
        Ok(())
    }
    
    fn disable(&mut self) -> Result<(), Self::Error> {
        if self.should_fail {
            return Err(MockLtdcError::SimulatedFailure);
        }
        
        self.enabled = false;
        Ok(())
    }
    
    fn set_framebuffer(&mut self, layer: u8, address: u32) -> Result<(), Self::Error> {
        if self.should_fail {
            return Err(MockLtdcError::SimulatedFailure);
        }
        
        if layer > 7 {
            return Err(MockLtdcError::InvalidLayer);
        }
        
        // Remove existing framebuffer for this layer
        self.framebuffer_addresses.retain(|(l, _)| *l != layer);
        
        // Add new framebuffer address
        self.framebuffer_addresses.push((layer, address));
        Ok(())
    }
    
    fn get_dimensions(&self) -> (u16, u16) {
        self.dimensions
    }
}

/// Mock framebuffer for testing
#[derive(Debug)]
pub struct MockFramebuffer {
    buffer: Vec<u16>,
    width: u16,
    height: u16,
}

impl MockFramebuffer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        let buffer = vec![0; size]; // Initialize to black
        
        Self {
            buffer,
            width,
            height,
        }
    }
    
    pub fn get_pixel(&self, x: u16, y: u16) -> Option<u16> {
        if x >= self.width || y >= self.height {
            return None;
        }
        
        let index = (y as usize) * (self.width as usize) + (x as usize);
        self.buffer.get(index).copied()
    }
}

impl FramebufferInterface for MockFramebuffer {
    fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: u16) {
        let end_x = core::cmp::min(x + width, self.width);
        let end_y = core::cmp::min(y + height, self.height);
        
        for row in y..end_y {
            for col in x..end_x {
                let index = (row as usize) * (self.width as usize) + (col as usize);
                if let Some(pixel) = self.buffer.get_mut(index) {
                    *pixel = color;
                }
            }
        }
    }
    
    fn set_pixel(&mut self, x: u16, y: u16, color: u16) {
        if x < self.width && y < self.height {
            let index = (y as usize) * (self.width as usize) + (x as usize);
            if let Some(pixel) = self.buffer.get_mut(index) {
                *pixel = color;
            }
        }
    }
    
    fn clear(&mut self, color: u16) {
        for pixel in self.buffer.iter_mut() {
            *pixel = color;
        }
    }
    
    fn get_dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }
    
    fn get_buffer_ptr(&self) -> *const u16 {
        self.buffer.as_ptr()
    }
    
    fn get_buffer_size(&self) -> usize {
        self.buffer.len() * 2 // 2 bytes per u16
    }
}

/// Mock platform interface for testing
#[derive(Debug)]
pub struct MockPlatformInterface {
    pub initialized: bool,
    pub should_fail: bool,
    pub low_power_mode: bool,
    pub platform_info: &'static str,
}

#[derive(Debug)]
pub enum MockPlatformError {
    InitializationFailed,
    SimulatedFailure,
}

impl MockPlatformInterface {
    pub fn new(platform_info: &'static str) -> Self {
        Self {
            initialized: false,
            should_fail: false,
            low_power_mode: false,
            platform_info,
        }
    }
    
    pub fn set_should_fail(&mut self, fail: bool) {
        self.should_fail = fail;
    }
}

impl PlatformInterface for MockPlatformInterface {
    type Error = MockPlatformError;
    
    fn init_platform(&mut self) -> Result<(), Self::Error> {
        if self.should_fail {
            return Err(MockPlatformError::InitializationFailed);
        }
        
        self.initialized = true;
        Ok(())
    }
    
    fn get_platform_info(&self) -> &'static str {
        self.platform_info
    }
    
    fn enter_low_power(&mut self) -> Result<(), Self::Error> {
        if self.should_fail {
            return Err(MockPlatformError::SimulatedFailure);
        }
        
        self.low_power_mode = true;
        Ok(())
    }
    
    fn exit_low_power(&mut self) -> Result<(), Self::Error> {
        if self.should_fail {
            return Err(MockPlatformError::SimulatedFailure);
        }
        
        self.low_power_mode = false;
        Ok(())
    }
}