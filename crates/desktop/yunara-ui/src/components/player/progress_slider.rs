/// Progress slider component for track playback position.
///
/// Displays current playback progress with a draggable slider
/// and time labels showing current position and total duration.

use std::time::Duration;

use gpui::{div, prelude::*, px, App, ElementId, IntoElement, ParentElement, Styled, Window};

use crate::components::theme::ThemeExt;

/// A progress slider for displaying and controlling playback position.
///
/// Layout:
/// ```text
/// ━━━━━━━━━━━━━━━━━●━━━━━━━━━━━━━━  2:37/4:01
/// ```
#[derive(IntoElement)]
pub struct ProgressSlider {
    id: ElementId,
    current_time: Duration,
    total_duration: Duration,
    on_seek: Option<Box<dyn Fn(f32, &mut Window, &mut App) + 'static>>,
}

impl ProgressSlider {
    /// Creates a new progress slider.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            current_time: Duration::ZERO,
            total_duration: Duration::ZERO,
            on_seek: None,
        }
    }

    /// Sets the current playback time.
    pub fn current_time(mut self, time: Duration) -> Self {
        self.current_time = time;
        self
    }

    /// Sets the total track duration.
    pub fn total_duration(mut self, duration: Duration) -> Self {
        self.total_duration = duration;
        self
    }

    /// Sets the seek handler called when user clicks on the progress bar.
    /// The handler receives a fraction (0.0 to 1.0) representing the target position.
    pub fn on_seek(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_seek = Some(Box::new(handler));
        self
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

impl RenderOnce for ProgressSlider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // Compute progress inline since we're consuming self
        let progress = if self.total_duration.is_zero() {
            0.0
        } else {
            self.current_time.as_secs_f32() / self.total_duration.as_secs_f32()
        };
        let current_str = Self::format_time(self.current_time);
        let total_str = Self::format_time(self.total_duration);
        let id = self.id;

        div()
            .id(id)
            .w_full()
            .flex()
            .items_center()
            .gap(px(8.0))
            // Progress bar
            .child(
                div()
                    .flex_1()
                    .h(px(4.0))
                    .rounded_full()
                    .bg(theme.progress_track)
                    .cursor_pointer()
                    .overflow_hidden()
                    .child(
                        div()
                            .h_full()
                            .rounded_full()
                            .bg(theme.progress_fill)
                            .w(gpui::relative(progress)),
                    ),
            )
            // Time display
            .child(
                div()
                    .text_color(theme.text_secondary)
                    .text_size(px(11.0))
                    .child(format!("{}/{}", current_str, total_str)),
            )
    }
}
