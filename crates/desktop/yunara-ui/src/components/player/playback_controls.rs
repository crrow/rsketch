/// Playback control buttons component.
///
/// Provides previous, play/pause, and next track buttons with
/// visual feedback for enabled/disabled states.

use gpui::{
    div, prelude::*, px, App, ElementId, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window,
};

use crate::components::theme::ThemeExt;

/// Playback control buttons (previous, play/pause, next).
///
/// Layout:
/// ```text
/// [◄◄] [▶/❚❚] [►►]
/// ```
#[derive(IntoElement)]
pub struct PlaybackControls {
    id: ElementId,
    is_playing: bool,
    has_previous: bool,
    has_next: bool,
    on_previous: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_play_pause: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_next: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl PlaybackControls {
    /// Creates new playback controls.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            is_playing: false,
            has_previous: false,
            has_next: false,
            on_previous: None,
            on_play_pause: None,
            on_next: None,
        }
    }

    /// Sets whether audio is currently playing.
    pub fn playing(mut self, is_playing: bool) -> Self {
        self.is_playing = is_playing;
        self
    }

    /// Sets whether there is a previous track available.
    pub fn has_previous(mut self, has_previous: bool) -> Self {
        self.has_previous = has_previous;
        self
    }

    /// Sets whether there is a next track available.
    pub fn has_next(mut self, has_next: bool) -> Self {
        self.has_next = has_next;
        self
    }

    /// Sets the handler for the previous button.
    pub fn on_previous(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_previous = Some(Box::new(handler));
        self
    }

    /// Sets the handler for the play/pause button.
    pub fn on_play_pause(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_play_pause = Some(Box::new(handler));
        self
    }

    /// Sets the handler for the next button.
    pub fn on_next(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_next = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for PlaybackControls {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let id = self.id;
        let is_playing = self.is_playing;
        let has_previous = self.has_previous;
        let has_next = self.has_next;
        let on_previous = self.on_previous;
        let on_play_pause = self.on_play_pause;
        let on_next = self.on_next;

        div()
            .id(id)
            .flex()
            .items_center()
            .gap(px(8.0))
            // Previous button
            .child(
                div()
                    .id("prev-btn")
                    .size(px(32.0))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .text_color(if has_previous {
                        theme.text_primary
                    } else {
                        theme.text_muted
                    })
                    .hover(|style| {
                        if has_previous {
                            style.bg(theme.hover)
                        } else {
                            style
                        }
                    })
                    .when_some(on_previous.filter(|_| has_previous), |el, handler| {
                        el.on_click(move |_event, window, cx| handler(window, cx))
                    })
                    .child("◄◄"),
            )
            // Play/Pause button
            .child(
                div()
                    .id("play-btn")
                    .size(px(40.0))
                    .rounded_full()
                    .bg(theme.text_primary)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .text_color(theme.background_primary)
                    .hover(|style| style.bg(theme.text_secondary))
                    .when_some(on_play_pause, |el, handler| {
                        el.on_click(move |_event, window, cx| handler(window, cx))
                    })
                    .child(if is_playing { "❚❚" } else { "▶" }),
            )
            // Next button
            .child(
                div()
                    .id("next-btn")
                    .size(px(32.0))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .text_color(if has_next {
                        theme.text_primary
                    } else {
                        theme.text_muted
                    })
                    .hover(|style| {
                        if has_next {
                            style.bg(theme.hover)
                        } else {
                            style
                        }
                    })
                    .when_some(on_next.filter(|_| has_next), |el, handler| {
                        el.on_click(move |_event, window, cx| handler(window, cx))
                    })
                    .child("►►"),
            )
    }
}
