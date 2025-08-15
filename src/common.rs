//! Common utilities for display operations

/// Convert RGB888 to RGB565 format
pub fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    let r5 = (r >> 3) as u16;
    let g6 = (g >> 2) as u16;
    let b5 = (b >> 3) as u16;
    (r5 << 11) | (g6 << 5) | b5
}

/// Convert RGB565 to RGB888 format
pub fn rgb565_to_rgb888(color: u16) -> (u8, u8, u8) {
    let r = ((color >> 11) & 0x1F) as u8;
    let g = ((color >> 5) & 0x3F) as u8;
    let b = (color & 0x1F) as u8;
    
    // Expand to 8-bit values
    let r8 = (r << 3) | (r >> 2);
    let g8 = (g << 2) | (g >> 4);
    let b8 = (b << 3) | (b >> 2);
    
    (r8, g8, b8)
}