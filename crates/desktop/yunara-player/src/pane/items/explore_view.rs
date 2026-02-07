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

//! Explore view pane item.
//!
//! Displays discovery content, trending music, and recommendations.

use gpui::{AnyView, Context, EntityId, IntoElement, ParentElement, Render, Styled, WeakEntity};
use yunara_ui::components::theme::ThemeExt;

use crate::{app_state::AppState, pane::PaneItem};

/// Explore view for discovering new music.
pub struct ExploreView {
    weak_self: WeakEntity<Self>,
    _app_state: AppState,
}

impl ExploreView {
    /// Creates a new explore view.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            _app_state: app_state,
        }
    }
}

impl PaneItem for ExploreView {
    fn entity_id(&self) -> EntityId { self.weak_self.entity_id() }

    fn tab_title(&self) -> String { "Explore".to_string() }

    fn to_any_view(&self) -> AnyView {
        self.weak_self
            .upgrade()
            .map(AnyView::from)
            .expect("ExploreView should still be alive")
    }

    fn can_close(&self) -> bool { false }
}

impl Render for ExploreView {
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
                    .child("Explore"),
            )
            .child(
                gpui::div().text_color(theme.text_secondary).child(
                    "Discover new music, trending tracks, and personalized recommendations.",
                ),
            )
            .child(
                gpui::div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.text_muted)
                    .child("Content coming soon..."),
            )
    }
}
