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
//! Displays the main landing page with the currently playing track and a
//! Song/Video content mode toggle.

use gpui::{
    AnyView, Context, EntityId, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, StyledImage, WeakEntity, prelude::FluentBuilder, px,
};
use yunara_ui::components::theme::ThemeExt;

use crate::{NowPlayingInfo, app_state::AppState, pane::PaneItem};

/// Content display mode toggled by the Song/Video pills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentMode {
    Song,
    Video,
}

/// Home view displaying recently played content and recommendations.
pub struct HomeView {
    weak_self: WeakEntity<Self>,
    app_state: AppState,
    content_mode: ContentMode,
}

impl HomeView {
    /// Creates a new home view.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            app_state,
            content_mode: ContentMode::Song,
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
        let content_mode = self.content_mode;
        let weak_self = self.weak_self.clone();

        // Read current playing state
        let player_state = self.app_state.player_state().read();
        let now_playing = player_state.now_playing.clone();
        drop(player_state);

        // Build toggle pills using the weak handle for click callbacks
        let song_active = content_mode == ContentMode::Song;
        let video_active = content_mode == ContentMode::Video;

        let weak_for_song = weak_self.clone();
        let song_tab = gpui::div()
            .id("toggle-song")
            .px(px(18.0))
            .py(px(8.0))
            .rounded(px(20.0))
            .cursor_pointer()
            .text_sm()
            .when(song_active, |el| {
                el.bg(theme.text_primary)
                    .text_color(theme.background_primary)
            })
            .when(!song_active, |el| {
                el.bg(theme.background_elevated)
                    .text_color(theme.text_secondary)
                    .hover(|style| style.bg(theme.hover))
            })
            .on_click(move |_event, _window, cx| {
                weak_for_song
                    .update(cx, |view, cx| {
                        view.content_mode = ContentMode::Song;
                        cx.notify();
                    })
                    .ok();
            })
            .child("Song");

        let weak_for_video = weak_self.clone();
        let video_tab = gpui::div()
            .id("toggle-video")
            .px(px(18.0))
            .py(px(8.0))
            .rounded(px(20.0))
            .cursor_pointer()
            .text_sm()
            .when(video_active, |el| {
                el.bg(theme.text_primary)
                    .text_color(theme.background_primary)
            })
            .when(!video_active, |el| {
                el.bg(theme.background_elevated)
                    .text_color(theme.text_secondary)
                    .hover(|style| style.bg(theme.hover))
            })
            .on_click(move |_event, _window, cx| {
                weak_for_video
                    .update(cx, |view, cx| {
                        view.content_mode = ContentMode::Video;
                        cx.notify();
                    })
                    .ok();
            })
            .child("Video");

        let toggle_bar = gpui::div()
            .flex()
            .items_center()
            .gap_2()
            .child(song_tab)
            .child(video_tab);

        let content = match content_mode {
            ContentMode::Video => render_video_placeholder(&theme),
            ContentMode::Song => match now_playing {
                Some(info) => render_now_playing(&theme, &info),
                None => render_empty_state(&theme),
            },
        };

        gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .items_center()
            .p_4()
            .gap_4()
            .child(toggle_bar)
            .child(content)
    }
}

/// Renders the empty state shown when no track is playing in Song mode.
fn render_empty_state(theme: &yunara_ui::components::theme::ThemeConfig) -> gpui::Div {
    gpui::div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .w_full()
        .gap_4()
        .child(
            gpui::div()
                .w(px(120.0))
                .h(px(120.0))
                .rounded(px(60.0))
                .bg(theme.background_elevated)
                .flex()
                .items_center()
                .justify_center()
                .text_3xl()
                .text_color(theme.text_muted)
                .child("♫"),
        )
        .child(
            gpui::div()
                .flex()
                .flex_col()
                .items_center()
                .gap_1()
                .child(
                    gpui::div()
                        .text_xl()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child("Nothing playing"),
                )
                .child(
                    gpui::div()
                        .text_sm()
                        .text_color(theme.text_secondary)
                        .child("Select a playlist to start listening"),
                ),
        )
}

/// Renders the video mode placeholder.
fn render_video_placeholder(theme: &yunara_ui::components::theme::ThemeConfig) -> gpui::Div {
    gpui::div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .w_full()
        .gap_4()
        .child(
            gpui::div()
                .w(px(120.0))
                .h(px(120.0))
                .rounded(px(60.0))
                .bg(theme.background_elevated)
                .flex()
                .items_center()
                .justify_center()
                .text_3xl()
                .text_color(theme.text_muted)
                .child("▶"),
        )
        .child(
            gpui::div()
                .flex()
                .flex_col()
                .items_center()
                .gap_1()
                .child(
                    gpui::div()
                        .text_xl()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child("Video playback coming soon"),
                )
                .child(
                    gpui::div()
                        .text_sm()
                        .text_color(theme.text_secondary)
                        .child("Switch to Song mode to listen to music"),
                ),
        )
}

/// Renders the now-playing display with album art, title, and artist.
fn render_now_playing(
    theme: &yunara_ui::components::theme::ThemeConfig,
    info: &NowPlayingInfo,
) -> gpui::Div {
    let cover_url = info.cover_url.clone();
    let has_cover = cover_url.is_some();

    gpui::div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .w_full()
        .gap_4()
        .child(
            gpui::div()
                .w(px(340.0))
                .h(px(340.0))
                .rounded(px(12.0))
                .bg(theme.background_elevated)
                .overflow_hidden()
                .flex()
                .items_center()
                .justify_center()
                .text_color(theme.text_muted)
                .when_some(cover_url, |el, url| {
                    el.child(
                        gpui::img(url)
                            .w(px(340.0))
                            .h(px(340.0))
                            .rounded(px(12.0))
                            .object_fit(gpui::ObjectFit::Cover),
                    )
                })
                .when(!has_cover, |el| {
                    el.child(
                        gpui::div()
                            .text_3xl()
                            .text_color(theme.text_muted)
                            .child("♪"),
                    )
                }),
        )
        .child(
            gpui::div()
                .flex()
                .flex_col()
                .items_center()
                .gap_1()
                .child(
                    gpui::div()
                        .text_xl()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child(info.track_title.clone()),
                )
                .child(
                    gpui::div()
                        .text_base()
                        .text_color(theme.text_secondary)
                        .child(info.artist_name.clone()),
                ),
        )
}
