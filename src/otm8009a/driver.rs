//! OTM8009A Driver Implementation
//! 
//! Hardware abstraction layer for the OTM8009A display controller.
//! This driver is hardware-agnostic and works with any DSI/LTDC implementation.

use crate::otm8009a::defs::*;
use crate::traits::*;

pub struct OTM8009ADriver<D, L, F> 
where
    D: DsiInterface,
    L: LtdcInterface,
    F: FramebufferInterface,
{
    dsi: D,
    ltdc: L,
    framebuffer: F,
    width: u16,
    height: u16,
    initialized: bool,
}

impl<D, L, F> OTM8009ADriver<D, L, F> 
where
    D: DsiInterface,
    L: LtdcInterface,
    F: FramebufferInterface,
{
    pub fn new(dsi: D, ltdc: L, framebuffer: F) -> Self {
        Self {
            dsi,
            ltdc,
            framebuffer,
            width: LCD_WIDTH,
            height: LCD_HEIGHT,
            initialized: false,
        }
    }

    pub fn init(&mut self, color_format: u32, orientation: u32) -> Result<(), Otm8009aError> {
        // Initialize the OTM8009A display controller
        self.init_otm8009a(color_format, orientation)?;
        
        // Configure the LTDC layer
        let layer_config = LayerConfig {
            layer: 0,
            window_x0: 0,
            window_x1: self.width,
            window_y0: 0, 
            window_y1: self.height,
            pixel_format: match color_format {
                OTM8009A_FORMAT_RGB565 => PixelFormat::Rgb565,
                OTM8009A_FORMAT_RGB888 => PixelFormat::Rgb888,
                OTM8009A_FORMAT_RGB666 => PixelFormat::Rgb888, // Map RGB666 to RGB888
                _ => return Err(Otm8009aError::InvalidConfig),
            },
            alpha: 255,
            red_blue_swap: false,
            framebuffer_address: self.framebuffer.get_buffer_ptr() as u32,
            framebuffer_pitch: self.width * 2, // 2 bytes per pixel for RGB565
        };
        
        self.ltdc.configure_layer(0, layer_config)
            .map_err(|_| Otm8009aError::CommError)?;
        
        // Set framebuffer address
        let fb_addr = self.framebuffer.get_buffer_ptr() as u32;
        self.ltdc.set_framebuffer(0, fb_addr)
            .map_err(|_| Otm8009aError::CommError)?;
        
        // Enable LTDC
        self.ltdc.enable()
            .map_err(|_| Otm8009aError::CommError)?;
        
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn get_dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: u16) -> Result<(), Otm8009aError> {
        if !self.initialized {
            return Err(Otm8009aError::NotReady);
        }
        
        if x >= self.width || y >= self.height {
            return Err(Otm8009aError::InvalidCoordinates);
        }
        
        self.framebuffer.fill_rect(x, y, width, height, color);
        Ok(())
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, color: u16) -> Result<(), Otm8009aError> {
        if !self.initialized {
            return Err(Otm8009aError::NotReady);
        }
        
        if x >= self.width || y >= self.height {
            return Err(Otm8009aError::InvalidCoordinates);
        }
        
        self.framebuffer.set_pixel(x, y, color);
        Ok(())
    }

    pub fn clear(&mut self, color: u16) -> Result<(), Otm8009aError> {
        if !self.initialized {
            return Err(Otm8009aError::NotReady);
        }
        
        self.framebuffer.clear(color);
        Ok(())
    }

    pub fn set_orientation(&mut self, orientation: u32) -> Result<(), Otm8009aError> {
        if !self.initialized {
            return Err(Otm8009aError::NotReady);
        }
        
        let (cmd, caset, paset) = match orientation {
            OTM8009A_ORIENTATION_PORTRAIT => (
                &init_sequences::CMD_PORTRAIT,
                &init_sequences::CMD_CASET_PORTRAIT,
                &init_sequences::CMD_PASET_PORTRAIT,
            ),
            OTM8009A_ORIENTATION_LANDSCAPE => (
                &init_sequences::CMD_LANDSCAPE,
                &init_sequences::CMD_CASET_LANDSCAPE,
                &init_sequences::CMD_PASET_LANDSCAPE,
            ),
            OTM8009A_ORIENTATION_PORTRAIT_FLIPPED => (
                &init_sequences::CMD_PORTRAIT_FLIPPED,
                &init_sequences::CMD_CASET_PORTRAIT,
                &init_sequences::CMD_PASET_PORTRAIT,
            ),
            OTM8009A_ORIENTATION_LANDSCAPE_FLIPPED => (
                &init_sequences::CMD_LANDSCAPE_FLIPPED,
                &init_sequences::CMD_CASET_LANDSCAPE,
                &init_sequences::CMD_PASET_LANDSCAPE,
            ),
            _ => return Err(Otm8009aError::InvalidConfig),
        };
        
        // Set orientation
        self.dsi.send_dcs_command(cmd.len() - 1, &cmd[1..])
            .map_err(|_| Otm8009aError::CommError)?;
        
        // Set column address
        self.dsi.send_dcs_command(caset.len() - 1, &caset[1..])
            .map_err(|_| Otm8009aError::CommError)?;
        
        // Set page address
        self.dsi.send_dcs_command(paset.len() - 1, &paset[1..])
            .map_err(|_| Otm8009aError::CommError)?;
        
        // Update dimensions based on orientation
        match orientation {
            OTM8009A_ORIENTATION_PORTRAIT | OTM8009A_ORIENTATION_PORTRAIT_FLIPPED => {
                self.width = LCD_HEIGHT;
                self.height = LCD_WIDTH;
            }
            OTM8009A_ORIENTATION_LANDSCAPE | OTM8009A_ORIENTATION_LANDSCAPE_FLIPPED => {
                self.width = LCD_WIDTH;
                self.height = LCD_HEIGHT;
            }
            _ => return Err(Otm8009aError::InvalidConfig),
        }
        
        Ok(())
    }

    fn init_otm8009a(&mut self, color_format: u32, orientation: u32) -> Result<(), Otm8009aError> {
        // Enable CMD2 to access vendor specific commands
        self.dsi.send_dcs_command(
            init_sequences::CMD_EXTC.len() - 1, 
            &init_sequences::CMD_EXTC[1..]
        ).map_err(|_| Otm8009aError::CommError)?;

        self.dsi.delay_ms(timing::CMD_DELAY_MS);

        // Enter ORISE Command 2
        self.dsi.send_dcs_command(
            init_sequences::CMD_ORISE_ENTER.len(),
            &init_sequences::CMD_ORISE_ENTER
        ).map_err(|_| Otm8009aError::CommError)?;

        self.dsi.delay_ms(timing::CMD_DELAY_MS);

        // GVDD/NGVDD settings
        self.dsi.send_dcs_command(
            init_sequences::CMD_GVDD_NGVDD.len() - 1,
            &init_sequences::CMD_GVDD_NGVDD[1..]
        ).map_err(|_| Otm8009aError::CommError)?;

        self.dsi.delay_ms(timing::CMD_DELAY_MS);

        // Exit CMD2 mode
        self.dsi.send_dcs_command(
            init_sequences::CMD_EXIT_CMD2.len() - 1,
            &init_sequences::CMD_EXIT_CMD2[1..]
        ).map_err(|_| Otm8009aError::CommError)?;

        self.dsi.delay_ms(timing::CMD_DELAY_MS);

        // Send NOP
        self.dsi.send_dcs_command(0, &[])
            .map_err(|_| Otm8009aError::CommError)?;

        self.dsi.delay_ms(timing::CMD_DELAY_MS);

        // Gamma correction tables
        self.dsi.send_dcs_command(
            init_sequences::CMD_GAMMA_POSITIVE.len() - 1,
            &init_sequences::CMD_GAMMA_POSITIVE[1..]
        ).map_err(|_| Otm8009aError::CommError)?;
        
        self.dsi.send_dcs_command(
            init_sequences::CMD_GAMMA_NEGATIVE.len() - 1,
            &init_sequences::CMD_GAMMA_NEGATIVE[1..]
        ).map_err(|_| Otm8009aError::CommError)?;

        self.dsi.delay_ms(timing::CMD_DELAY_MS);

        // Sleep out
        self.dsi.send_dcs_command(0, &[])
            .map_err(|_| Otm8009aError::CommError)?;
        self.dsi.delay_ms(timing::SLEEP_OUT_DELAY_MS);

        // Set color format
        let color_cmd = match color_format {
            OTM8009A_FORMAT_RGB565 => &init_sequences::CMD_RGB565,
            OTM8009A_FORMAT_RGB888 => &init_sequences::CMD_RGB888,
            OTM8009A_FORMAT_RGB666 => &init_sequences::CMD_RGB666,
            _ => return Err(Otm8009aError::InvalidConfig),
        };

        self.dsi.send_dcs_command(
            color_cmd.len() - 1,
            &color_cmd[1..]
        ).map_err(|_| Otm8009aError::CommError)?;

        self.dsi.delay_ms(timing::CMD_DELAY_MS);

        // Set orientation
        self.set_orientation(orientation)?;

        // CABC: Content Adaptive Backlight Control
        self.dsi.send_dcs_command(
            init_sequences::CMD_BRIGHTNESS_CTRL.len() - 1,
            &init_sequences::CMD_BRIGHTNESS_CTRL[1..]
        ).map_err(|_| Otm8009aError::CommError)?;
        
        self.dsi.send_dcs_command(
            init_sequences::CMD_CABC_CTRL.len() - 1,
            &init_sequences::CMD_CABC_CTRL[1..]
        ).map_err(|_| Otm8009aError::CommError)?;
        
        self.dsi.send_dcs_command(
            init_sequences::CMD_CABC_MIN_BRIGHTNESS.len() - 1,
            &init_sequences::CMD_CABC_MIN_BRIGHTNESS[1..]
        ).map_err(|_| Otm8009aError::CommError)?;

        self.dsi.delay_ms(timing::CMD_DELAY_MS);

        // Display on
        self.dsi.send_dcs_command(0, &[])
            .map_err(|_| Otm8009aError::CommError)?;
        
        self.dsi.delay_ms(timing::DISPLAY_ON_DELAY_MS);

        // Send final NOP
        self.dsi.send_dcs_command(0, &[])
            .map_err(|_| Otm8009aError::CommError)?;
        
        // Start GRAM write
        self.dsi.send_dcs_command(0, &[])
            .map_err(|_| Otm8009aError::CommError)?;

        Ok(())
    }

    pub fn enter_sleep(&mut self) -> Result<(), Otm8009aError> {
        if !self.initialized {
            return Err(Otm8009aError::NotReady);
        }

        // Turn off display
        self.dsi.send_dcs_command(0, &[])
            .map_err(|_| Otm8009aError::CommError)?;
        
        self.dsi.delay_ms(timing::DISPLAY_ON_DELAY_MS);

        // Enter sleep mode
        self.dsi.send_dcs_command(0, &[])
            .map_err(|_| Otm8009aError::CommError)?;
        
        self.dsi.delay_ms(timing::SLEEP_OUT_DELAY_MS);

        Ok(())
    }

    pub fn exit_sleep(&mut self) -> Result<(), Otm8009aError> {
        if !self.initialized {
            return Err(Otm8009aError::NotReady);
        }

        // Sleep out
        self.dsi.send_dcs_command(0, &[])
            .map_err(|_| Otm8009aError::CommError)?;
        
        self.dsi.delay_ms(timing::SLEEP_OUT_DELAY_MS);

        // Turn on display
        self.dsi.send_dcs_command(0, &[])
            .map_err(|_| Otm8009aError::CommError)?;
        
        self.dsi.delay_ms(timing::DISPLAY_ON_DELAY_MS);

        Ok(())
    }

    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), Otm8009aError> {
        if !self.initialized {
            return Err(Otm8009aError::NotReady);
        }

        let brightness_cmd = [commands::WRITE_CTRL_DISPLAY, brightness];
        self.dsi.send_dcs_command(1, &brightness_cmd[1..])
            .map_err(|_| Otm8009aError::CommError)?;

        Ok(())
    }

    pub fn enable_cabc(&mut self, mode: u8) -> Result<(), Otm8009aError> {
        if !self.initialized {
            return Err(Otm8009aError::NotReady);
        }

        if mode > 3 {
            return Err(Otm8009aError::InvalidConfig);
        }

        let cabc_cmd = [commands::WRITE_CABC, mode];
        self.dsi.send_dcs_command(1, &cabc_cmd[1..])
            .map_err(|_| Otm8009aError::CommError)?;

        Ok(())
    }

    pub fn disable_cabc(&mut self) -> Result<(), Otm8009aError> {
        self.enable_cabc(0)
    }

    /// Get the framebuffer interface for direct pixel operations
    pub fn framebuffer(&self) -> &F {
        &self.framebuffer
    }

    /// Get mutable access to the framebuffer interface
    pub fn framebuffer_mut(&mut self) -> &mut F {
        &mut self.framebuffer
    }

    /// Check if the DSI interface is ready
    pub fn is_dsi_ready(&self) -> bool {
        self.dsi.is_ready()
    }

    /// Reset the display
    pub fn reset(&mut self) -> Result<(), Otm8009aError> {
        self.dsi.reset()
            .map_err(|_| Otm8009aError::CommError)?;
        
        self.dsi.delay_ms(timing::RESET_DELAY_MS);
        self.initialized = false;
        
        Ok(())
    }
}

// NIF bindings - only compiled for production builds
#[cfg(feature = "nifs")]
mod nif_bindings {
    use avmnif_rs::{
        nif_collection,
        term::{Context, Term, TermValue, NifResult, NifError, Heap},
    };
    use super::*;

    // Global display handle placeholder - in a real implementation,
    // you'd need proper resource management
    static mut DISPLAY_HANDLE: Option<()> = None;

    fn display_init_nif(ctx: &Context, args: &[Term]) -> NifResult<Term> {
        if args.len() != 1 {
            return Err(NifError::BadArity);
        }

        // Extract configuration term
        let config_term = args[0];
        let config_value = config_term.to_value()?;

        // Parse configuration - validate it's a tuple
        let _config_tuple = config_value.as_tuple().ok_or(NifError::BadArg)?;

        // TODO: Initialize actual display driver
        // let mut driver = OTM8009ADriver::new(dsi, ltdc, framebuffer);
        // driver.init(color_format, orientation)?;

        // Return ok atom - need to get heap from context and atom table
        // For now, return a placeholder that compiles
        // In real implementation, you'd need:
        // let heap = ctx.heap_mut();
        // let atom_table = ctx.atom_table();
        // let ok_atom = TermValue::atom("ok", atom_table);
        // Term::from_value(ok_atom, heap)
        
        Ok(Term::from_raw(0)) // Placeholder until proper context API is available
    }

    fn display_fill_rect_nif(_ctx: &Context, args: &[Term]) -> NifResult<Term> {
        if args.len() != 6 {
            return Err(NifError::BadArity);
        }

        // Extract arguments: X, Y, Width, Height, Color, Handle
        let x = args[0].to_value()?.as_int().ok_or(NifError::BadArg)?;
        let y = args[1].to_value()?.as_int().ok_or(NifError::BadArg)?;
        let width = args[2].to_value()?.as_int().ok_or(NifError::BadArg)?;
        let height = args[3].to_value()?.as_int().ok_or(NifError::BadArg)?;
        let color = args[4].to_value()?.as_int().ok_or(NifError::BadArg)?;
        let _handle = args[5];

        // Validate parameters
        if x < 0 || y < 0 || width <= 0 || height <= 0 || color < 0 {
            return Err(NifError::BadArg);
        }

        // TODO: Get driver from handle and call fill_rect
        // driver.fill_rect(x as u16, y as u16, width as u16, height as u16, color as u16)?;

        Ok(Term::from_raw(0)) // Placeholder
    }

    fn display_set_pixel_nif(_ctx: &Context, args: &[Term]) -> NifResult<Term> {
        if args.len() != 4 {
            return Err(NifError::BadArity);
        }

        // Extract arguments: X, Y, Color, Handle
        let x = args[0].to_value()?.as_int().ok_or(NifError::BadArg)?;
        let y = args[1].to_value()?.as_int().ok_or(NifError::BadArg)?;
        let color = args[2].to_value()?.as_int().ok_or(NifError::BadArg)?;
        let _handle = args[3];

        // Validate parameters
        if x < 0 || y < 0 || color < 0 {
            return Err(NifError::BadArg);
        }

        // TODO: Get driver from handle and call set_pixel
        // driver.set_pixel(x as u16, y as u16, color as u16)?;

        Ok(Term::from_raw(0)) // Placeholder
    }

    fn display_get_info_nif(_ctx: &Context, args: &[Term]) -> NifResult<Term> {
        if args.len() != 1 {
            return Err(NifError::BadArity);
        }

        let _handle = args[0];

        // TODO: Get actual dimensions from driver and create proper terms
        // let (width, height) = driver.get_dimensions();
        // 
        // Real implementation would be:
        // let heap = ctx.heap_mut();
        // let atom_table = ctx.atom_table();
        // let info_tuple = TermValue::tuple(vec![
        //     TermValue::int(width as i32),
        //     TermValue::int(height as i32), 
        //     TermValue::atom("rgb565", atom_table)
        // ]);
        // let result = TermValue::tuple(vec![
        //     TermValue::atom("ok", atom_table),
        //     info_tuple
        // ]);
        // Term::from_value(result, heap)
        
        Ok(Term::from_raw(0)) // Placeholder
    }

    fn display_clear_nif(_ctx: &Context, args: &[Term]) -> NifResult<Term> {
        if args.len() != 1 {
            return Err(NifError::BadArg);
        }

        let _handle = args[0];

        // TODO: Get driver from handle and call clear
        // driver.clear(0x0000)?; // Clear to black

        Ok(Term::from_raw(0)) // Placeholder
    }

    // Register the NIF collection
    nif_collection!(
        display,
        init = display_nif_init,
        nifs = [
            ("init", 1, display_init_nif),
            ("fill_rect", 6, display_fill_rect_nif),
            ("set_pixel", 4, display_set_pixel_nif),
            ("get_info", 1, display_get_info_nif),
            ("clear", 1, display_clear_nif),
        ]
    );
}