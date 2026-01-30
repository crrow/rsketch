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

/// Track item component for displaying a single track in a list.
///
/// Shows track cover art, title, artist, album, duration, and like status.
/// Supports hover states and playing indicator.
use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px,
};

use crate::{components::theme::ThemeExt, models::Track};

/// A single track item for display in track lists.
///
/// Layout:
/// ```text
/// ┌─────────────────────────────────────────────────────────────────┐
/// │  [cover]   Track Title                                    3:41  │
/// │   40px     Artist • Album                                       │
/// └─────────────────────────────────────────────────────────────────┘
/// ```
#[derive(IntoElement)]
pub struct TrackItem {
    id:         ElementId,
    track:      Track,
    is_playing: bool,
    show_index: Option<usize>,
    on_click:   Option<Box<dyn Fn(&Track, &mut Window, &mut App) + 'static>>,
}

impl TrackItem {
    /// Creates a new TrackItem for the given track.
    pub fn new(id: impl Into<ElementId>, track: Track) -> Self {
        Self {
            id: id.into(),
            track,
            is_playing: false,
            show_index: None,
            on_click: None,
        }
    }

    /// Sets whether this track is currently playing.
    pub fn playing(mut self, is_playing: bool) -> Self {
        self.is_playing = is_playing;
        self
    }

    /// Shows the track index instead of cover art.
    pub fn with_index(mut self, index: usize) -> Self {
        self.show_index = Some(index);
        self
    }

    /// Sets the click handler for when the track is selected.
    pub fn on_click(mut self, handler: impl Fn(&Track, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TrackItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // Extract all values we need before moving on_click
        let id = self.id;
        let track = self.track;
        let is_playing = self.is_playing;
        let show_index = self.show_index;
        let on_click = self.on_click;

        // Prepare display values
        let subtitle: SharedString = if let Some(album) = &track.album {
            format!("{} • {}", track.artist, album).into()
        } else {
            track.artist.clone()
        };
        let title = track.title.clone();
        let duration = track.formatted_duration();
        let index_display = show_index.map(|i| format!("{}", i + 1));

        // Clone track for the click handler
        let track_for_click = track.clone();

        div()
            .id(id)
            .w_full()
            .h(px(56.0))
            .px(px(16.0))
            .flex()
            .items_center()
            .gap(px(12.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .hover(|style| style.bg(theme.hover))
            .when(is_playing, |el| el.bg(theme.active))
            .when_some(on_click, |el, handler| {
                el.on_click(move |_event, window, cx| {
                    handler(&track_for_click, window, cx);
                })
            })
            .child(
                // Cover art or index
                div()
                    .size(px(40.0))
                    .rounded(px(4.0))
                    .bg(theme.background_elevated)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.text_secondary)
                    .text_size(px(12.0))
                    .child(index_display.unwrap_or_else(|| "♪".to_string())),
            )
            .child(
                // Track info (title and subtitle)
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_color(if is_playing {
                                theme.accent
                            } else {
                                theme.text_primary
                            })
                            .text_size(px(14.0))
                            .line_height(px(20.0))
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(title),
                    )
                    .child(
                        div()
                            .text_color(theme.text_secondary)
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(subtitle),
                    ),
            )
            .child(
                // Duration
                div()
                    .text_color(theme.text_secondary)
                    .text_size(px(12.0))
                    .child(duration),
            )
    }
}
