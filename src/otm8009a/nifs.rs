//! OTM8009A NIF Functions
//! 
//! NIF implementations for AtomVM integration.
//! This entire module is only compiled when the nifs feature is enabled.

#[cfg(feature = "nifs")]
mod nif_impl {
    use avmnif_rs::{
        nif_collection,
        term::{Context, Term, TermValue, NifResult, NifError},
    };
    use crate::otm8009a::defs::{LCD_WIDTH, LCD_HEIGHT};

    // Register the NIF collection
    nif_collection!(
        otm8009a,
        init = otm8009a_nif_init,
        nifs = [
            ("init", 1, otm8009a_init),
            ("set_pixel", 4, otm8009a_set_pixel),
            ("fill_rect", 6, otm8009a_fill_rect),
            ("clear", 1, otm8009a_clear),
            ("get_info", 1, otm8009a_get_info),
            ("update", 1, otm8009a_update),
        ]
    );

    // Initialize the display
    fn otm8009a_init(ctx: &mut Context, args: &[usize]) -> NifResult<usize> {
        if args.len() != 1 {
            return Err(NifError::BadArity);
        }

        // Extract configuration term
        let config_term = Term::from_raw(args[0]);
        let config_value = config_term.to_value()?;

        // Parse configuration tuple: {width, height, orientation}
        let config_tuple = config_value.as_tuple().ok_or(NifError::BadArg)?;
        if config_tuple.len() != 3 {
            return Err(NifError::BadArg);
        }

        let width = config_tuple[0].as_int().ok_or(NifError::BadArg)?;
        let height = config_tuple[1].as_int().ok_or(NifError::BadArg)?;
        let orientation = config_tuple[2].as_int().ok_or(NifError::BadArg)?;

        // Validate parameters
        if width <= 0 || height <= 0 || width > 1024 || height > 1024 {
            return Err(NifError::BadArg);
        }

        if orientation < 0 || orientation > 3 {
            return Err(NifError::BadArg);
        }

        // TODO: Get driver from context and initialize
        // let mut driver = get_driver_from_context(ctx);
        // driver.init(RGB565, orientation as u32)?;
        
        // TODO: Use proper atom creation API from avmnif-rs
        Ok(Term::from_raw(0).raw())  // Placeholder
    }

    // Set a single pixel
    fn otm8009a_set_pixel(ctx: &mut Context, args: &[usize]) -> NifResult<usize> {
        if args.len() != 4 {
            return Err(NifError::BadArity);
        }

        // Extract arguments: X, Y, Color, Handle
        let x_term = Term::from_raw(args[0]);
        let y_term = Term::from_raw(args[1]);
        let color_term = Term::from_raw(args[2]);
        let _handle_term = Term::from_raw(args[3]);

        let x = x_term.to_value()?.as_int().ok_or(NifError::BadArg)?;
        let y = y_term.to_value()?.as_int().ok_or(NifError::BadArg)?;
        let _color = color_term.to_value()?.as_int().ok_or(NifError::BadArg)?;

        // Validate coordinates
        if x < 0 || y < 0 || x >= LCD_WIDTH as i32 || y >= LCD_HEIGHT as i32 {
            return Err(NifError::BadArg);
        }

        // TODO: Get driver from context and call set_pixel
        // let mut driver = get_driver_from_context(ctx);
        // driver.set_pixel(x as u16, y as u16, color as u16)?;

        Ok(Term::from_raw(0).raw())  // Placeholder
    }

    // Fill a rectangle
    fn otm8009a_fill_rect(ctx: &mut Context, args: &[usize]) -> NifResult<usize> {
        if args.len() != 6 {
            return Err(NifError::BadArity);
        }

        // Extract arguments: X, Y, Width, Height, Color, Handle
        let x_term = Term::from_raw(args[0]);
        let y_term = Term::from_raw(args[1]);
        let width_term = Term::from_raw(args[2]);
        let height_term = Term::from_raw(args[3]);
        let color_term = Term::from_raw(args[4]);
        let _handle_term = Term::from_raw(args[5]);

        let x = x_term.to_value()?.as_int().ok_or(NifError::BadArg)?;
        let y = y_term.to_value()?.as_int().ok_or(NifError::BadArg)?;
        let width = width_term.to_value()?.as_int().ok_or(NifError::BadArg)?;
        let height = height_term.to_value()?.as_int().ok_or(NifError::BadArg)?;
        let _color = color_term.to_value()?.as_int().ok_or(NifError::BadArg)?;

        // Validate parameters
        if x < 0 || y < 0 || width <= 0 || height <= 0 {
            return Err(NifError::BadArg);
        }
        
        if x + width > LCD_WIDTH as i32 || y + height > LCD_HEIGHT as i32 {
            return Err(NifError::BadArg);
        }

        // TODO: Get driver from context and call fill_rect
        // let mut driver = get_driver_from_context(ctx);
        // driver.fill_rect(x as u16, y as u16, width as u16, height as u16, color as u16)?;

        Ok(Term::from_raw(0).raw())  // Placeholder
    }

    // Clear the display
    fn otm8009a_clear(_ctx: &mut Context, args: &[usize]) -> NifResult<usize> {
        if args.len() != 1 {
            return Err(NifError::BadArity);
        }

        let _handle_term = Term::from_raw(args[0]);

        // TODO: Get driver from context and call clear
        // let mut driver = get_driver_from_context(ctx);
        // driver.clear(0x0000)?; // Clear to black

        Ok(Term::from_raw(0).raw())  // Placeholder
    }

    // Get display information
    fn otm8009a_get_info(_ctx: &mut Context, args: &[usize]) -> NifResult<usize> {
        if args.len() != 1 {
            return Err(NifError::BadArity);
        }

        let _handle_term = Term::from_raw(args[0]);

        // TODO: Get driver from context and return dimensions
        // let driver = get_driver_from_context(ctx);
        // let (width, height) = driver.get_dimensions();
        // Create tuple {ok, {width, height, pixel_format}}
        
        Ok(Term::from_raw(0).raw())  // Placeholder
    }

    // Update/refresh the display
    fn otm8009a_update(_ctx: &mut Context, args: &[usize]) -> NifResult<usize> {
        if args.len() != 1 {
            return Err(NifError::BadArity);
        }

        let _handle_term = Term::from_raw(args[0]);

        // TODO: Trigger display update
        // This might involve LTDC refresh or similar

        Ok(Term::from_raw(0).raw())  // Placeholder
    }

    // Helper function to extract RGB color from term
    #[allow(dead_code)]
    fn extract_rgb_color(term: Term) -> NifResult<u32> {
        let value = term.to_value()?;
        
        // Handle different color formats:
        // - Integer: direct RGB value
        // - Tuple: {R, G, B} format
        match value {
            TermValue::SmallInt(color) => {
                if color < 0 {
                    Err(NifError::BadArg)
                } else {
                    Ok(color as u32)
                }
            }
            TermValue::Tuple(elements) if elements.len() == 3 => {
                let r = elements[0].as_int().ok_or(NifError::BadArg)?;
                let g = elements[1].as_int().ok_or(NifError::BadArg)?;
                let b = elements[2].as_int().ok_or(NifError::BadArg)?;
                
                // Validate RGB values
                if r < 0 || r > 255 || g < 0 || g > 255 || b < 0 || b > 255 {
                    return Err(NifError::BadArg);
                }
                
                // Convert to RGB565 format (assuming 16-bit color)
                let rgb565 = ((r as u32 & 0xF8) << 8) | 
                            ((g as u32 & 0xFC) << 3) | 
                            ((b as u32 & 0xF8) >> 3);
                Ok(rgb565)
            }
            _ => Err(NifError::BadArg)
        }
    }
}