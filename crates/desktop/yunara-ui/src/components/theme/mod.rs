/// Theme system for the Yunara music player.
///
/// Provides customizable color schemes through `ThemeConfig` and global
/// theme access via `ThemeProvider`. Supports preset themes (YTMusic Dark,
/// OLED Black, Light) and runtime theme switching.

mod theme_config;
mod theme_provider;

pub use theme_config::ThemeConfig;
pub use theme_provider::{ThemeExt, ThemeProvider};
