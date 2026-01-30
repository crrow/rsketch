/// Color utility functions for converting hex values to GPUI's Rgba format.
/// These utilities make it easier to define colors using familiar hex notation
/// (e.g., 0x121212 for dark backgrounds) rather than separate RGB float values.

use gpui::Rgba;

/// Converts a hex color value to an Rgba with full opacity.
///
/// # Arguments
/// * `hex` - A 24-bit hex color value (e.g., 0xFF0000 for red)
///
/// # Example
/// ```ignore
/// let red = rgba_from_hex(0xFF0000);
/// let dark_bg = rgba_from_hex(0x121212);
/// ```
pub fn rgba_from_hex(hex: u32) -> Rgba {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    Rgba { r, g, b, a: 1.0 }
}

/// Converts a hex color value to an Rgba with custom alpha.
///
/// # Arguments
/// * `hex` - A 24-bit hex color value (e.g., 0xFF0000 for red)
/// * `alpha` - Opacity value from 0.0 (transparent) to 1.0 (opaque)
///
/// # Example
/// ```ignore
/// let semi_transparent_white = rgba_from_hex_alpha(0xFFFFFF, 0.5);
/// ```
pub fn rgba_from_hex_alpha(hex: u32, alpha: f32) -> Rgba {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    Rgba { r, g, b, a: alpha }
}
