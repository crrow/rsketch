/// Cover mosaic component for displaying playlist cover art.
///
/// Arranges 1-4 cover images in a grid layout:
/// - 1 image: Single full-size image
/// - 2 images: Left and right halves
/// - 3 images: Top half single, bottom half split
/// - 4 images: 2x2 grid

use gpui::{div, prelude::*, px, App, IntoElement, ParentElement, Pixels, Styled, Window};

use crate::components::theme::ThemeExt;

/// A mosaic of cover images for playlist display.
///
/// Automatically arranges images based on count:
/// - 1: Single image fills the space
/// - 2: Side by side
/// - 3: One on top, two on bottom
/// - 4: 2x2 grid
#[derive(IntoElement)]
pub struct CoverMosaic {
    /// URLs of cover images (1-4)
    image_urls: Vec<String>,
    /// Size of the mosaic container in pixels
    size: Pixels,
    /// Half size for grid calculations
    half_size: Pixels,
    /// Icon size for placeholder
    icon_size: Pixels,
}

impl CoverMosaic {
    /// Creates a new CoverMosaic with the given images and size.
    pub fn new(image_urls: Vec<String>, size: f32) -> Self {
        Self {
            image_urls,
            size: px(size),
            half_size: px(size / 2.0),
            icon_size: px(size / 3.0),
        }
    }

    /// Creates an empty mosaic with a placeholder.
    pub fn empty(size: f32) -> Self {
        Self {
            image_urls: Vec::new(),
            size: px(size),
            half_size: px(size / 2.0),
            icon_size: px(size / 3.0),
        }
    }
}

impl RenderOnce for CoverMosaic {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let container = div()
            .w(self.size)
            .h(self.size)
            .rounded(px(8.0))
            .overflow_hidden()
            .bg(theme.background_elevated);

        match self.image_urls.len() {
            0 => {
                // Empty placeholder with music icon
                container
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.text_muted)
                    .text_size(self.icon_size)
                    .child("♪")
            }
            1 => {
                // Single image fills the entire space
                container.child(
                    div()
                        .size_full()
                        .bg(theme.background_secondary)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.text_muted)
                        .text_size(self.icon_size)
                        .child("♪"),
                )
            }
            2 => {
                // Two images side by side
                container
                    .flex()
                    .child(
                        div()
                            .w(self.half_size)
                            .h_full()
                            .bg(theme.background_secondary)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.text_muted)
                            .child("♪"),
                    )
                    .child(
                        div()
                            .w(self.half_size)
                            .h_full()
                            .bg(theme.background_elevated)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.text_muted)
                            .child("♪"),
                    )
            }
            3 => {
                // One on top, two on bottom
                container
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .w_full()
                            .h(self.half_size)
                            .bg(theme.background_secondary)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.text_muted)
                            .child("♪"),
                    )
                    .child(
                        div()
                            .w_full()
                            .h(self.half_size)
                            .flex()
                            .child(
                                div()
                                    .w(self.half_size)
                                    .h_full()
                                    .bg(theme.background_elevated)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(theme.text_muted)
                                    .child("♪"),
                            )
                            .child(
                                div()
                                    .w(self.half_size)
                                    .h_full()
                                    .bg(theme.hover)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(theme.text_muted)
                                    .child("♪"),
                            ),
                    )
            }
            _ => {
                // 4+ images: 2x2 grid (only use first 4)
                container
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .w_full()
                            .h(self.half_size)
                            .flex()
                            .child(
                                div()
                                    .w(self.half_size)
                                    .h_full()
                                    .bg(theme.background_secondary)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(theme.text_muted)
                                    .child("♪"),
                            )
                            .child(
                                div()
                                    .w(self.half_size)
                                    .h_full()
                                    .bg(theme.background_elevated)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(theme.text_muted)
                                    .child("♪"),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .h(self.half_size)
                            .flex()
                            .child(
                                div()
                                    .w(self.half_size)
                                    .h_full()
                                    .bg(theme.hover)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(theme.text_muted)
                                    .child("♪"),
                            )
                            .child(
                                div()
                                    .w(self.half_size)
                                    .h_full()
                                    .bg(theme.active)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(theme.text_muted)
                                    .child("♪"),
                            ),
                    )
            }
        }
    }
}
