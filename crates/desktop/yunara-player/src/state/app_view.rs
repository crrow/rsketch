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

use gpui::{Entity, IntoElement, Render};
use yunara_ui::{
    components::layout::AppShell,
    models::{PlaylistSummary, Route},
};

use crate::app_state::AppState;

/// Main application view that renders the UI shell.
///
/// This view holds a reference to the application state and is responsible
/// for rendering the main layout including sidebar, content area, queue panel,
/// and player bar.
///
/// The view reactively updates when the underlying state changes through
/// GPUI's Entity system.
pub struct AppView {
    /// Reference to the global application state
    app_state: Entity<AppState>,

    /// Currently active navigation route
    current_route: Route,
}

impl AppView {
    /// Creates a new AppView with the given application state.
    ///
    /// # Arguments
    /// * `app_state` - Entity containing the application state
    pub fn new(app_state: Entity<AppState>) -> Self {
        Self {
            app_state,
            current_route: Route::Home,
        }
    }

    /// Gets the current playlists from the application state.
    ///
    /// TODO: Replace with actual playlist fetching from database
    fn get_playlists(&self, cx: &gpui::Context<Self>) -> Vec<PlaylistSummary> {
        // For now, return sample data
        // In the future, this should query from app_state.db()
        let _db = self.app_state.read(cx).db();

        vec![
            PlaylistSummary::new("1", "Liked Music", "You", 342),
            PlaylistSummary::new("2", "My Playlist #1", "You", 28),
            PlaylistSummary::new("3", "Workout Mix", "You", 65),
            PlaylistSummary::new("4", "Chill Vibes", "You", 112),
        ]
    }
}

impl Render for AppView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let playlists = self.get_playlists(cx);

        AppShell::new("app-shell")
            .playlists(playlists)
            .current_route(self.current_route.clone())
            .playback_state(false, false, false)
            .progress(0, 0)
            .volume_state(0.5, false)
    }
}
