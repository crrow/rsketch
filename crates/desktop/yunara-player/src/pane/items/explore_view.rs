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

//! Explore view pane item.
//!
//! Displays discovery content organized into browsable categories such as
//! new releases, charts, and moods & genres.

use gpui::{
    AnyView, Context, EntityId, FontWeight, InteractiveElement, IntoElement, ParentElement, Render,
    Rgba, StatefulInteractiveElement, Styled, WeakEntity, px,
};
use yunara_ui::components::theme::ThemeExt;
use yunara_ui::components::theme::ThemeConfig;

use crate::{app_state::AppState, pane::PaneItem};

/// A browsable category displayed as a card in the Explore view.
struct ExploreCategory {
    title:       &'static str,
    subtitle:    &'static str,
    accent:      Rgba,
    icon_symbol: &'static str,
}

const CATEGORIES: &[ExploreCategory] = &[
    ExploreCategory {
        title:       "New Releases",
        subtitle:    "The latest albums and singles",
        accent:      Rgba { r: 0.90, g: 0.25, b: 0.30, a: 1.0 },
        icon_symbol: "NEW",
    },
    ExploreCategory {
        title:       "Charts",
        subtitle:    "Top tracks and trending now",
        accent:      Rgba { r: 0.20, g: 0.60, b: 0.95, a: 1.0 },
        icon_symbol: "TOP",
    },
    ExploreCategory {
        title:       "Moods & Genres",
        subtitle:    "Music for every moment",
        accent:      Rgba { r: 0.55, g: 0.25, b: 0.85, a: 1.0 },
        icon_symbol: "MIX",
    },
    ExploreCategory {
        title:       "Podcasts",
        subtitle:    "Popular shows and episodes",
        accent:      Rgba { r: 0.15, g: 0.75, b: 0.55, a: 1.0 },
        icon_symbol: "POD",
    },
    ExploreCategory {
        title:       "Live",
        subtitle:    "Live performances and concerts",
        accent:      Rgba { r: 0.95, g: 0.50, b: 0.15, a: 1.0 },
        icon_symbol: "LIVE",
    },
    ExploreCategory {
        title:       "Community Playlists",
        subtitle:    "Curated by listeners like you",
        accent:      Rgba { r: 0.85, g: 0.20, b: 0.60, a: 1.0 },
        icon_symbol: "USR",
    },
];

/// Mood/genre quick-pick labels.
const MOOD_LABELS: &[(&str, Rgba)] = &[
    ("Chill",       Rgba { r: 0.30, g: 0.65, b: 0.80, a: 1.0 }),
    ("Workout",     Rgba { r: 0.90, g: 0.35, b: 0.20, a: 1.0 }),
    ("Focus",       Rgba { r: 0.40, g: 0.50, b: 0.85, a: 1.0 }),
    ("Party",       Rgba { r: 0.85, g: 0.20, b: 0.55, a: 1.0 }),
    ("Sleep",       Rgba { r: 0.25, g: 0.35, b: 0.60, a: 1.0 }),
    ("Romance",     Rgba { r: 0.80, g: 0.30, b: 0.45, a: 1.0 }),
    ("Commute",     Rgba { r: 0.50, g: 0.70, b: 0.30, a: 1.0 }),
    ("Sad",         Rgba { r: 0.35, g: 0.40, b: 0.55, a: 1.0 }),
    ("Feel Good",   Rgba { r: 0.95, g: 0.65, b: 0.15, a: 1.0 }),
    ("Energize",    Rgba { r: 0.95, g: 0.45, b: 0.10, a: 1.0 }),
];

/// Explore view for discovering new music.
pub struct ExploreView {
    weak_self:  WeakEntity<Self>,
    _app_state: AppState,
}

impl ExploreView {
    /// Creates a new explore view.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self:  cx.weak_entity(),
            _app_state: app_state,
        }
    }
}

impl PaneItem for ExploreView {
    fn entity_id(&self) -> EntityId { self.weak_self.entity_id() }

    fn tab_title(&self) -> String { "Explore".to_string() }

    fn to_any_view(&self) -> AnyView {
        self.weak_self
            .upgrade()
            .map(AnyView::from)
            .expect("ExploreView should still be alive")
    }

    fn can_close(&self) -> bool { false }
}

/// Builds a single category card element.
fn render_category_card(category: &ExploreCategory, theme: &ThemeConfig) -> gpui::Div {
    let accent = category.accent;
    let accent_muted = Rgba {
        r: accent.r,
        g: accent.g,
        b: accent.b,
        a: 0.15,
    };

    gpui::div()
        .flex()
        .flex_col()
        .w(px(200.0))
        .h(px(160.0))
        .rounded(px(12.0))
        .bg(theme.background_elevated)
        .border_1()
        .border_color(theme.border)
        .overflow_hidden()
        .cursor_pointer()
        // Accent bar at the top of the card
        .child(
            gpui::div()
                .w_full()
                .h(px(4.0))
                .bg(accent),
        )
        .child(
            gpui::div()
                .flex_1()
                .flex()
                .flex_col()
                .p(px(14.0))
                .gap(px(8.0))
                // Icon badge
                .child(
                    gpui::div()
                        .w(px(40.0))
                        .h(px(40.0))
                        .rounded(px(8.0))
                        .bg(accent_muted)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(accent)
                        .child(category.icon_symbol),
                )
                // Title
                .child(
                    gpui::div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child(category.title),
                )
                // Subtitle
                .child(
                    gpui::div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(category.subtitle),
                ),
        )
}

/// Builds a mood/genre pill element.
fn render_mood_pill(label: &str, color: Rgba, theme: &ThemeConfig) -> gpui::Div {
    let pill_bg = Rgba {
        r: color.r,
        g: color.g,
        b: color.b,
        a: 0.12,
    };

    gpui::div()
        .px(px(16.0))
        .py(px(8.0))
        .rounded(px(20.0))
        .bg(pill_bg)
        .border_1()
        .border_color(Rgba {
            r: color.r,
            g: color.g,
            b: color.b,
            a: 0.25,
        })
        .cursor_pointer()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.text_primary)
        .child(label.to_string())
}

impl Render for ExploreView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        gpui::div()
            .id("explore-view")
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .overflow_y_scroll()
            .p(px(24.0))
            .gap(px(28.0))
            // Header
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        gpui::div()
                            .text_3xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child("Explore"),
                    )
                    .child(
                        gpui::div()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .child("Discover new music, trending tracks, and curated collections."),
                    ),
            )
            // Browse categories section
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .gap(px(14.0))
                    .child(
                        gpui::div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text_primary)
                            .child("Browse"),
                    )
                    .child(
                        gpui::div()
                            .flex()
                            .flex_wrap()
                            .gap(px(12.0))
                            .children(
                                CATEGORIES.iter().map(|cat| render_category_card(cat, theme)),
                            ),
                    ),
            )
            // Moods & Genres quick picks
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .gap(px(14.0))
                    .child(
                        gpui::div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text_primary)
                            .child("Moods & Genres"),
                    )
                    .child(
                        gpui::div()
                            .flex()
                            .flex_wrap()
                            .gap(px(8.0))
                            .children(
                                MOOD_LABELS
                                    .iter()
                                    .map(|(label, color)| render_mood_pill(label, *color, theme)),
                            ),
                    ),
            )
    }
}
