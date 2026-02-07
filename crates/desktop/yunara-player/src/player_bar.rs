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

//! PlayerBar - Bottom player controls with state management.

use std::time::Duration;

use gpui::{
    AnyView, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, WeakEntity, Window, div, img, prelude::*, px, svg,
};
use yunara_ui::components::theme::ThemeExt;

use crate::{AppState, dock::DockPanel};

/// Player bar component that manages playback controls and state.
///
/// This component:
/// - Holds a reference to AppState (which contains PlayerState)
/// - Renders the complete player bar UI using yunara-ui primitives
/// - Handles user interactions and updates state
pub struct PlayerBar {
    /// Weak reference to self for event handlers
    weak_self: WeakEntity<Self>,
    /// Reference to global application state
    app_state: AppState,
}

impl PlayerBar {
    /// Creates a new PlayerBar.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            app_state,
        }
    }

    fn format_time(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{}:{:02}", minutes, seconds)
        }
    }
}

impl Render for PlayerBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // Read player state
        let player_state = self.app_state.player_state().read();
        let is_playing = player_state.playback.is_playing;
        let _has_previous = player_state.playback.has_previous;
        let _has_next = player_state.playback.has_next;
        let current_time = player_state.progress.current_time;
        let total_duration = player_state.progress.total_duration;
        let volume = player_state.volume.volume;
        let is_muted = player_state.volume.is_muted;
        let now_playing = player_state.now_playing.clone();
        drop(player_state); // Release lock

        let has_track = now_playing.is_some();

        // Calculate progress
        let progress = if total_duration.is_zero() {
            0.0
        } else {
            current_time.as_secs_f32() / total_duration.as_secs_f32()
        };

        // Volume
        let effective_volume = if is_muted { 0.0 } else { volume };
        let volume_width = effective_volume * 100.0;

        // Time strings
        let _current_str = Self::format_time(current_time);
        let _total_str = Self::format_time(total_duration);

        // Control button helper
        let control_button =
            |id: &'static str, icon_path: &'static str, size: f32, icon_size: f32| {
                let inner_size = icon_size;
                div()
                    .id(id)
                    .w(px(size))
                    .h(px(size))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .child(
                        svg()
                            .path(icon_path)
                            .size(px(inner_size))
                            .text_color(theme.text_primary),
                    )
            };

        // Mute toggle handler
        let weak_self = self.weak_self.clone();
        let app_state = self.app_state.clone();

        div()
            .id("player-bar")
            .h(px(72.0))
            .flex()
            .flex_col()
            .bg(theme.background_secondary)
            // Progress bar at top
            .child(
                div()
                    .w_full()
                    .h(px(4.0))
                    .bg(theme.progress_track)
                    .cursor_pointer()
                    .child(
                        div()
                            .w(gpui::relative(progress))
                            .h(px(4.0))
                            .bg(theme.progress_fill),
                    ),
            )
            // Main player bar content
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    // Left: Playback controls
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(200.0))
                            .gap_4()
                            .child(control_button(
                                "prev-btn",
                                yunara_assets::icons::MEDIA_PREVIOUS,
                                36.0,
                                22.0,
                            ))
                            .child(control_button(
                                "play-btn",
                                if is_playing {
                                    yunara_assets::icons::MEDIA_PAUSE
                                } else {
                                    yunara_assets::icons::MEDIA_PLAY
                                },
                                44.0,
                                28.0,
                            ))
                            .child(control_button(
                                "next-btn",
                                yunara_assets::icons::MEDIA_NEXT,
                                36.0,
                                22.0,
                            )),
                    )
                    // Center: Album art + song info
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_3()
                            .px(px(16.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    // Cover art
                                    .child(
                                        div()
                                            .w(px(48.0))
                                            .h(px(48.0))
                                            .rounded(px(4.0))
                                            .bg(theme.background_elevated)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_color(theme.text_muted)
                                            .when(has_track && now_playing.as_ref().and_then(|n| n.cover_url.as_ref()).is_none(), |el| el.child("♪"))
                                            .when_some(now_playing.as_ref().and_then(|n| n.cover_url.as_ref()), |el, url| {
                                                el.child(
                                                    img(url.clone())
                                                        .w(px(48.0))
                                                        .h(px(48.0))
                                                        .rounded(px(4.0)),
                                                )
                                            }),
                                    )
                                    // Track info
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .overflow_hidden()
                                            .when_some(now_playing.as_ref().map(|n| &n.track_title), |el, title| {
                                                el.child(
                                                    div()
                                                        .text_color(theme.text_primary)
                                                        .text_sm()
                                                        .overflow_hidden()
                                                        .text_ellipsis()
                                                        .child(title.clone()),
                                                )
                                            })
                                            .when_some(now_playing.as_ref().map(|n| &n.artist_name), |el, artist| {
                                                el.child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme.text_secondary)
                                                        .overflow_hidden()
                                                        .text_ellipsis()
                                                        .child(artist.clone()),
                                                )
                                            }),
                                    ),
                            ),
                    )
                    // Right: Volume control
                    .child(
                        div()
                            .id("volume-control")
                            .flex()
                            .items_center()
                            .justify_end()
                            .w(px(200.0))
                            .pr(px(16.0))
                            .gap_2()
                            .group("volume")
                            // Volume slider - hidden by default, shown on hover
                            .child(
                                div()
                                    .invisible()
                                    .group_hover("volume", |style| style.visible())
                                    .flex()
                                    .items_center()
                                    .child(
                                        div()
                                            .w(px(100.0))
                                            .h(px(4.0))
                                            .rounded_full()
                                            .bg(theme.progress_track)
                                            .cursor_pointer()
                                            .child(
                                                div()
                                                    .w(px(volume_width))
                                                    .h(px(4.0))
                                                    .rounded_full()
                                                    .bg(theme.text_primary),
                                            ),
                                    ),
                            )
                            // Volume icon
                            .child({
                                let icon_path = if is_muted {
                                    yunara_assets::icons::VOLUME_MUTED
                                } else {
                                    yunara_assets::icons::VOLUME
                                };

                                div()
                                    .id("volume-icon")
                                    .w(px(36.0))
                                    .h(px(36.0))
                                    .rounded_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .hover(|style| style.bg(theme.hover))
                                    .on_click(move |_event, _window, cx| {
                                        app_state.player_state().write().volume.toggle_mute();
                                        if let Some(this) = weak_self.upgrade() {
                                            this.update(cx, |_, cx| cx.notify());
                                        }
                                    })
                                    .child(
                                        svg()
                                            .path(icon_path)
                                            .size(px(20.0))
                                            .text_color(theme.text_secondary),
                                    )
                            }),
                    ),
            )
    }
}

impl DockPanel for PlayerBar {
    fn title(&self) -> String { "Player".to_string() }

    fn icon(&self) -> Option<&'static str> { None }

    fn to_any_view(&self) -> AnyView {
        self.weak_self
            .upgrade()
            .map(AnyView::from)
            .expect("PlayerBar view should still be alive")
    }
}
