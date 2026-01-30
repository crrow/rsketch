/// Bottom player bar component.
///
/// Combines now-playing info, playback controls, progress slider, and volume
/// into a single bar fixed at the bottom of the application window.

use std::time::Duration;

use gpui::{
    div, prelude::*, px, App, ElementId, IntoElement, ParentElement, SharedString, Styled, Window,
};

use crate::components::theme::ThemeExt;

use super::{PlaybackControls, ProgressSlider, VolumeControl};

/// Complete player bar for the bottom of the application.
///
/// Layout:
/// ```text
/// ┌────────────────────────────────────────────────────────────────────────┐
/// │  [cover]  Track Title          [◄◄] [▶] [►►]           🔊━━━━━━━━      │
/// │   48px    Artist               ━━━●━━━━━━━━━━━━━  2:37/4:01            │
/// └────────────────────────────────────────────────────────────────────────┘
/// ```
#[derive(IntoElement)]
pub struct PlayerBar {
    id: ElementId,
    track_title: Option<SharedString>,
    artist_name: Option<SharedString>,
    is_playing: bool,
    has_previous: bool,
    has_next: bool,
    current_time: Duration,
    total_duration: Duration,
    volume: f32,
    is_muted: bool,
}

impl PlayerBar {
    /// Creates a new player bar.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            track_title: None,
            artist_name: None,
            is_playing: false,
            has_previous: false,
            has_next: false,
            current_time: Duration::ZERO,
            total_duration: Duration::ZERO,
            volume: 0.5,
            is_muted: false,
        }
    }

    /// Sets the currently playing track info.
    pub fn now_playing(
        mut self,
        title: impl Into<SharedString>,
        artist: impl Into<SharedString>,
    ) -> Self {
        self.track_title = Some(title.into());
        self.artist_name = Some(artist.into());
        self
    }

    /// Sets whether audio is currently playing.
    pub fn playing(mut self, is_playing: bool) -> Self {
        self.is_playing = is_playing;
        self
    }

    /// Sets navigation availability.
    pub fn navigation(mut self, has_previous: bool, has_next: bool) -> Self {
        self.has_previous = has_previous;
        self.has_next = has_next;
        self
    }

    /// Sets the progress state.
    pub fn progress(mut self, current: Duration, total: Duration) -> Self {
        self.current_time = current;
        self.total_duration = total;
        self
    }

    /// Sets the volume state.
    pub fn volume(mut self, volume: f32, is_muted: bool) -> Self {
        self.volume = volume;
        self.is_muted = is_muted;
        self
    }
}

impl RenderOnce for PlayerBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let has_track = self.track_title.is_some();

        div()
            .id(self.id)
            .w_full()
            .h(px(80.0))
            .bg(theme.background_secondary)
            .border_t_1()
            .border_color(theme.border)
            .px(px(16.0))
            .flex()
            .flex_col()
            .justify_center()
            .gap(px(8.0))
            .child(
                // Top row: track info, controls, volume
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(16.0))
                    // Now playing info (left)
                    .child(
                        div()
                            .w(px(200.0))
                            .flex()
                            .items_center()
                            .gap(px(12.0))
                            // Cover placeholder
                            .child(
                                div()
                                    .size(px(48.0))
                                    .rounded(px(4.0))
                                    .bg(theme.background_elevated)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(theme.text_muted)
                                    .when(has_track, |el| el.child("♪")),
                            )
                            // Track info
                            .child(
                                div()
                                    .flex_1()
                                    .overflow_hidden()
                                    .when_some(self.track_title.clone(), |el, title| {
                                        el.child(
                                            div()
                                                .text_color(theme.text_primary)
                                                .text_size(px(14.0))
                                                .overflow_hidden()
                                                .text_ellipsis()
                                                .child(title),
                                        )
                                    })
                                    .when_some(self.artist_name.clone(), |el, artist| {
                                        el.child(
                                            div()
                                                .text_color(theme.text_secondary)
                                                .text_size(px(12.0))
                                                .overflow_hidden()
                                                .text_ellipsis()
                                                .child(artist),
                                        )
                                    }),
                            ),
                    )
                    // Playback controls (center)
                    .child(
                        div().flex_1().flex().justify_center().child(
                            PlaybackControls::new("playback-controls")
                                .playing(self.is_playing)
                                .has_previous(self.has_previous)
                                .has_next(self.has_next),
                        ),
                    )
                    // Volume (right)
                    .child(
                        div().w(px(140.0)).flex().justify_end().child(
                            VolumeControl::new("volume-control")
                                .volume(self.volume)
                                .muted(self.is_muted),
                        ),
                    ),
            )
            // Bottom row: progress bar
            .child(
                ProgressSlider::new("progress-slider")
                    .current_time(self.current_time)
                    .total_duration(self.total_duration),
            )
    }
}
