/// Theme configuration for the Yunara music player UI.
///
/// This module defines the color scheme structure that controls the visual
/// appearance of all UI components. The theme system supports both preset
/// themes (YTMusic Dark, OLED Black, Light) and fully customizable user themes.

use gpui::Rgba;

use crate::styles::color::rgba_from_hex;

/// Complete theme configuration containing all color values used throughout the UI.
///
/// Colors are organized into categories:
/// - Background colors: Primary, secondary, and elevated surfaces
/// - Text colors: Primary, secondary, and muted text
/// - Accent colors: Brand color and hover states
/// - Interactive colors: Hover, active, and border states
/// - Player-specific: Progress bar track and fill colors
#[derive(Debug, Clone)]
pub struct ThemeConfig {
    /// Main application background (e.g., #121212 for dark mode)
    pub background_primary: Rgba,
    /// Sidebar and panel backgrounds (e.g., #1d1d1d)
    pub background_secondary: Rgba,
    /// Floating/elevated element backgrounds (e.g., #282828)
    pub background_elevated: Rgba,

    /// Primary text color (e.g., #ffffff)
    pub text_primary: Rgba,
    /// Secondary/subdued text color (e.g., #a0a0a0)
    pub text_secondary: Rgba,
    /// Muted/disabled text color (e.g., #6a6a6a)
    pub text_muted: Rgba,

    /// Primary accent color (e.g., #ff0000 for YTMusic red)
    pub accent: Rgba,
    /// Accent color on hover state
    pub accent_hover: Rgba,

    /// Background color on hover
    pub hover: Rgba,
    /// Background color when active/selected
    pub active: Rgba,
    /// Border color for separators and outlines
    pub border: Rgba,

    /// Progress bar track (unfilled portion) color
    pub progress_track: Rgba,
    /// Progress bar fill color (typically matches accent)
    pub progress_fill: Rgba,
}

impl ThemeConfig {
    /// Creates the default YTMusic Dark theme.
    ///
    /// This is the standard dark theme inspired by YouTube Music's official
    /// dark mode, using #121212 as the primary background and #ff0000 as accent.
    pub fn ytmusic_dark() -> Self {
        Self {
            background_primary: rgba_from_hex(0x121212),
            background_secondary: rgba_from_hex(0x1d1d1d),
            background_elevated: rgba_from_hex(0x282828),

            text_primary: rgba_from_hex(0xffffff),
            text_secondary: rgba_from_hex(0xa0a0a0),
            text_muted: rgba_from_hex(0x6a6a6a),

            accent: rgba_from_hex(0xff0000),
            accent_hover: rgba_from_hex(0xcc0000),

            hover: rgba_from_hex(0x2a2a2a),
            active: rgba_from_hex(0x3a3a3a),
            border: rgba_from_hex(0x3a3a3a),

            progress_track: rgba_from_hex(0x4a4a4a),
            progress_fill: rgba_from_hex(0xff0000),
        }
    }

    /// Creates the OLED Black theme.
    ///
    /// Optimized for OLED displays with pure black backgrounds (#000000)
    /// to save power and provide maximum contrast.
    pub fn oled_black() -> Self {
        Self {
            background_primary: rgba_from_hex(0x000000),
            background_secondary: rgba_from_hex(0x0a0a0a),
            background_elevated: rgba_from_hex(0x1a1a1a),

            text_primary: rgba_from_hex(0xffffff),
            text_secondary: rgba_from_hex(0xa0a0a0),
            text_muted: rgba_from_hex(0x6a6a6a),

            accent: rgba_from_hex(0xff0000),
            accent_hover: rgba_from_hex(0xcc0000),

            hover: rgba_from_hex(0x1a1a1a),
            active: rgba_from_hex(0x2a2a2a),
            border: rgba_from_hex(0x2a2a2a),

            progress_track: rgba_from_hex(0x3a3a3a),
            progress_fill: rgba_from_hex(0xff0000),
        }
    }

    /// Creates the Light theme.
    ///
    /// A light mode option with white background for users who prefer
    /// lighter interfaces or better visibility in bright environments.
    pub fn light() -> Self {
        Self {
            background_primary: rgba_from_hex(0xffffff),
            background_secondary: rgba_from_hex(0xf5f5f5),
            background_elevated: rgba_from_hex(0xffffff),

            text_primary: rgba_from_hex(0x030303),
            text_secondary: rgba_from_hex(0x606060),
            text_muted: rgba_from_hex(0x909090),

            accent: rgba_from_hex(0xff0000),
            accent_hover: rgba_from_hex(0xcc0000),

            hover: rgba_from_hex(0xeeeeee),
            active: rgba_from_hex(0xe0e0e0),
            border: rgba_from_hex(0xe0e0e0),

            progress_track: rgba_from_hex(0xd0d0d0),
            progress_fill: rgba_from_hex(0xff0000),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::ytmusic_dark()
    }
}
