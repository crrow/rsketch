// Copyright 2025 Crrow
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
