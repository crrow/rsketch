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
    AnyView, Context, EntityId, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, StyledImage, WeakEntity, img, prelude::FluentBuilder, px,
};
use yunara_ui::components::theme::ThemeExt;
use ytmapi_rs::common::YoutubeID;
use ytmapi_rs::parse::PlaylistItem;

use crate::{app_state::AppState, pane::PaneItem};

/// View displaying a specific playlist's contents.
pub struct PlaylistView {
    weak_self:     WeakEntity<Self>,
    app_state:     AppState,
    playlist_id:   String,
    playlist_name: String,
    thumbnail_url: Option<String>,
    tracks:        Vec<PlaylistItem>,
    loading:       bool,
}

impl PlaylistView {
    /// Creates a new playlist view and begins loading tracks.
    pub fn new(
        app_state: AppState,
        playlist_id: String,
        playlist_name: String,
        cx: &mut Context<Self>,
    ) -> Self {
        let playlists = app_state.playlist_service().get_playlists();
        let thumbnail_url = select_thumbnail_url(&playlists, &playlist_id);
        let mut view = Self {
            weak_self: cx.weak_entity(),
            app_state,
            playlist_id,
            playlist_name,
            thumbnail_url,
            tracks: Vec::new(),
            loading: true,
        };
        view.load_tracks(cx);
        view
    }

    /// Spawns an async task to load playlist tracks.
    fn load_tracks(&mut self, cx: &mut Context<Self>) {
        self.loading = true;
        let service = self.app_state.playlist_service().clone();
        let playlist_id = self.playlist_id.clone();

        // Run API call on Tokio runtime, then update GPUI state
        let tokio_task = gpui_tokio::Tokio::spawn(cx, async move {
            service.get_playlist_details(&playlist_id).await
        });

        cx.spawn(async move |this, cx| {
            let result = match tokio_task.await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Playlist load task panicked: {}", e);
                    return;
                }
            };

            match cx.update(|cx| {
                this.update(cx, |view, cx| {
                    match result {
                        Ok(tracks) => {
                            view.tracks = tracks;
                        }
                        Err(e) => {
                            tracing::error!("Failed to load playlist: {}", e);
                            view.tracks = Vec::new();
                        }
                    }

                    if view.thumbnail_url.is_none() {
                        let playlists = view.app_state.playlist_service().get_playlists();
                        view.thumbnail_url = select_thumbnail_url(&playlists, &view.playlist_id);
                    }

                    view.loading = false;
                    cx.notify();
                })
            }) {
                Ok(()) => {}
                Err(error) => {
                    tracing::error!("Failed to update playlist view: {}", error);
                }
            }
        })
        .detach();
    }
}

impl PaneItem for PlaylistView {
    fn entity_id(&self) -> EntityId { self.weak_self.entity_id() }

    fn tab_title(&self) -> String { self.playlist_name.clone() }

    fn to_any_view(&self) -> AnyView {
        self.weak_self
            .upgrade()
            .map(AnyView::from)
            .expect("PlaylistView should still be alive")
    }

    fn can_close(&self) -> bool { true }
}

/// Extracts display info from a PlaylistItem variant.
fn track_display_info(item: &PlaylistItem) -> (&str, String, &str, Option<String>) {
    match item {
        PlaylistItem::Song(song) => {
            let artists = song
                .artists
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let thumbnail_url = song
                .thumbnails
                .iter()
                .max_by_key(|t| t.width.saturating_mul(t.height))
                .map(|t| t.url.clone());
            (&song.title, artists, &song.duration, thumbnail_url)
        }
        PlaylistItem::Video(video) => {
            let thumbnail_url = video
                .thumbnails
                .iter()
                .max_by_key(|t| t.width.saturating_mul(t.height))
                .map(|t| t.url.clone());
            (&video.title, video.channel_name.clone(), &video.duration, thumbnail_url)
        }
        PlaylistItem::Episode(ep) => {
            let duration = match &ep.duration {
                ytmapi_rs::parse::EpisodeDuration::Live => "LIVE",
                ytmapi_rs::parse::EpisodeDuration::Recorded { duration } => duration.as_str(),
            };
            let thumbnail_url = ep
                .thumbnails
                .iter()
                .max_by_key(|t| t.width.saturating_mul(t.height))
                .map(|t| t.url.clone());
            (&ep.title, ep.podcast_name.clone(), duration, thumbnail_url)
        }
        PlaylistItem::UploadSong(upload) => {
            let artists = upload
                .artists
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let thumbnail_url = upload
                .thumbnails
                .iter()
                .max_by_key(|t| t.width.saturating_mul(t.height))
                .map(|t| t.url.clone());
            (&upload.title, artists, &upload.duration, thumbnail_url)
        }
    }
}

fn select_thumbnail_url(
    playlists: &[ytmapi_rs::parse::LibraryPlaylist],
    playlist_id: &str,
) -> Option<String> {
    let playlist = playlists
        .iter()
        .find(|playlist| playlist.playlist_id.get_raw() == playlist_id)?;

    playlist
        .thumbnails
        .iter()
        .max_by_key(|thumbnail| thumbnail.width.saturating_mul(thumbnail.height))
        .map(|thumbnail| thumbnail.url.clone())
}

impl Render for PlaylistView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let has_thumbnail = self.thumbnail_url.is_some();
        let thumbnail_url = self.thumbnail_url.clone();

        gpui::div()
            .id("playlist-view")
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p_4()
            // Header
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .pb_4()
                    .child(
                        gpui::div()
                            .w(px(160.0))
                            .h(px(160.0))
                            .rounded(px(8.0))
                            .bg(theme.background_elevated)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.text_muted)
                            .when_some(thumbnail_url, |el, url| {
                                el.child(
                                    img(url)
                                        .w(px(160.0))
                                        .h(px(160.0))
                                        .rounded(px(8.0)),
                                )
                            })
                            .when(!has_thumbnail, |el| el.child("♪")),
                    )
                    .child(
                        gpui::div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                gpui::div()
                                    .text_2xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme.text_primary)
                                    .child(self.playlist_name.clone()),
                            )
                            .child(
                                gpui::div()
                                    .text_sm()
                                    .text_color(theme.text_muted)
                                    .child(format!("{} songs", self.tracks.len())),
                            ),
                    ),
            )
            // Loading state
            .when(self.loading, |el| {
                el.child(
                    gpui::div()
                        .text_sm()
                        .text_color(theme.text_muted)
                        .child("Loading..."),
                )
            })
            // Empty state
            .when(!self.loading && self.tracks.is_empty(), |el| {
                el.child(
                    gpui::div()
                        .text_sm()
                        .text_color(theme.text_muted)
                        .child("This playlist is empty"),
                )
            })
            // Track list
            .when(!self.loading && !self.tracks.is_empty(), |el| {
                el.child(
                    gpui::div()
                        .id("playlist-tracks")
                        .flex()
                        .flex_col()
                        .flex_1()
                        .overflow_y_scroll()
                        .children(
                        self.tracks.iter().map(|item| {
                            let (title, artist, duration, thumbnail_url) = track_display_info(item);
                            let has_thumbnail = thumbnail_url.is_some();

                            gpui::div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .px_2()
                                .py(gpui::px(6.0))
                                .rounded(gpui::px(4.0))
                                .cursor_pointer()
                                .hover(|style| style.bg(theme.hover))
                                // Thumbnail
                                .child(
                                    gpui::div()
                                        .w(gpui::px(48.0))
                                        .h(gpui::px(48.0))
                                        .rounded(gpui::px(4.0))
                                        .bg(theme.background_elevated)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_color(theme.text_muted)
                                        .overflow_hidden()
                                        .when_some(thumbnail_url, |el, url| {
                                            el.child(
                                                img(url)
                                                    .w(gpui::px(48.0))
                                                    .h(gpui::px(48.0))
                                                    .object_fit(gpui::ObjectFit::Cover),
                                            )
                                        })
                                        .when(!has_thumbnail, |el| el.child("♪")),
                                )
                                // Title and artist
                                .child(
                                    gpui::div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .overflow_hidden()
                                        .child(
                                            gpui::div()
                                                .text_sm()
                                                .text_color(theme.text_primary)
                                                .overflow_hidden()
                                                .child(title.to_owned()),
                                        )
                                        .child(
                                            gpui::div()
                                                .text_xs()
                                                .text_color(theme.text_muted)
                                                .overflow_hidden()
                                                .child(artist),
                                        ),
                                )
                                // Duration
                                .child(
                                    gpui::div()
                                        .text_sm()
                                        .text_color(theme.text_muted)
                                        .child(duration.to_owned()),
                                )
                        }),
                    ),
                )
            })
    }
}
