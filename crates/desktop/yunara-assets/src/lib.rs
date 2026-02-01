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

use anyhow::Context as _;
use gpui::{App, AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

/// Icon asset paths for use with `gpui::svg().path()` or `gpui::img().path()`.
pub mod icons {
    /// Logo with "Music" text for dark theme
    pub const LOGO_DARK: &str = "icons/on_platform_logo_dark.svg";

    // Navigation icons (SVG)
    pub const HOME: &str = "icons/home.svg";
    pub const EXPLORE: &str = "icons/explore.svg";
    pub const LIBRARY: &str = "icons/library.svg";

    // Navigation icons - filled versions for active state
    pub const HOME_FILLED: &str = "icons/home-filled.svg";
    pub const EXPLORE_FILLED: &str = "icons/explore-filled.svg";
    pub const LIBRARY_FILLED: &str = "icons/library-filled.svg";

    // Header icons
    pub const MENU: &str = "icons/menu.svg";

    // Volume control icons (SVG)
    pub const VOLUME: &str = "icons/volume.svg";
    pub const VOLUME_MUTED: &str = "icons/volume-muted.svg";

    // Playback control icons (SVG)
    pub const MEDIA_PREVIOUS: &str = "icons/media-previous.svg";
    pub const MEDIA_PLAY: &str = "icons/media-play.svg";
    pub const MEDIA_PAUSE: &str = "icons/media-pause.svg";
    pub const MEDIA_NEXT: &str = "icons/media-next.svg";
}

#[derive(RustEmbed)]
#[folder = "../assets"]
#[include = "fonts/**/*"]
#[include = "icons/**/*"]
#[include = "images/**/*"]
#[include = "themes/**/*"]
#[include = "*.md"]
#[exclude = "*.DS_Store"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Self::get(path)
            .map(|f| Some(f.data))
            .with_context(|| format!("loading asset at path {path:?}"))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| {
                if p.starts_with(path) {
                    Some(p.into())
                } else {
                    None
                }
            })
            .collect())
    }
}

impl Assets {
    /// Populate the [`TextSystem`] of the given [`AppContext`] with all `.ttf`
    /// fonts in the `fonts` directory.
    pub fn load_fonts(&self, cx: &App) -> anyhow::Result<()> {
        let font_paths = self.list("fonts")?;
        let mut embedded_fonts = Vec::new();
        for font_path in font_paths {
            if font_path.ends_with(".ttf") {
                let font_bytes = cx
                    .asset_source()
                    .load(&font_path)?
                    .expect("Assets should never return None");
                embedded_fonts.push(font_bytes);
            }
        }

        cx.text_system().add_fonts(embedded_fonts)
    }

    pub fn load_test_fonts(&self, cx: &App) {
        cx.text_system()
            .add_fonts(vec![
                self.load("fonts/lilex/Lilex-Regular.ttf").unwrap().unwrap(),
            ])
            .unwrap()
    }
}
