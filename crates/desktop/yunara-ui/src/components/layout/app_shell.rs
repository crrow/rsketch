/// Application shell component that assembles the complete UI.
///
/// Combines the sidebar, content area, queue panel, and player bar
/// into the main application layout.

use gpui::{
    div, prelude::*, AnyElement, App, ElementId, IntoElement, ParentElement, Styled, Window,
};

use crate::components::theme::ThemeExt;

use super::{ContentArea, QueuePanel, Sidebar};
use crate::components::player::PlayerBar;
use crate::models::{PlaylistSummary, Route, Track};

/// Main application shell that provides the overall layout structure.
///
/// Layout:
/// ```text
/// ┌─────────────────────────────────────────────────────────────────┐
/// │  Header (64px) - optional                                       │
/// ├─────────┬───────────────────────────────────────┬───────────────┤
/// │ Sidebar │         Content Area                  │  Queue Panel  │
/// │ (240px) │         (flex: 1)                     │   (320px)     │
/// │         │                                       │  (conditional)│
/// ├─────────┴───────────────────────────────────────┴───────────────┤
/// │  PlayerBar (80px)                                               │
/// └─────────────────────────────────────────────────────────────────┘
/// ```
#[derive(IntoElement)]
pub struct AppShell {
    id: ElementId,
    playlists: Vec<PlaylistSummary>,
    current_route: Option<Route>,
    show_queue_panel: bool,
    queue: Vec<Track>,
    current_queue_index: Option<usize>,
    content: Vec<AnyElement>,
    // Player bar state
    track_title: Option<String>,
    artist_name: Option<String>,
    is_playing: bool,
    has_previous: bool,
    has_next: bool,
    current_time_secs: u64,
    total_duration_secs: u64,
    volume: f32,
    is_muted: bool,
}

impl AppShell {
    /// Creates a new app shell.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            playlists: Vec::new(),
            current_route: None,
            show_queue_panel: false,
            queue: Vec::new(),
            current_queue_index: None,
            content: Vec::new(),
            track_title: None,
            artist_name: None,
            is_playing: false,
            has_previous: false,
            has_next: false,
            current_time_secs: 0,
            total_duration_secs: 0,
            volume: 0.5,
            is_muted: false,
        }
    }

    /// Sets the playlists for the sidebar.
    pub fn playlists(mut self, playlists: Vec<PlaylistSummary>) -> Self {
        self.playlists = playlists;
        self
    }

    /// Sets the current route for navigation highlighting.
    pub fn current_route(mut self, route: Route) -> Self {
        self.current_route = Some(route);
        self
    }

    /// Sets whether to show the queue panel.
    pub fn show_queue_panel(mut self, show: bool) -> Self {
        self.show_queue_panel = show;
        self
    }

    /// Sets the playback queue.
    pub fn queue(mut self, queue: Vec<Track>, current_index: Option<usize>) -> Self {
        self.queue = queue;
        self.current_queue_index = current_index;
        self
    }

    /// Adds content to the main content area.
    pub fn content(mut self, element: impl IntoElement) -> Self {
        self.content.push(element.into_any_element());
        self
    }

    /// Sets the now playing info for the player bar.
    pub fn now_playing(mut self, title: impl Into<String>, artist: impl Into<String>) -> Self {
        self.track_title = Some(title.into());
        self.artist_name = Some(artist.into());
        self
    }

    /// Sets the playback state.
    pub fn playback_state(mut self, is_playing: bool, has_previous: bool, has_next: bool) -> Self {
        self.is_playing = is_playing;
        self.has_previous = has_previous;
        self.has_next = has_next;
        self
    }

    /// Sets the progress state.
    pub fn progress(mut self, current_secs: u64, total_secs: u64) -> Self {
        self.current_time_secs = current_secs;
        self.total_duration_secs = total_secs;
        self
    }

    /// Sets the volume state.
    pub fn volume_state(mut self, volume: f32, is_muted: bool) -> Self {
        self.volume = volume;
        self.is_muted = is_muted;
        self
    }
}

impl RenderOnce for AppShell {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let id = self.id;
        let current_route = self.current_route.clone();

        // Build the player bar
        let mut player_bar = PlayerBar::new("player-bar")
            .playing(self.is_playing)
            .navigation(self.has_previous, self.has_next)
            .progress(
                std::time::Duration::from_secs(self.current_time_secs),
                std::time::Duration::from_secs(self.total_duration_secs),
            )
            .volume(self.volume, self.is_muted);

        if let (Some(title), Some(artist)) = (self.track_title, self.artist_name) {
            player_bar = player_bar.now_playing(title, artist);
        }

        div()
            .id(id)
            .size_full()
            .bg(theme.background_primary)
            .flex()
            .flex_col()
            // Main content area (sidebar + content + queue)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .overflow_hidden()
                    // Sidebar
                    .child({
                        let mut sidebar = Sidebar::new("sidebar").playlists(self.playlists);
                        if let Some(route) = current_route {
                            sidebar = sidebar.current_route(route);
                        }
                        sidebar
                    })
                    // Content area
                    .child({
                        let mut content_area = ContentArea::new("content");
                        for child in self.content {
                            content_area = content_area.child(child);
                        }
                        content_area
                    })
                    // Queue panel (conditional)
                    .when(self.show_queue_panel, |el| {
                        el.child(
                            QueuePanel::new("queue-panel")
                                .queue(self.queue)
                                .current_index(self.current_queue_index),
                        )
                    }),
            )
            // Player bar at bottom
            .child(player_bar)
    }
}
