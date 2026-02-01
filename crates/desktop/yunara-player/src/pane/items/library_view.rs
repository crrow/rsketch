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

//! Library view pane item.
//!
//! Displays the user's music library including songs, albums, artists, and playlists.

use gpui::{
    px, AnyView, Context, EntityId, IntoElement, ParentElement, Render, Styled, WeakEntity,
};
use yunara_ui::components::theme::ThemeExt;

use crate::{app_state::AppState, pane::PaneItem};

/// Library view for browsing user's music collection.
pub struct LibraryView {
    weak_self: WeakEntity<Self>,
    app_state: AppState,
}

impl LibraryView {
    /// Creates a new library view.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            app_state,
        }
    }
}

impl PaneItem for LibraryView {
    fn entity_id(&self) -> EntityId {
        self.weak_self.entity_id()
    }

    fn tab_title(&self) -> String {
        "Library".to_string()
    }

    fn to_any_view(&self) -> AnyView {
        self.weak_self
            .upgrade()
            .map(AnyView::from)
            .expect("LibraryView should still be alive")
    }

    fn can_close(&self) -> bool {
        false
    }
}

impl Render for LibraryView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p_4()
            .gap_4()
            .child(
                gpui::div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .child("Your Library"),
            )
            .child(
                gpui::div()
                    .flex()
                    .gap_4()
                    .child(
                        gpui::div()
                            .px(px(16.0))
                            .py(px(8.0))
                            .rounded(px(20.0))
                            .bg(theme.text_primary)
                            .text_color(theme.background_primary)
                            .text_sm()
                            .cursor_pointer()
                            .child("Playlists"),
                    )
                    .child(
                        gpui::div()
                            .px(px(16.0))
                            .py(px(8.0))
                            .rounded(px(20.0))
                            .bg(theme.background_elevated)
                            .text_color(theme.text_secondary)
                            .text_sm()
                            .cursor_pointer()
                            .child("Albums"),
                    )
                    .child(
                        gpui::div()
                            .px(px(16.0))
                            .py(px(8.0))
                            .rounded(px(20.0))
                            .bg(theme.background_elevated)
                            .text_color(theme.text_secondary)
                            .text_sm()
                            .cursor_pointer()
                            .child("Artists"),
                    )
                    .child(
                        gpui::div()
                            .px(px(16.0))
                            .py(px(8.0))
                            .rounded(px(20.0))
                            .bg(theme.background_elevated)
                            .text_color(theme.text_secondary)
                            .text_sm()
                            .cursor_pointer()
                            .child("Songs"),
                    ),
            )
            .child(
                gpui::div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.text_muted)
                    .child("Library content coming soon..."),
            )
    }
}
