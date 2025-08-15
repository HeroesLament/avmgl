// src/platforms/stm32/display_stm32f769i.rs

use avmnif_rs::{
    term::{Term, NifResult, NifError},
    nif_collection,
    atom::atoms::{ok, error},
};
use stm32f7xx_hal::{
    pac::{DSI, LTDC, RCC},
    prelude::*,
};
use core::ptr;

// Display configuration for STM32F769I-DISCO
const LCD_WIDTH: u16 = 480;
const LCD_HEIGHT: u16 = 272;
const LCD_PIXEL_FORMAT: u8 = 2; // RGB565

// OTM8009A display controller registers
const OTM8009A_CMD_NOP: u8 = 0x00;
const OTM8009A_CMD_SWRESET: u8 = 0x01;
const OTM8009A_CMD_SLPIN: u8 = 0x10;
const OTM8009A_CMD_SLPOUT: u8 = 0x11;
const OTM8009A_CMD_DISPOFF: u8 = 0x28;
const OTM8009A_CMD_DISPON: u8 = 0x29;

// Global display state
static mut FRAMEBUFFER: [u16; (LCD_WIDTH as usize) * (LCD_HEIGHT as usize)] = [0; (LCD_WIDTH as usize) * (LCD_HEIGHT as usize)];
static mut DISPLAY_INITIALIZED: bool = false;

pub struct DisplayDriver {
    dsi: DSI,
    ltdc: LTDC,
    width: u16,
    height: u16,
}

impl DisplayDriver {
    pub fn new(dsi: DSI, ltdc: LTDC) -> Self {
        Self {
            dsi,
            ltdc,
            width: LCD_WIDTH,
            height: LCD_HEIGHT,
        }
    }

    /// Initialize the MIPI DSI interface
    fn init_dsi(&mut self) -> Result<(), &'static str> {
        // Enable DSI clock
        unsafe {
            let rcc = &(*RCC::ptr());
            rcc.apb2enr.modify(|_, w| w.dsien().set_bit());
            rcc.apb2rstr.modify(|_, w| w.dsirst().set_bit());
            rcc.apb2rstr.modify(|_, w| w.dsirst().clear_bit());
        }

        // Configure DSI for OTM8009A
        self.dsi.wrpcr.write(|w| unsafe {
            w.ndiv().bits(125)  // PLL multiplication factor
                .idf().bits(2)   // PLL input division factor  
                .odf().bits(1)   // PLL output division factor
        });

        // Enable DSI PLL
        self.dsi.wrpcr.modify(|_, w| w.pllen().set_bit());
        
        // Wait for PLL lock
        while !self.dsi.wisr.read().pllls().bit_is_set() {}

        // Configure DSI wrapper
        self.dsi.wcfgr.write(|w| unsafe {
            w.dsim().set_bit()     // DSI mode
                .colmux().bits(5)   // Color multiplexing (RGB565)
        });

        // Configure virtual channel
        self.dsi.vcccr.write(|w| unsafe { w.numc().bits(0) }); // Virtual channel 0

        // Configure command mode
        self.dsi.cmcr.write(|w| unsafe {
            w.teare().clear_bit()    // Tearing effect acknowledge disable
                .are().clear_bit()    // Automatic refresh disable
                .gsw0tx().bits(0)     // Generic short write zero parameters
                .gsw1tx().bits(0)     // Generic short write one parameter
                .gsw2tx().bits(0)     // Generic short write two parameters
                .gsr0tx().bits(0)     // Generic short read zero parameters
                .gsr1tx().bits(0)     // Generic short read one parameter
                .gsr2tx().bits(0)     // Generic short read two parameters
                .glwtx().bits(0)      // Generic long write
                .dsw0tx().bits(0)     // DCS short write zero parameters
                .dsw1tx().bits(0)     // DCS short write one parameter
                .dsr0tx().bits(0)     // DCS short read
                .dlwtx().bits(0)      // DCS long write
                .mrdps().bits(24)     // Maximum read packet size
        });

        // Configure timing
        self.dsi.cltcr.write(|w| unsafe {
            w.lp2hs_time().bits(35)
                .hs2lp_time().bits(35)
        });

        self.dsi.dltcr.write(|w| unsafe {
            w.mrd_time().bits(0)
                .lp2hs_time().bits(35)
                .hs2lp_time().bits(35)
        });

        // Configure physical layer
        self.dsi.pctlr.write(|w| unsafe {
            w.den().set_bit()      // Digital enable
                .ck_iddly().bits(0) // Clock lane idle delay
                .ck_pdly().bits(0)  // Clock lane post delay
        });

        // Configure DSI host timeouts
        self.dsi.tccr0.write(|w| unsafe {
            w.lprx_tocnt().bits(0xFFFF)
                .hstx_tocnt().bits(0xFFFF)
        });

        Ok(())
    }

    /// Send DCS command to display
    fn send_dcs_command(&mut self, cmd: u8, params: &[u8]) -> Result<(), &'static str> {
        // Wait for command FIFO to be empty
        while !self.dsi.gpsr.read().cmdfe().bit_is_set() {}

        if params.is_empty() {
            // DCS short write, no parameters
            self.dsi.ghcr.write(|w| unsafe {
                w.dt().bits(0x05)    // DCS short write, no parameters
                    .vcid().bits(0)   // Virtual channel ID
                    .wclsb().bits(cmd) // Command
                    .wcmsb().bits(0)
            });
        } else if params.len() == 1 {
            // DCS short write, one parameter
            self.dsi.ghcr.write(|w| unsafe {
                w.dt().bits(0x15)      // DCS short write, one parameter
                    .vcid().bits(0)     // Virtual channel ID
                    .wclsb().bits(cmd)  // Command
                    .wcmsb().bits(params[0])
            });
        } else {
            // DCS long write
            let total_len = params.len() + 1; // +1 for command byte
            self.dsi.ghcr.write(|w| unsafe {
                w.dt().bits(0x39)                    // DCS long write
                    .vcid().bits(0)                   // Virtual channel ID
                    .wclsb().bits((total_len & 0xFF) as u8)
                    .wcmsb().bits(((total_len >> 8) & 0xFF) as u8)
            });

            // Send command
            self.dsi.gpdr.write(|w| unsafe { w.data1().bits(cmd) });

            // Send parameters
            for chunk in params.chunks(4) {
                let mut data = 0u32;
                for (i, &byte) in chunk.iter().enumerate() {
                    data |= (byte as u32) << (i * 8);
                }
                self.dsi.gpdr.write(|w| unsafe { w.data1().bits(data) });
            }
        }

        // Wait for transmission complete
        while !self.dsi.wisr.read().teif().bit_is_set() && 
              !self.dsi.wisr.read().erif().bit_is_set() {}

        if self.dsi.wisr.read().erif().bit_is_set() {
            return Err("DSI transmission error");
        }

        Ok(())
    }

    /// Initialize OTM8009A display controller
    fn init_otm8009a(&mut self) -> Result<(), &'static str> {
        // Software reset
        self.send_dcs_command(OTM8009A_CMD_SWRESET, &[])?;
        
        // Wait for reset to complete
        cortex_m::asm::delay(120_000); // ~10ms delay at 12MHz

        // Sleep out
        self.send_dcs_command(OTM8009A_CMD_SLPOUT, &[])?;
        cortex_m::asm::delay(1_200_000); // ~100ms delay

        // OTM8009A specific initialization sequence
        // Enable command 2
        self.send_dcs_command(0xFF, &[0x80, 0x09, 0x01])?;
        
        // Enable Orise mode
        self.send_dcs_command(0x00, &[0x80])?;
        self.send_dcs_command(0xFF, &[0x80, 0x09])?;

        // Set panel resolution
        self.send_dcs_command(0x00, &[0x03])?;
        self.send_dcs_command(0xFF, &[0x01])?;

        // Power control settings
        self.send_dcs_command(0x00, &[0x00])?;
        self.send_dcs_command(0xD8, &[0x74, 0x02])?;
        
        self.send_dcs_command(0x00, &[0x00])?;
        self.send_dcs_command(0xD9, &[0x5E])?;

        // Gamma correction
        self.send_dcs_command(0x00, &[0x00])?;
        self.send_dcs_command(0xE1, &[
            0x00, 0x09, 0x0F, 0x0E, 0x07, 0x10, 0x0B, 0x0A,
            0x04, 0x07, 0x0B, 0x08, 0x0F, 0x10, 0x0A, 0x01
        ])?;

        self.send_dcs_command(0x00, &[0x00])?;
        self.send_dcs_command(0xE2, &[
            0x00, 0x09, 0x0F, 0x0E, 0x07, 0x10, 0x0B, 0x0A,
            0x04, 0x07, 0x0B, 0x08, 0x0F, 0x10, 0x0A, 0x01
        ])?;

        // Display on
        self.send_dcs_command(OTM8009A_CMD_DISPON, &[])?;
        
        Ok(())
    }

    /// Initialize LTDC (LCD-TFT Display Controller)
    fn init_ltdc(&mut self) -> Result<(), &'static str> {
        // Enable LTDC clock
        unsafe {
            let rcc = &(*RCC::ptr());
            rcc.apb2enr.modify(|_, w| w.ltdcen().set_bit());
            rcc.apb2rstr.modify(|_, w| w.ltdcrst().set_bit());
            rcc.apb2rstr.modify(|_, w| w.ltdcrst().clear_bit());
        }

        // Configure synchronization size
        self.ltdc.sscr.write(|w| unsafe {
            w.hsw().bits(0)  // Horizontal sync width - 1
                .vsh().bits(0) // Vertical sync height - 1
        });

        // Configure back porch
        self.ltdc.bpcr.write(|w| unsafe {
            w.ahbp().bits(0)  // Accumulated horizontal back porch
                .avbp().bits(0) // Accumulated vertical back porch
        });

        // Configure active width/height
        self.ltdc.awcr.write(|w| unsafe {
            w.aaw().bits(LCD_WIDTH - 1)   // Accumulated active width
                .aah().bits(LCD_HEIGHT - 1) // Accumulated active height
        });

        // Configure total width/height
        self.ltdc.twcr.write(|w| unsafe {
            w.totalw().bits(LCD_WIDTH - 1)   // Total width
                .totalh().bits(LCD_HEIGHT - 1) // Total height
        });

        // Configure background color (black)
        self.ltdc.bccr.write(|w| unsafe {
            w.bcred().bits(0)
                .bcgreen().bits(0)
                .bcblue().bits(0)
        });

        // Configure layer 1
        self.configure_layer(1)?;

        // Enable LTDC
        self.ltdc.gcr.modify(|_, w| w.ltdcen().set_bit());

        Ok(())
    }

    fn configure_layer(&mut self, layer: u8) -> Result<(), &'static str> {
        if layer != 1 {
            return Err("Only layer 1 supported");
        }

        // Configure layer window
        self.ltdc.l1whpcr.write(|w| unsafe {
            w.whstpos().bits(0)              // Window horizontal start position
                .whsppos().bits(LCD_WIDTH - 1) // Window horizontal stop position
        });

        self.ltdc.l1wvpcr.write(|w| unsafe {
            w.wvstpos().bits(0)               // Window vertical start position
                .wvsppos().bits(LCD_HEIGHT - 1) // Window vertical stop position
        });

        // Configure pixel format (RGB565)
        self.ltdc.l1pfcr.write(|w| unsafe { w.pf().bits(2) });

        // Configure constant alpha
        self.ltdc.l1cacr.write(|w| unsafe { w.consta().bits(255) });

        // Configure default color
        self.ltdc.l1dccr.write(|w| unsafe {
            w.dcred().bits(0)
                .dcgreen().bits(0)
                .dcblue().bits(0)
                .dcalpha().bits(0)
        });

        // Configure blending factors
        self.ltdc.l1bfcr.write(|w| unsafe {
            w.bf1().bits(6)  // Constant alpha
                .bf2().bits(7) // Constant alpha
        });

        // Set frame buffer address
        let fb_addr = unsafe { FRAMEBUFFER.as_ptr() as u32 };
        self.ltdc.l1cfbar.write(|w| unsafe { w.cfbadd().bits(fb_addr) });

        // Configure line length and pitch
        let line_length = LCD_WIDTH * 2; // 2 bytes per pixel for RGB565
        self.ltdc.l1cfblr.write(|w| unsafe {
            w.cfbll().bits(line_length + 3) // Line length + 3
                .cfbp().bits(line_length)    // Pitch
        });

        // Configure number of lines
        self.ltdc.l1cfblnr.write(|w| unsafe { w.cfblnbr().bits(LCD_HEIGHT) });

        // Enable layer
        self.ltdc.l1cr.modify(|_, w| w.len().set_bit());

        // Reload configuration
        self.ltdc.srcr.write(|w| w.imr().set_bit());

        Ok(())
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        // Initialize DSI
        self.init_dsi()?;
        
        // Initialize display controller
        self.init_otm8009a()?;
        
        // Initialize LTDC
        self.init_ltdc()?;

        unsafe {
            DISPLAY_INITIALIZED = true;
        }

        Ok(())
    }

    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: u16) {
        if x >= LCD_WIDTH || y >= LCD_HEIGHT {
            return;
        }

        let end_x = core::cmp::min(x + width, LCD_WIDTH);
        let end_y = core::cmp::min(y + height, LCD_HEIGHT);

        unsafe {
            for row in y..end_y {
                let start_idx = (row as usize * LCD_WIDTH as usize) + x as usize;
                let end_idx = (row as usize * LCD_WIDTH as usize) + end_x as usize;
                
                for idx in start_idx..end_idx {
                    FRAMEBUFFER[idx] = color;
                }
            }
        }
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, color: u16) {
        if x < LCD_WIDTH && y < LCD_HEIGHT {
            unsafe {
                let idx = (y as usize * LCD_WIDTH as usize) + x as usize;
                FRAMEBUFFER[idx] = color;
            }
        }
    }

    pub fn get_dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }
}

// Helper function to convert RGB888 to RGB565
fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    let r5 = (r >> 3) as u16;
    let g6 = (g >> 2) as u16;
    let b5 = (b >> 3) as u16;
    (r5 << 11) | (g6 << 5) | b5
}

// NIF implementations following AtomGL patterns
nif_collection! {
    "display_stm32f769i_nif" => [
        ("display_init", 1, display_init_nif),
        ("display_fill_rect", 7, display_fill_rect_nif),
        ("display_set_pixel", 5, display_set_pixel_nif),
        ("display_get_info", 0, display_get_info_nif),
        ("display_clear", 1, display_clear_nif),
    ]
}

/// Initialize display - equivalent to display_init in AtomGL
fn display_init_nif(_env: &avmnif_rs::term::Context, _args: &[Term]) -> NifResult<Term> {
    // In a real implementation, you'd need to get the DSI and LTDC peripherals
    // This is a simplified example
    unsafe {
        if !DISPLAY_INITIALIZED {
            // For now, just mark as initialized
            // In practice, you'd call DisplayDriver::init() here
            DISPLAY_INITIALIZED = true;
        }
    }
    
    Ok(ok())
}

/// Fill rectangle - equivalent to display_fill_rect in AtomGL
fn display_fill_rect_nif(_env: &avmnif_rs::term::Context, args: &[Term]) -> NifResult<Term> {
    let x = args[0].try_into_i32().map_err(|_| NifError::BadArg)? as u16;
    let y = args[1].try_into_i32().map_err(|_| NifError::BadArg)? as u16;
    let width = args[2].try_into_i32().map_err(|_| NifError::BadArg)? as u16;
    let height = args[3].try_into_i32().map_err(|_| NifError::BadArg)? as u16;
    let r = args[4].try_into_i32().map_err(|_| NifError::BadArg)? as u8;
    let g = args[5].try_into_i32().map_err(|_| NifError::BadArg)? as u8;
    let b = args[6].try_into_i32().map_err(|_| NifError::BadArg)? as u8; // Fixed index
    
    let color = rgb888_to_rgb565(r, g, b);
    
    // Fill the rectangle in framebuffer
    if x >= LCD_WIDTH || y >= LCD_HEIGHT {
        return Ok(error());
    }

    let end_x = core::cmp::min(x + width, LCD_WIDTH);
    let end_y = core::cmp::min(y + height, LCD_HEIGHT);

    unsafe {
        for row in y..end_y {
            let start_idx = (row as usize * LCD_WIDTH as usize) + x as usize;
            let end_idx = (row as usize * LCD_WIDTH as usize) + end_x as usize;
            
            for idx in start_idx..end_idx {
                FRAMEBUFFER[idx] = color;
            }
        }
    }
    
    Ok(ok())
}

/// Set single pixel - equivalent to display_set_pixel in AtomGL
fn display_set_pixel_nif(_env: &avmnif_rs::term::Context, args: &[Term]) -> NifResult<Term> {
    let x = args[0].try_into_i32().map_err(|_| NifError::BadArg)? as u16;
    let y = args[1].try_into_i32().map_err(|_| NifError::BadArg)? as u16;
    let r = args[2].try_into_i32().map_err(|_| NifError::BadArg)? as u8;
    let g = args[3].try_into_i32().map_err(|_| NifError::BadArg)? as u8;
    let b = args[4].try_into_i32().map_err(|_| NifError::BadArg)? as u8; // Fixed to separate arg
    
    let color = rgb888_to_rgb565(r, g, b);
    
    if x < LCD_WIDTH && y < LCD_HEIGHT {
        unsafe {
            let idx = (y as usize * LCD_WIDTH as usize) + x as usize;
            FRAMEBUFFER[idx] = color;
        }
    }
    
    Ok(ok())
}

/// Get display info - equivalent to display_get_info in AtomGL
fn display_get_info_nif(env: &avmnif_rs::term::Context, _args: &[Term]) -> NifResult<Term> {
    let width_term = Term::from(LCD_WIDTH as i32);
    let height_term = Term::from(LCD_HEIGHT as i32);
    
    // Create atom for RGB565 format
    let format_atom_index = env.get_atom_table().insert_atom("rgb565", Default::default())
        .map_err(|_| NifError::BadArg)?;
    let format_term = Term::atom_from_index(format_atom_index);
    
    // Return {Width, Height, Format}
    let info_tuple = Term::make_tuple(env, &[width_term, height_term, format_term])
        .map_err(|_| NifError::BadArg)?;
    Ok(info_tuple)
}

/// Clear display - equivalent to display_clear in AtomGL  
fn display_clear_nif(_env: &avmnif_rs::term::Context, args: &[Term]) -> NifResult<Term> {
    let color = if args.is_empty() {
        0u16 // Black
    } else {
        // Parse color argument - could be RGB tuple or single color value
        if let Ok(single_color) = args[0].try_into_i32() {
            single_color as u16
        } else {
            // Try to parse as RGB tuple {R, G, B}
            // This would need proper tuple extraction from avmnif_rs
            0u16
        }
    };
    
    unsafe {
        for pixel in FRAMEBUFFER.iter_mut() {
            *pixel = color;
        }
    }
    
    Ok(ok())
}