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

/// Theme provider for global theme access throughout the application.
///
/// This module implements the GPUI Global pattern to provide theme
/// configuration to all UI components. Components access the current theme via:
/// ```ignore
/// let theme = cx.global::<ThemeProvider>().theme();
/// ```
use gpui::{App, Global};

use super::theme_config::ThemeConfig;

/// Global theme provider that stores the current theme configuration.
///
/// Registered as a GPUI Global, allowing any component to access the current
/// theme without explicit prop drilling. Supports runtime theme switching.
pub struct ThemeProvider {
    theme: ThemeConfig,
}

impl Global for ThemeProvider {}

impl ThemeProvider {
    /// Creates a new ThemeProvider with the specified theme configuration.
    pub fn new(theme: ThemeConfig) -> Self { Self { theme } }

    /// Returns a reference to the current theme configuration.
    pub fn theme(&self) -> &ThemeConfig { &self.theme }

    /// Updates the current theme configuration.
    pub fn set_theme(&mut self, theme: ThemeConfig) { self.theme = theme; }

    /// Initializes the theme provider as a GPUI global with default theme.
    ///
    /// Call this during application startup to make the theme available
    /// to all components.
    pub fn init(cx: &mut App) { cx.set_global(Self::new(ThemeConfig::default())); }

    /// Initializes the theme provider with a specific theme configuration.
    pub fn init_with_theme(cx: &mut App, theme: ThemeConfig) { cx.set_global(Self::new(theme)); }
}

/// Extension trait for convenient theme access from any context.
pub trait ThemeExt {
    /// Returns a reference to the current theme configuration.
    fn theme(&self) -> &ThemeConfig;
}

impl ThemeExt for App {
    fn theme(&self) -> &ThemeConfig { self.global::<ThemeProvider>().theme() }
}
