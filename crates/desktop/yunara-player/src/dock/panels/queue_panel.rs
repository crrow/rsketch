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

//! Play queue dock panel.
//!
//! Displays the current play queue and upcoming songs.

use gpui::{AnyView, Context, IntoElement, ParentElement, Render, Styled, WeakEntity};

use crate::{app_state::AppState, dock::DockPanel};

/// Panel showing the current play queue.
pub struct QueuePanel {
    weak_self: WeakEntity<Self>,
    app_state: AppState,
}

impl QueuePanel {
    /// Creates a new queue panel.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            app_state,
        }
    }
}

impl DockPanel for QueuePanel {
    fn title(&self) -> String { "Queue".to_string() }

    fn icon(&self) -> Option<&'static str> { Some("queue_music") }

    fn to_any_view(&self) -> AnyView {
        self.weak_self
            .upgrade()
            .map(AnyView::from)
            .expect("QueuePanel view should still be alive")
    }
}

impl Render for QueuePanel {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        use yunara_ui::components::theme::ThemeExt;
        let theme = cx.theme();

        // Tab helper
        let tab = |label: &'static str, active: bool| {
            let base = gpui::div()
                .px(gpui::px(16.0))
                .py(gpui::px(8.0))
                .cursor_pointer()
                .text_sm()
                .text_color(if active {
                    theme.text_primary
                } else {
                    theme.text_muted
                });
            let styled = if active {
                base.border_b_2().border_color(theme.text_primary)
            } else {
                base
            };
            styled.child(label)
        };

        // Queue item with thumbnail and duration
        let queue_item = |thumbnail: &'static str,
                          title: &'static str,
                          artist: &'static str,
                          duration: &'static str| {
            gpui::div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px(gpui::px(12.0))
                    .py(gpui::px(8.0))
                    .rounded(gpui::px(6.0))
                    .cursor_pointer()
                    // Thumbnail
                    .child(
                        gpui::div()
                            .w(gpui::px(48.0))
                            .h(gpui::px(48.0))
                            .rounded(gpui::px(4.0))
                            .bg(theme.background_elevated)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.text_muted)
                            .text_xs()
                            .child(thumbnail),
                    )
                    // Song info (flex-1 to take available space)
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
                                    .child(title),
                            )
                            .child(
                                gpui::div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child(artist),
                            ),
                    )
                    // Duration
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(duration),
                    )
        };

        gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            // Tabs header
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .px(gpui::px(8.0))
                    .py(gpui::px(8.0))
                    .child(tab("UP NEXT", true))
                    .child(tab("LYRICS", false))
                    .child(tab("RELATED", false)),
            )
            // Queue list
            .child(
                gpui::div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .py(gpui::px(8.0))
                    .overflow_hidden()
                    .child(queue_item("♪", "ただ声一つ - One Voice", "Rokudenashi", "2:42"))
                    .child(queue_item("♪", "Runaway", "KOKO", "4:30"))
                    .child(queue_item("♪", "天球", "KOKO", "3:20"))
                    .child(queue_item("♪", "夜の合唱 - yorunoaizu", "Yorunoaizu", "4:15"))
                    .child(queue_item("♪", "冬が終わる前に", "Covered by りりぁ...", "3:58"))
                    .child(queue_item("♪", "月暈 - Lunar Eclipse", "Risa", "4:45"))
                    .child(queue_item("♪", "だから僕は音楽を辞めた", "ヨルシカ", "4:03"))
                    .child(queue_item("♪", "Kawaii Cuteness Overload", "Haruki Mori", "2:29"))
                    .child(queue_item("♪", "花かり - Withered Flower", "Risa", "5:26")),
            )
    }
}
