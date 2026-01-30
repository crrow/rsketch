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

//! Library dock panel.
//!
//! Shows playlists, artists, albums, etc.

use gpui::{AnyView, Context, IntoElement, ParentElement, Render, Styled, WeakEntity};

use crate::{app_state::AppState, dock::DockPanel};

/// Panel showing the music library (playlists, artists, albums).
pub struct LibraryPanel {
    weak_self: WeakEntity<Self>,
    app_state: AppState,
}

impl LibraryPanel {
    /// Creates a new library panel.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            app_state,
        }
    }
}

impl DockPanel for LibraryPanel {
    fn title(&self) -> String { "Library".to_string() }

    fn icon(&self) -> Option<&'static str> { Some("library_music") }

    fn to_any_view(&self) -> AnyView {
        self.weak_self
            .upgrade()
            .map(AnyView::from)
            .expect("LibraryPanel view should still be alive")
    }
}

impl Render for LibraryPanel {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        use yunara_ui::components::theme::ThemeExt;
        let theme = cx.theme();

        // Navigation item helper
        let nav_item = |icon: &'static str, label: &'static str, active: bool| {
            let base = gpui::div()
                .flex()
                .items_center()
                .gap_3()
                .px(gpui::px(12.0))
                .py(gpui::px(10.0))
                .rounded(gpui::px(8.0))
                .cursor_pointer();
            let styled = if active { base.bg(theme.active) } else { base };
            styled
                .child(
                    gpui::div()
                        .text_color(if active {
                            theme.text_primary
                        } else {
                            theme.text_secondary
                        })
                        .child(icon),
                )
                .child(
                    gpui::div()
                        .text_color(if active {
                            theme.text_primary
                        } else {
                            theme.text_secondary
                        })
                        .child(label),
                )
        };

        // Playlist item helper with thumbnail placeholder
        let playlist_item = |label: &'static str| {
            gpui::div()
                .flex()
                .items_center()
                .gap_3()
                .px(gpui::px(12.0))
                .py(gpui::px(8.0))
                .rounded(gpui::px(6.0))
                .cursor_pointer()
                .child(
                    gpui::div()
                        .w(gpui::px(40.0))
                        .h(gpui::px(40.0))
                        .rounded(gpui::px(4.0))
                        .bg(theme.background_elevated)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.text_muted)
                        .text_xs()
                        .child("♪"),
                )
                .child(
                    gpui::div()
                        .flex()
                        .flex_col()
                        .child(
                            gpui::div()
                                .text_sm()
                                .text_color(theme.text_primary)
                                .child(label),
                        )
                        .child(
                            gpui::div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child("Auto playlist"),
                        ),
                )
        };

        gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .overflow_hidden()
            // Top navigation
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .py(gpui::px(8.0))
                    .child(nav_item("🏠", "Home", false))
                    .child(nav_item("🔍", "Explore", false))
                    .child(nav_item("📚", "Library", true)),
            )
            // New playlist button
            .child(
                gpui::div()
                    .px(gpui::px(12.0))
                    .py(gpui::px(12.0))
                    .child(
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px(gpui::px(16.0))
                            .py(gpui::px(10.0))
                            .rounded(gpui::px(20.0))
                            .border_1()
                            .border_color(theme.border)
                            .cursor_pointer()
                            .text_color(theme.text_primary)
                            .text_sm()
                            .child("+")
                            .child("New playlist"),
                    ),
            )
            // Playlists section
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(playlist_item("Liked Music"))
                    .child(playlist_item("Japanese"))
                    .child(playlist_item("BGM"))
                    .child(playlist_item("english"))
                    .child(playlist_item("KPop")),
            )
    }
}
