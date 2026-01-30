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

/// Content area component for the main content display.
///
/// A flexible container that fills the space between the sidebar and
/// queue panel, rendering whatever view is currently active.
use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement, Styled, Window, div, prelude::*, px,
};

use crate::components::theme::ThemeExt;

/// Main content area that renders the current view.
///
/// This is a flexible container that receives its content as children.
/// It handles scrolling and padding for the content.
#[derive(IntoElement)]
pub struct ContentArea {
    id:       ElementId,
    children: Vec<AnyElement>,
}

impl ContentArea {
    /// Creates a new content area.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id:       id.into(),
            children: Vec::new(),
        }
    }

    /// Adds a child element to the content area.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for ContentArea {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .id(self.id)
            .flex_1()
            .h_full()
            .bg(theme.background_primary)
            .overflow_hidden()
            .p(px(24.0))
            .children(self.children)
    }
}
