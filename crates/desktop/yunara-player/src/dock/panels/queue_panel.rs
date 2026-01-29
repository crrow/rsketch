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

//! Play queue dock panel.
//!
//! Displays the current play queue and upcoming songs.

use gpui::{AnyView, Context, Entity, IntoElement, ParentElement, Render, Styled};

use crate::{app_state::AppState, dock::DockPanel};

/// Panel showing the current play queue.
pub struct QueuePanel {
    app_state: Entity<AppState>,
}

impl QueuePanel {
    /// Creates a new queue panel.
    pub fn new(app_state: Entity<AppState>) -> Self {
        Self { app_state }
    }
}

impl DockPanel for QueuePanel {
    fn title(&self) -> String {
        "Queue".to_string()
    }

    fn icon(&self) -> Option<&'static str> {
        Some("queue_music")
    }

    fn to_any_view(&self) -> AnyView {
        todo!("Implement to_any_view for QueuePanel")
    }
}

impl Render for QueuePanel {
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
            .gap_2()
            .p_2()
            .child(
                gpui::div()
                    .font_weight(gpui::FontWeight::BOLD)
                    .pb_2()
                    .border_b_1()
                    .border_color(gpui::rgb(0x333333))
                    .child("Play Queue"),
            )
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(gpui::div().child("1. Song A - Artist A"))
                    .child(gpui::div().child("2. Song B - Artist B"))
                    .child(gpui::div().child("3. Song C - Artist C")),
            )
    }
}
