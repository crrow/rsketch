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

//! Core pane implementation for managing pane items.
//!
//! A Pane is a container that holds one or more PaneItems and manages
//! which one is currently active. Similar to tabs in a browser or editor.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, prelude::FluentBuilder as _, px};
use yunara_ui::components::theme::ThemeExt;

use super::pane_item::PaneItemHandle;

/// A pane that can contain multiple items with tab-like navigation.
///
/// Similar to Zed's Pane, this manages a collection of items where
/// only one is active at a time.
pub struct Pane {
    /// All items in this pane
    items: Vec<PaneItemHandle>,

    /// Index of the currently active item
    active_item_index: usize,

    /// Whether this pane currently has focus
    has_focus: bool,
}

impl Pane {
    /// Creates a new empty pane.
    pub fn new() -> Self {
        Self {
            items:             Vec::new(),
            active_item_index: 0,
            has_focus:         false,
        }
    }

    /// Adds a new item to this pane and optionally activates it.
    pub fn add_item(&mut self, item: PaneItemHandle, activate: bool) {
        self.items.push(item);
        if activate {
            self.active_item_index = self.items.len() - 1;
        }
    }

    /// Returns the currently active item, if any.
    pub fn active_item(&self) -> Option<&PaneItemHandle> { self.items.get(self.active_item_index) }

    /// Returns all items in this pane.
    pub fn items(&self) -> &[PaneItemHandle] { &self.items }

    /// Returns the index of the active item.
    pub fn active_item_index(&self) -> usize { self.active_item_index }

    /// Activates the item at the given index.
    pub fn activate_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.active_item_index = index;
        }
    }

    /// Closes the item at the given index.
    pub fn close_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);

            // Adjust active index if needed
            if self.active_item_index >= self.items.len() && !self.items.is_empty() {
                self.active_item_index = self.items.len() - 1;
            }
        }
    }

    /// Returns whether this pane has focus.
    pub fn has_focus(&self) -> bool { self.has_focus }

    /// Sets the focus state of this pane.
    pub fn set_focus(&mut self, focus: bool) { self.has_focus = focus; }

    /// Returns whether this pane is empty.
    pub fn is_empty(&self) -> bool { self.items.is_empty() }

    /// Returns the number of items in this pane.
    pub fn item_count(&self) -> usize { self.items.len() }
}

impl Default for Pane {
    fn default() -> Self { Self::new() }
}

impl Render for Pane {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let active_index = self.active_item_index;
        let active_view = self.active_item().map(|item| item.view().clone());
        let content = match active_view {
            Some(view) => gpui::div().flex_1().p_4().child(view),
            None => gpui::div()
                .flex_1()
                .p_4()
                .text_color(theme.text_secondary)
                .child("No content"),
        };

        // Only show tab bar when there are multiple items
        let show_tabs = self.items.len() > 1;

        let mut container = gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(theme.background_primary);

        if show_tabs {
            container = container.child(
                gpui::div()
                    .h(px(40.0))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(12.0))
                    .child({
                        let mut tabs = gpui::div().flex().items_center().gap_2();
                        for (index, item) in self.items.iter().enumerate() {
                            let is_active = index == active_index;
                            tabs = tabs.child(
                                gpui::div()
                                    .px(px(10.0))
                                    .py(px(6.0))
                                    .rounded(px(6.0))
                                    .text_sm()
                                    .when(is_active, |el| el.bg(theme.active))
                                    .text_color(if is_active {
                                        theme.text_primary
                                    } else {
                                        theme.text_secondary
                                    })
                                    .child(item.title().to_string()),
                            );
                        }
                        tabs
                    }),
            );
        }

        container.child(content)
    }
}
