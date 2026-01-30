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

//! Home view pane item.
//!
//! Displays the main landing page with recently played, recommended content,
//! etc.

use gpui::{
    AnyView, Context, EntityId, IntoElement, ParentElement, Render, Styled, WeakEntity, px,
};
use yunara_ui::components::theme::ThemeExt;

use crate::{app_state::AppState, pane::PaneItem};

/// Home view displaying recently played content and recommendations.
pub struct HomeView {
    weak_self: WeakEntity<Self>,
    app_state: AppState,
}

impl HomeView {
    /// Creates a new home view.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            app_state,
        }
    }
}

impl PaneItem for HomeView {
    fn entity_id(&self) -> EntityId { self.weak_self.entity_id() }

    fn tab_title(&self) -> String { "Home".to_string() }

    fn to_any_view(&self) -> AnyView {
        self.weak_self
            .upgrade()
            .map(AnyView::from)
            .expect("HomeView view should still be alive")
    }

    fn can_close(&self) -> bool {
        false // Home view cannot be closed
    }
}

impl Render for HomeView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // Song/Video toggle tab
        let toggle_tab = |label: &'static str, active: bool| {
            let base = gpui::div()
                .px(px(16.0))
                .py(px(8.0))
                .rounded(px(20.0))
                .cursor_pointer()
                .text_sm();
            let styled = if active {
                base.bg(theme.text_primary)
                    .text_color(theme.background_primary)
            } else {
                base.bg(theme.background_elevated)
                    .text_color(theme.text_secondary)
            };
            styled.child(label)
        };

        gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .items_center()
            .p_4()
            .gap_4()
            // Song/Video toggle - centered at top
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(toggle_tab("Song", true))
                    .child(toggle_tab("Video", false)),
            )
            // Main album artwork area - square, centered, large
            .child(
                gpui::div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .child(
                        gpui::div()
                            // Make it a large square, responsive to container
                            .w(px(380.0))
                            .h(px(380.0))
                            .rounded(px(8.0))
                            .bg(theme.background_elevated)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.text_muted)
                            .child("Album artwork"),
                    ),
            )
            // Song info below artwork
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .pb(px(16.0))
                    .child(
                        gpui::div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child("逆転劇 - Gyakutengeki"),
                    )
                    .child(
                        gpui::div()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .child("Tsukuyomi • Gyakutengeki • 2023"),
                    ),
            )
    }
}
