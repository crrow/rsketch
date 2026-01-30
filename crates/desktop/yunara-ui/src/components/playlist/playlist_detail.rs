/// Playlist detail view component.
///
/// Displays full playlist information including cover mosaic, metadata,
/// track list, and action buttons.

use gpui::{
    div, prelude::*, px, App, ElementId, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window,
};

use crate::{
    components::{
        media::{CoverMosaic, TrackItem},
        theme::ThemeExt,
    },
    models::{Playlist, SortOrder, Track},
};

/// Full playlist detail view.
///
/// Layout:
/// ```text
/// ┌──────────────────────────────────────────────────────────────────┐
/// │  [CoverMosaic]  Playlist Name                     ≡ Sort ▼      │
/// │                 Owner Name                                       │
/// │                 Playlist • Public • 2025                         │
/// │                 X views • Y tracks • Z minutes                   │
/// │                                                                  │
/// │                 [▶ Play]                                         │
/// ├──────────────────────────────────────────────────────────────────┤
/// │  Track List                                                      │
/// │  ...                                                             │
/// └──────────────────────────────────────────────────────────────────┘
/// ```
#[derive(IntoElement)]
pub struct PlaylistDetail {
    id: ElementId,
    playlist: Option<Playlist>,
    tracks: Vec<Track>,
    current_playing_track_id: Option<String>,
    sort_order: SortOrder,
    on_play: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_track_select: Option<Box<dyn Fn(&Track, &mut Window, &mut App) + 'static>>,
}

impl PlaylistDetail {
    /// Creates a new playlist detail view.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            playlist: None,
            tracks: Vec::new(),
            current_playing_track_id: None,
            sort_order: SortOrder::Default,
            on_play: None,
            on_track_select: None,
        }
    }

    /// Sets the playlist to display.
    pub fn playlist(mut self, playlist: Playlist) -> Self {
        self.playlist = Some(playlist);
        self
    }

    /// Sets the tracks in the playlist.
    pub fn tracks(mut self, tracks: Vec<Track>) -> Self {
        self.tracks = tracks;
        self
    }

    /// Sets the currently playing track ID.
    pub fn current_playing(mut self, track_id: Option<String>) -> Self {
        self.current_playing_track_id = track_id;
        self
    }

    /// Sets the sort order.
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = order;
        self
    }

    /// Sets the play button handler.
    pub fn on_play(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_play = Some(Box::new(handler));
        self
    }

    /// Sets the track selection handler.
    pub fn on_track_select(
        mut self,
        handler: impl Fn(&Track, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_track_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for PlaylistDetail {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let id = self.id;
        let tracks = self.tracks;
        let current_playing_track_id = self.current_playing_track_id;
        let on_play = self.on_play;

        div()
            .id(id)
            .w_full()
            .flex()
            .flex_col()
            .gap(px(24.0))
            // Header section
            .when_some(self.playlist, |el, playlist| {
                el.child(
                    div()
                        .flex()
                        .gap(px(24.0))
                        // Cover mosaic
                        .child(CoverMosaic::new(
                            playlist
                                .cover_images
                                .iter()
                                .map(|s| s.to_string())
                                .collect(),
                            200.0,
                        ))
                        // Playlist info
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .justify_end()
                                .gap(px(8.0))
                                // Name
                                .child(
                                    div()
                                        .text_color(theme.text_primary)
                                        .text_size(px(32.0))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .child(playlist.name.clone()),
                                )
                                // Owner
                                .child(
                                    div()
                                        .text_color(theme.text_secondary)
                                        .text_size(px(14.0))
                                        .child(playlist.owner.clone()),
                                )
                                // Metadata line
                                .child(
                                    div()
                                        .text_color(theme.text_muted)
                                        .text_size(px(12.0))
                                        .child(format!(
                                            "Playlist • {} • {}",
                                            playlist.visibility.label(),
                                            playlist.year
                                        )),
                                )
                                // Stats line
                                .child(
                                    div()
                                        .text_color(theme.text_muted)
                                        .text_size(px(12.0))
                                        .child(playlist.stats_summary()),
                                )
                                // Play button
                                .child(
                                    div()
                                        .mt(px(16.0))
                                        .child(
                                            div()
                                                .id("play-playlist-btn")
                                                .px(px(32.0))
                                                .py(px(12.0))
                                                .rounded_full()
                                                .bg(theme.accent)
                                                .text_color(theme.text_primary)
                                                .text_size(px(14.0))
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .cursor_pointer()
                                                .hover(|style| style.bg(theme.accent_hover))
                                                .when_some(on_play, |el, handler| {
                                                    el.on_click(move |_event, window, cx| {
                                                        handler(window, cx)
                                                    })
                                                })
                                                .child("▶ Play"),
                                        ),
                                ),
                        ),
                )
            })
            // Track list
            .child(
                div()
                    .flex()
                    .flex_col()
                    .children(tracks.into_iter().enumerate().map(|(index, track)| {
                        let is_playing = current_playing_track_id
                            .as_ref()
                            .map(|id| id == &track.id)
                            .unwrap_or(false);

                        TrackItem::new(format!("playlist-track-{}", index), track)
                            .playing(is_playing)
                            .with_index(index)
                    })),
            )
    }
}
