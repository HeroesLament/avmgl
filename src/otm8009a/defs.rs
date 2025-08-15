//! Constants and definitions for the OTM8009A display driver

/// Display dimensions
pub const LCD_WIDTH: u16 = 800;
pub const LCD_HEIGHT: u16 = 480;

/// Color format constants
pub const OTM8009A_FORMAT_RGB565: u32 = 0x55;
pub const OTM8009A_FORMAT_RGB888: u32 = 0x77;
pub const OTM8009A_FORMAT_RGB666: u32 = 0x66;

/// Orientation constants
pub const OTM8009A_ORIENTATION_PORTRAIT: u32 = 0;
pub const OTM8009A_ORIENTATION_LANDSCAPE: u32 = 1;
pub const OTM8009A_ORIENTATION_PORTRAIT_FLIPPED: u32 = 2;
pub const OTM8009A_ORIENTATION_LANDSCAPE_FLIPPED: u32 = 3;

/// OTM8009A specific DCS commands
pub mod commands {
    /// Standard DCS commands
    pub const SLEEP_OUT: u8 = 0x11;
    pub const DISPLAY_ON: u8 = 0x29;
    pub const DISPLAY_OFF: u8 = 0x28;
    pub const SLEEP_IN: u8 = 0x10;
    pub const NOP: u8 = 0x00;
    
    /// Memory access control
    pub const SET_MEMORY_ACCESS_CONTROL: u8 = 0x36;
    pub const SET_PIXEL_FORMAT: u8 = 0x3A;
    pub const SET_COLUMN_ADDRESS: u8 = 0x2A;
    pub const SET_PAGE_ADDRESS: u8 = 0x2B;
    pub const WRITE_MEMORY_START: u8 = 0x2C;
    
    /// Manufacturer specific commands
    pub const SET_EXTC: u8 = 0xFF;
    pub const SET_MIPI: u8 = 0xE3;
    pub const SET_FUNCTION_CTRL: u8 = 0xB6;
    pub const SET_POWER_CTRL1: u8 = 0xC0;
    pub const SET_POWER_CTRL2: u8 = 0xC1;
    pub const SET_VCOM_CTRL1: u8 = 0xC5;
    pub const SET_VCOM_CTRL2: u8 = 0xC7;
    pub const SET_GAMMA_CTRL1: u8 = 0xE0;
    pub const SET_GAMMA_CTRL2: u8 = 0xE1;
    
    /// CABC commands
    pub const WRITE_CTRL_DISPLAY: u8 = 0x53;
    pub const WRITE_CABC: u8 = 0x55;
    pub const WRITE_CABC_MIN_BRIGHTNESS: u8 = 0x5E;
}

/// Command data sequences for initialization
pub mod init_sequences {
    /// Enable CMD2 to access vendor specific commands
    pub const CMD_EXTC: [u8; 4] = [0xFF, 0x80, 0x09, 0x01];
    
    /// Enter ORISE Command 2
    pub const CMD_ORISE_ENTER: [u8; 3] = [0x80, 0x09, 0x00];
    
    /// GVDD/NGVDD settings  
    pub const CMD_GVDD_NGVDD: [u8; 3] = [0xC5, 0x17, 0x40];
    
    /// Exit CMD2 mode
    pub const CMD_EXIT_CMD2: [u8; 4] = [0xFF, 0x00, 0x00, 0x00];
    
    /// Gamma correction positive
    pub const CMD_GAMMA_POSITIVE: [u8; 17] = [
        0xE0, 0x00, 0x09, 0x0F, 0x0E, 0x07, 0x10, 0x0B, 0x0A, 0x04, 0x07, 0x0B, 0x08, 0x0F, 0x10, 0x0A, 0x01
    ];
    
    /// Gamma correction negative
    pub const CMD_GAMMA_NEGATIVE: [u8; 17] = [
        0xE1, 0x00, 0x09, 0x0F, 0x0E, 0x07, 0x10, 0x0B, 0x0A, 0x04, 0x07, 0x0B, 0x08, 0x0F, 0x10, 0x0A, 0x01
    ];
    
    /// Color format commands
    pub const CMD_RGB565: [u8; 2] = [0x3A, 0x55];
    pub const CMD_RGB888: [u8; 2] = [0x3A, 0x77];
    pub const CMD_RGB666: [u8; 2] = [0x3A, 0x66];
    
    /// Orientation commands
    pub const CMD_PORTRAIT: [u8; 2] = [0x36, 0x00];
    pub const CMD_LANDSCAPE: [u8; 2] = [0x36, 0x60];
    pub const CMD_PORTRAIT_FLIPPED: [u8; 2] = [0x36, 0xC0];
    pub const CMD_LANDSCAPE_FLIPPED: [u8; 2] = [0x36, 0xA0];
    
    /// Column address set for different orientations
    pub const CMD_CASET_LANDSCAPE: [u8; 5] = [0x2A, 0x00, 0x00, 0x03, 0x1F]; // 0-799
    pub const CMD_CASET_PORTRAIT: [u8; 5] = [0x2A, 0x00, 0x00, 0x01, 0xDF];  // 0-479
    
    /// Page address set for different orientations  
    pub const CMD_PASET_LANDSCAPE: [u8; 5] = [0x2B, 0x00, 0x00, 0x01, 0xDF]; // 0-479
    pub const CMD_PASET_PORTRAIT: [u8; 5] = [0x2B, 0x00, 0x00, 0x03, 0x1F];  // 0-799
    
    /// CABC (Content Adaptive Backlight Control) commands
    pub const CMD_BRIGHTNESS_CTRL: [u8; 2] = [0x53, 0x24];
    pub const CMD_CABC_CTRL: [u8; 2] = [0x55, 0x00];
    pub const CMD_CABC_MIN_BRIGHTNESS: [u8; 2] = [0x5E, 0x00];
    
    /// Final commands
    pub const CMD_DISPLAY_ON: [u8; 1] = [0x29];
    pub const CMD_WRITE_MEMORY_START: [u8; 1] = [0x2C];
    pub const CMD_SLEEP_OUT: [u8; 1] = [0x11];
    pub const CMD_NOP: [u8; 1] = [0x00];
}

/// Single byte commands (SHORT_REG_DATA equivalents)
pub mod single_commands {
    pub const NOP: [u8; 1] = [0x00];
    pub const SLEEP_OUT: [u8; 1] = [0x11];
    pub const DISPLAY_ON: [u8; 1] = [0x29];
    pub const WRITE_MEMORY_START: [u8; 1] = [0x2C];
}

/// Display orientations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Portrait = 0,
    Landscape = 1,
    PortraitFlipped = 2,
    LandscapeFlipped = 3,
}

/// Color formats supported by OTM8009A
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    Rgb565 = 0x55,
    Rgb666 = 0x66,
    Rgb888 = 0x77,
}

/// Power modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMode {
    Normal,
    Sleep,
    PartialDisplay,
    IdleMode,
}

/// Timing parameters for display initialization
pub mod timing {
    /// Delays in milliseconds
    pub const RESET_DELAY_MS: u32 = 10;
    pub const SLEEP_OUT_DELAY_MS: u32 = 120;
    pub const DISPLAY_ON_DELAY_MS: u32 = 40;
    pub const POWER_ON_DELAY_MS: u32 = 50;
    pub const CMD_DELAY_MS: u32 = 1;
}

/// Configuration presets
pub mod presets {
    use super::*;
    
    /// Standard configuration for 800x480 landscape
    pub const STANDARD_LANDSCAPE: DisplayConfig = DisplayConfig {
        width: LCD_WIDTH,
        height: LCD_HEIGHT,
        orientation: Orientation::Landscape,
        color_format: ColorFormat::Rgb565,
        power_mode: PowerMode::Normal,
    };
    
    /// Standard configuration for 480x800 portrait
    pub const STANDARD_PORTRAIT: DisplayConfig = DisplayConfig {
        width: LCD_HEIGHT,
        height: LCD_WIDTH,
        orientation: Orientation::Portrait,
        color_format: ColorFormat::Rgb565,
        power_mode: PowerMode::Normal,
    };
}

/// Display configuration structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayConfig {
    pub width: u16,
    pub height: u16,
    pub orientation: Orientation,
    pub color_format: ColorFormat,
    pub power_mode: PowerMode,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        presets::STANDARD_LANDSCAPE
    }
}

/// Error types specific to OTM8009A
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Otm8009aError {
    /// Display not ready
    NotReady,
    /// Invalid configuration
    InvalidConfig,
    /// Communication error
    CommError,
    /// Timeout waiting for display
    Timeout,
    /// Invalid coordinates
    InvalidCoordinates,
    /// Unsupported operation
    Unsupported,
}

impl core::fmt::Display for Otm8009aError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Otm8009aError::NotReady => write!(f, "Display not ready"),
            Otm8009aError::InvalidConfig => write!(f, "Invalid configuration"),
            Otm8009aError::CommError => write!(f, "Communication error"),
            Otm8009aError::Timeout => write!(f, "Timeout"),
            Otm8009aError::InvalidCoordinates => write!(f, "Invalid coordinates"),
            Otm8009aError::Unsupported => write!(f, "Unsupported operation"),
        }
    }
}