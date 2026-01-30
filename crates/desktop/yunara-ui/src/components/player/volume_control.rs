/// Volume control component with mute toggle and slider.
///
/// Displays a speaker icon (that toggles mute) and a volume slider.

use gpui::{
    div, prelude::*, px, App, ElementId, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window,
};

use crate::components::theme::ThemeExt;

/// Volume control with mute button and slider.
///
/// Layout:
/// ```text
/// 🔊━━━━━━━━
/// ```
#[derive(IntoElement)]
pub struct VolumeControl {
    id: ElementId,
    volume: f32,
    is_muted: bool,
    on_volume_change: Option<Box<dyn Fn(f32, &mut Window, &mut App) + 'static>>,
    on_mute_toggle: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl VolumeControl {
    /// Creates a new volume control.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            volume: 0.5,
            is_muted: false,
            on_volume_change: None,
            on_mute_toggle: None,
        }
    }

    /// Sets the current volume level (0.0 to 1.0).
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Sets whether volume is muted.
    pub fn muted(mut self, is_muted: bool) -> Self {
        self.is_muted = is_muted;
        self
    }

    /// Sets the handler for volume changes.
    pub fn on_volume_change(
        mut self,
        handler: impl Fn(f32, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_volume_change = Some(Box::new(handler));
        self
    }

    /// Sets the handler for mute toggle.
    pub fn on_mute_toggle(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_mute_toggle = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for VolumeControl {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // Compute derived values before moving fields
        let volume = if self.is_muted { 0.0 } else { self.volume };
        let icon = if self.is_muted || self.volume == 0.0 {
            "🔇"
        } else if self.volume < 0.3 {
            "🔈"
        } else if self.volume < 0.7 {
            "🔉"
        } else {
            "🔊"
        };
        let id = self.id;
        let on_mute_toggle = self.on_mute_toggle;

        div()
            .id(id)
            .flex()
            .items_center()
            .gap(px(8.0))
            // Volume icon / mute button
            .child(
                div()
                    .id("volume-icon")
                    .size(px(24.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .text_color(theme.text_secondary)
                    .text_size(px(14.0))
                    .hover(|style| style.text_color(theme.text_primary))
                    .when_some(on_mute_toggle, |el, handler| {
                        el.on_click(move |_event, window, cx| handler(window, cx))
                    })
                    .child(icon),
            )
            // Volume slider
            .child(
                div()
                    .w(px(80.0))
                    .h(px(4.0))
                    .rounded_full()
                    .bg(theme.progress_track)
                    .cursor_pointer()
                    .overflow_hidden()
                    .child(
                        div()
                            .h_full()
                            .rounded_full()
                            .bg(theme.text_secondary)
                            .w(gpui::relative(volume)),
                    ),
            )
    }
}
