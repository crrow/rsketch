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
//! Displays the main landing page with recently played, recommended content, etc.

use gpui::{AnyView, Context, Entity, EntityId, IntoElement, ParentElement, Render, Styled};

use crate::{app_state::AppState, pane::PaneItem};

/// Home view displaying recently played content and recommendations.
pub struct HomeView {
    app_state: Entity<AppState>,
}

impl HomeView {
    /// Creates a new home view.
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        Self { app_state }
    }
}

impl PaneItem for HomeView {
    fn entity_id(&self) -> EntityId {
        self.app_state.entity_id()
    }

    fn tab_title(&self) -> String {
        "Home".to_string()
    }

    fn to_any_view(&self) -> AnyView {
        // TODO: Properly convert this view to AnyView
        // For now, create a placeholder
        todo!("Implement to_any_view for HomeView")
    }

    fn can_close(&self) -> bool {
        false // Home view cannot be closed
    }
}

impl Render for HomeView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .gap_4()
            .p_4()
            .child(
                gpui::div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("Welcome to Yunara"),
            )
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(gpui::div().child("Recently Played"))
                    .child(gpui::div().child("• Placeholder for recent content")),
            )
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(gpui::div().child("Recommended"))
                    .child(gpui::div().child("• Placeholder for recommendations")),
            )
    }
}
