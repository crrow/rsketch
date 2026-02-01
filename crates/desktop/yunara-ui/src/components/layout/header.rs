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

/// Application header component.
///
/// Contains the top navigation bar with hamburger menu, logo, search bar,
/// and navigation controls.
use gpui::{
    App, ElementId, IntoElement, ParentElement, Styled, Window, div, img, prelude::*, px, svg,
};

use crate::components::theme::ThemeExt;

/// Header bar for the top of the application.
///
/// Layout:
/// ```text
/// ┌──────────────────────────────────────────────────────────────────────────┐
/// │  ≡  [Logo]     [━━━━━━━━ Search ━━━━━━━━]              ←  →  [Profile]   │
/// └──────────────────────────────────────────────────────────────────────────┘
/// ```
#[derive(IntoElement)]
pub struct Header {
    id:                 ElementId,
    logo_path:          Option<&'static str>,
    search_placeholder: &'static str,
    sidebar_width:      Option<f32>,
}

impl Header {
    /// Creates a new header.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id:                 id.into(),
            logo_path:          None,
            search_placeholder: "Search songs, albums, artists, podcasts",
            sidebar_width:      None,
        }
    }

    /// Sets the logo image path.
    pub fn logo(mut self, path: &'static str) -> Self {
        self.logo_path = Some(path);
        self
    }

    /// Sets the search placeholder text.
    pub fn search_placeholder(mut self, text: &'static str) -> Self {
        self.search_placeholder = text;
        self
    }

    /// Sets the sidebar width so the search bar aligns with the center content
    /// area.
    pub fn sidebar_width(mut self, width: f32) -> Self {
        self.sidebar_width = Some(width);
        self
    }
}

impl RenderOnce for Header {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let left_width = self.sidebar_width.unwrap_or(0.0);

        div()
            .id(self.id)
            .h(px(56.0))
            .px(px(16.0))
            .flex()
            .items_center()
            .gap_4()
            .bg(theme.background_primary)
            // Left region (matches sidebar width)
            .child(
                div()
                    .w(px(left_width))
                    .flex()
                    .items_center()
                    .gap_4()
                    // Hamburger menu
                    .child(
                        div()
                            .w(px(24.0))
                            .h(px(24.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .child(
                                svg()
                                    .path(yunara_assets::icons::MENU)
                                    .size(px(22.0))
                                    .text_color(theme.text_primary),
                            ),
                    )
                    // Logo
                    .when_some(self.logo_path, |el, path| {
                        el.child(img(path).w(px(77.0)).h(px(26.0)))
                    }),
            )
            // Search bar
            .child(
                div().flex_1().flex().justify_start().child(
                    div()
                        .w(px(420.0))
                        .h(px(30.0))
                        .rounded(px(10.0))
                        .bg(theme.background_elevated)
                        .text_color(theme.text_muted)
                        .px(px(12.0))
                        .flex()
                        .items_center()
                        .child(self.search_placeholder),
                ),
            )
            // Navigation controls
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .text_color(theme.text_secondary)
                    .child("←")
                    .child("→")
                    .child("🙂"),
            )
    }
}
