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

//! Playlist view pane item.
//!
//! Displays the contents of a specific playlist with songs, metadata, etc.

use gpui::{
    AnyView,
    Context,
    Entity,
    EntityId,
    IntoElement,
    ParentElement,
    Render,
    Styled,
    WeakEntity,
};

use crate::{app_state::AppState, pane::PaneItem};

/// View displaying a specific playlist's contents.
pub struct PlaylistView {
    weak_self: WeakEntity<Self>,
    app_state: Entity<AppState>,
    playlist_id: String,
    playlist_name: String,
}

impl PlaylistView {
    /// Creates a new playlist view.
    pub fn new(
        app_state: Entity<AppState>,
        playlist_id: String,
        playlist_name: String,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            app_state,
            playlist_id,
            playlist_name,
        }
    }
}

impl PaneItem for PlaylistView {
    fn entity_id(&self) -> EntityId {
        self.weak_self.entity_id()
    }

    fn tab_title(&self) -> String {
        self.playlist_name.clone()
    }

    fn to_any_view(&self) -> AnyView {
        self.weak_self
            .upgrade()
            .map(AnyView::from)
            .expect("PlaylistView should still be alive")
    }

    fn can_close(&self) -> bool {
        true
    }
}

impl Render for PlaylistView {
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
                    .child(format!("Playlist: {}", self.playlist_name)),
            )
            .child(
                gpui::div()
                    .text_sm()
                    .text_color(gpui::rgb(0x808080))
                    .child(format!("ID: {}", self.playlist_id)),
            )
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(gpui::div().child("Songs:"))
                    .child(gpui::div().child("1. Song A - Artist A - 3:45"))
                    .child(gpui::div().child("2. Song B - Artist B - 4:12"))
                    .child(gpui::div().child("3. Song C - Artist C - 2:58")),
            )
    }
}
