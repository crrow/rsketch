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

//! Core pane implementation for managing a single pane item.
//!
//! Simplified from multi-tab to single-view mode for this music player use
//! case.

use gpui::{Context, IntoElement, ParentElement, Render, Styled};
use yunara_ui::components::theme::ThemeExt;

use super::pane_item::PaneItemHandle;

/// A pane that displays a single content view.
///
/// Simplified from multi-tab to single-view mode for this music player use
/// case.
pub struct Pane {
    /// Current item in this pane
    current_item: Option<PaneItemHandle>,
}

impl Pane {
    /// Creates a new empty pane.
    pub fn new() -> Self { Self { current_item: None } }

    /// Navigates to a new item, replacing the current one.
    pub fn navigate_to(&mut self, item: PaneItemHandle) { self.current_item = Some(item); }

    /// Returns the current item, if any.
    pub fn current_item(&self) -> Option<&PaneItemHandle> { self.current_item.as_ref() }

    /// Returns whether this pane is empty.
    pub fn is_empty(&self) -> bool { self.current_item.is_none() }

    /// Clears the current item.
    pub fn clear(&mut self) { self.current_item = None; }
}

impl Default for Pane {
    fn default() -> Self { Self::new() }
}

impl Render for Pane {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let active_view = self.current_item().map(|item| item.view().clone());

        match active_view {
            Some(view) => gpui::div()
                .flex()
                .flex_col()
                .w_full()
                .h_full()
                .bg(theme.background_primary)
                .child(view),
            None => gpui::div()
                .flex()
                .flex_col()
                .w_full()
                .h_full()
                .bg(theme.background_primary)
                .flex()
                .items_center()
                .justify_center()
                .text_color(theme.text_secondary)
                .child("No content"),
        }
    }
}
