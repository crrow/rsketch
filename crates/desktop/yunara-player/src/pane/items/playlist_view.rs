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
    AnyView, Context, EntityId, InteractiveElement, IntoElement, ParentElement, Render, Rgba,
    StatefulInteractiveElement, Styled, StyledImage, WeakEntity, img, prelude::FluentBuilder, px,
};
use yunara_ui::components::theme::ThemeExt;
use ytmapi_rs::common::YoutubeID;
use ytmapi_rs::parse::PlaylistItem;

use crate::{app_state::AppState, pane::PaneItem};

/// Fixed height for each playlist item in pixels
const ITEM_HEIGHT: f32 = 60.0;
/// Trigger load more when this many items remain before reaching bottom
const LOAD_THRESHOLD: usize = 20;

/// View displaying a specific playlist's contents.
pub struct PlaylistView {
    weak_self:     WeakEntity<Self>,
    app_state:     AppState,
    playlist_id:   String,
    playlist_name: String,
    thumbnail_url: Option<String>,
    gradient_top_color: Option<Rgba>,
    tracks:        Vec<PlaylistItem>,
    loading:       bool,

    // Pagination state
    continuation_token: Option<String>,
    has_more:           bool,
    loading_more:       bool,

    // Virtual scroll state
    scroll_offset:  f32,
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
            gradient_top_color: None,
            tracks: Vec::new(),
            loading: true,
            continuation_token: None,
            has_more: false,
            loading_more: false,
            scroll_offset: 0.0,
        };
        view.load_gradient_color(cx);
        view.load_first_page(cx);
        view
    }

    /// Loads the first page of playlist tracks (50 items)
    fn load_first_page(&mut self, cx: &mut Context<Self>) {
        self.loading = true;
        let service = self.app_state.playlist_service().clone();
        let playlist_id = self.playlist_id.clone();

        let tokio_task = gpui_tokio::Tokio::spawn(cx, async move {
            service.get_playlist_first_page(&playlist_id).await
        });

        cx.spawn(async move |this, cx| {
            let result = match tokio_task.await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Playlist first page task panicked: {}", e);
                    return;
                }
            };

            match cx.update(|cx| {
                this.update(cx, |view, cx| {
                    match result {
                        Ok(page) => {
                            view.tracks = page.items;
                            view.has_more = page.continuation.is_some();
                            view.continuation_token = page.continuation;
                        }
                        Err(e) => {
                            tracing::error!("Failed to load first page: {}", e);
                            view.tracks = Vec::new();
                            view.has_more = false;
                        }
                    }

                    if view.thumbnail_url.is_none() {
                        let playlists = view.app_state.playlist_service().get_playlists();
                        view.thumbnail_url = select_thumbnail_url(&playlists, &view.playlist_id);
                        view.load_gradient_color(cx);
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

    /// Loads the next page of playlist tracks
    fn load_next_page(&mut self, cx: &mut Context<Self>) {
        if self.loading_more || !self.has_more {
            return;
        }

        let Some(token) = self.continuation_token.clone() else {
            return;
        };

        self.loading_more = true;
        let service = self.app_state.playlist_service().clone();

        let tokio_task = gpui_tokio::Tokio::spawn(cx, async move {
            service.get_playlist_next_page(token).await
        });

        cx.spawn(async move |this, cx| {
            let result = match tokio_task.await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Playlist next page task panicked: {}", e);
                    return;
                }
            };

            match cx.update(|cx| {
                this.update(cx, |view, cx| {
                    match result {
                        Ok(page) => {
                            view.tracks.extend(page.items);
                            view.has_more = page.continuation.is_some();
                            view.continuation_token = page.continuation;
                        }
                        Err(e) => {
                            tracing::error!("Failed to load next page: {}", e);
                            view.has_more = false;
                        }
                    }
                    view.loading_more = false;
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

    /// Spawns an async task to load playlist tracks (legacy, kept for compatibility).
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
                        view.load_gradient_color(cx);
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

    fn load_gradient_color(&mut self, cx: &mut Context<Self>) {
        let Some(thumbnail_url) = self.thumbnail_url.clone() else {
            return;
        };

        let tokio_task = gpui_tokio::Tokio::spawn(cx, async move {
            fetch_dominant_color(&thumbnail_url).await
        });

        cx.spawn(async move |this, cx| {
            let result = match tokio_task.await {
                Ok(result) => result,
                Err(error) => {
                    tracing::error!("Gradient color task panicked: {}", error);
                    return;
                }
            };

            match result {
                Ok(color) => {
                    let _ = this.update(cx, |view, cx| {
                        view.gradient_top_color = Some(color);
                        cx.notify();
                    });
                }
                Err(error) => {
                    tracing::error!("Failed to compute gradient color: {}", error);
                }
            }
        })
        .detach();
    }

    /// Checks if we need to load more items based on scroll position
    fn check_load_more(&mut self, viewport_height: f32, cx: &mut Context<Self>) {
        if self.tracks.is_empty() {
            return;
        }

        // Calculate visible range
        let first_visible = (self.scroll_offset / ITEM_HEIGHT).floor() as usize;
        let visible_count = (viewport_height / ITEM_HEIGHT).ceil() as usize + 2;
        let last_visible = (first_visible + visible_count).min(self.tracks.len());

        // Check if near bottom
        let remaining = self.tracks.len().saturating_sub(last_visible);
        if remaining < LOAD_THRESHOLD && self.has_more && !self.loading_more {
            self.load_next_page(cx);
        }
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

async fn fetch_dominant_color(url: &str) -> anyhow::Result<Rgba> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let image = image::load_from_memory(&bytes)?;
    let image = image.thumbnail(32, 32).to_rgb8();

    let mut total_r = 0u64;
    let mut total_g = 0u64;
    let mut total_b = 0u64;
    let mut count = 0u64;

    for pixel in image.pixels() {
        let [r, g, b] = pixel.0;
        total_r += r as u64;
        total_g += g as u64;
        total_b += b as u64;
        count += 1;
    }

    if count == 0 {
        return Ok(Rgba::default());
    }

    Ok(Rgba {
        r: total_r as f32 / count as f32 / 255.0,
        g: total_g as f32 / count as f32 / 255.0,
        b: total_b as f32 / count as f32 / 255.0,
        a: 1.0,
    })
}

fn blend_colors(top: Rgba, bottom: Rgba, t: f32) -> Rgba {
    let t = t.clamp(0.0, 1.0);
    Rgba {
        r: top.r * (1.0 - t) + bottom.r * t,
        g: top.g * (1.0 - t) + bottom.g * t,
        b: top.b * (1.0 - t) + bottom.b * t,
        a: 1.0,
    }
}

fn with_alpha(color: Rgba, alpha: f32) -> Rgba {
    Rgba {
        r: color.r,
        g: color.g,
        b: color.b,
        a: alpha.clamp(0.0, 1.0),
    }
}

impl Render for PlaylistView {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = window.viewport_size();

        let theme = cx.theme();
        let has_thumbnail = self.thumbnail_url.is_some();
        let thumbnail_url = self.thumbnail_url.clone();
        let top_color = self
            .gradient_top_color
            .unwrap_or(theme.background_elevated);

        let glass_base = blend_colors(top_color, theme.background_primary, 0.45);
        let glass_bg = with_alpha(glass_base, 0.45);
        let glass_border = with_alpha(theme.text_primary, 0.12);
        let glass_highlight = with_alpha(theme.text_primary, 0.08);
        let glass_inner = with_alpha(theme.background_primary, 0.35);
        let icon_bg = Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.08,
        };
        let icon_bg_hover = Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.16,
        };

        let hero_card_size = 260.0;
        let hero_card_radius = 18.0_f32;
        let header_panel_radius = 18.0_f32;
        let content_pad = 18.0_f32;
        let meta_gap = 10.0_f32;
        let header_width = 320.0_f32;

        let header = gpui::div()
            .flex()
            .flex_col()
            .items_start()
            .text_left()
            .gap(px(meta_gap))
            .child(
                gpui::div()
                    .w(px(header_width))
                    .rounded(px(header_panel_radius))
                    .bg(glass_bg)
                    .border_1()
                    .border_color(glass_border)
                    .p(px(18.0))
                    .child(
                        gpui::div()
                            .w(px(hero_card_size))
                            .h(px(hero_card_size))
                            .rounded(px(hero_card_radius))
                            .bg(theme.background_elevated)
                            .overflow_hidden()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.text_muted)
                            .child(
                                gpui::div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .right_0()
                                    .bottom_0()
                                    .bg(glass_inner),
                            )
                            .when_some(thumbnail_url, |el, url| {
                                el.child(
                                    img(url)
                                        .w(px(hero_card_size))
                                        .h(px(hero_card_size))
                                        .rounded(px(hero_card_radius))
                                        .object_fit(gpui::ObjectFit::Cover),
                                )
                            })
                            .when(!has_thumbnail, |el| el.child("♪")),
                    )
                    .child(
                        gpui::div()
                            .mt(px(16.0))
                            .text_3xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(self.playlist_name.clone()),
                    )
                    .child(
                        gpui::div()
                            .mt(px(6.0))
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .child(
                                gpui::div()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(11.0))
                                    .bg(theme.background_elevated)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child("🙂"),
                            )
                            .child("Crrow"),
                    )
                    .child(
                        gpui::div()
                            .mt(px(8.0))
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .child("Auto playlist · 2026"),
                    )
                    .child(
                        gpui::div()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .child(format!("{} songs · 11+ hours", self.tracks.len())),
                    )
                    .child(
                        gpui::div()
                            .mt(px(10.0))
                            .text_sm()
                            .text_color(theme.text_muted)
                            .child("A smart mix of your most played tracks."),
                    )
                    .child(
                        gpui::div()
                            .mt(px(18.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_3()
                            .child(
                                gpui::div()
                                    .w(px(40.0))
                                    .h(px(40.0))
                                    .rounded(px(20.0))
                                    .bg(icon_bg)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(|style| style.bg(icon_bg_hover))
                                    .child(
                                        img(yunara_assets::icons::DOWNLOAD)
                                            .w(px(18.0))
                                            .h(px(18.0)),
                                    ),
                            )
                            .child(
                                gpui::div()
                                    .w(px(40.0))
                                    .h(px(40.0))
                                    .rounded(px(20.0))
                                    .bg(icon_bg)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(|style| style.bg(icon_bg_hover))
                                    .child(
                                        img(yunara_assets::icons::PLAYLIST_BOOKMARK)
                                            .w(px(18.0))
                                            .h(px(18.0)),
                                    ),
                            )
                            .child(
                                gpui::div()
                                    .w(px(56.0))
                                    .h(px(56.0))
                                    .rounded(px(28.0))
                                    .bg(Rgba {
                                        r: 1.0,
                                        g: 1.0,
                                        b: 1.0,
                                        a: 1.0,
                                    })
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        img(yunara_assets::icons::PLAY_BLACK)
                                            .w(px(22.0))
                                            .h(px(22.0)),
                                    ),
                            )
                            .child(
                                gpui::div()
                                    .w(px(40.0))
                                    .h(px(40.0))
                                    .rounded(px(20.0))
                                    .bg(icon_bg)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(|style| style.bg(icon_bg_hover))
                                    .child(
                                        img(yunara_assets::icons::PLAYLIST_SHARE)
                                            .w(px(18.0))
                                            .h(px(18.0)),
                                    ),
                            )
                            .child(
                                gpui::div()
                                    .w(px(40.0))
                                    .h(px(40.0))
                                    .rounded(px(20.0))
                                    .bg(icon_bg)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(|style| style.bg(icon_bg_hover))
                                    .child(
                                        img(yunara_assets::icons::PLAYLIST_MORE)
                                            .w(px(18.0))
                                            .h(px(18.0)),
                                    ),
                            ),
                    ),
            );

        gpui::div()
            .id("playlist-view")
            .flex()
            .flex_row()
            .w_full()
            .h_full()
            .relative()
            .p(px(content_pad))
            .gap_4()
            .child(header)
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(px(0.0))
                    .h_full()
                    .overflow_hidden()
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
                    // Track list with simplified virtual scrolling
                    .when(!self.loading && !self.tracks.is_empty(), |el| {
                        // For now, render all tracks but with fixed heights
                        // TODO: Implement full virtual scrolling when scroll events are available
                        let tracks_len = self.tracks.len();
                        let loading_more = self.loading_more;

                        el.child(
                            gpui::div()
                                .id("playlist-tracks")
                                .flex()
                                .flex_col()
                                .flex_1()
                                .min_h(px(0.0))
                                .overflow_y_scroll()
                                // Render all items for now
                                .children(self.tracks.iter().map(|item| {
                                    let (title, artist, duration, thumbnail_url) =
                                        track_display_info(item);
                                    let has_thumbnail = thumbnail_url.is_some();

                                    gpui::div()
                                        .h(px(ITEM_HEIGHT))
                                        .flex()
                                        .items_center()
                                        .gap_4()
                                        .px_2()
                                        .py(gpui::px(8.0))
                                        .rounded(gpui::px(6.0))
                                        .cursor_pointer()
                                        .hover(|style| style.bg(theme.hover))
                                        // Thumbnail
                                        .child(
                                            gpui::div()
                                                .w(gpui::px(48.0))
                                                .h(gpui::px(48.0))
                                                .rounded(gpui::px(6.0))
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
                                        .child(
                                            gpui::div()
                                                .text_sm()
                                                .text_color(theme.text_muted)
                                                .child("♥"),
                                        )
                                        // Duration
                                        .child(
                                            gpui::div()
                                                .text_sm()
                                                .text_color(theme.text_muted)
                                                .child(duration.to_owned()),
                                        )
                                }))
                                // Loading more indicator
                                .when(loading_more, |el| {
                                    el.child(
                                        gpui::div()
                                            .text_sm()
                                            .text_color(theme.text_muted)
                                            .p_4()
                                            .child("Loading more...")
                                    )
                                })
                        )
                    }),
            )
    }
}
