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

//! Dock implementation for collapsible side panels.
//!
//! Docks are collapsible panels that can be positioned on the left, right, or bottom
//! of the workspace. Similar to VS Code's or Zed's sidebar/panel system.

use gpui::{AnyView, Context, IntoElement, ParentElement, Render, Styled};
use yunara_ui::components::theme::ThemeExt;

use super::dock_position::DockPosition;

/// A panel that can be displayed in a dock.
pub trait DockPanel: Send + Sync {
    /// Returns the title of this panel.
    fn title(&self) -> String;

    /// Returns the icon name for this panel (if any).
    fn icon(&self) -> Option<&'static str> {
        None
    }

    /// Renders the panel's content.
    fn to_any_view(&self) -> AnyView;
}

/// Type-erased handle to a dock panel.
pub struct DockPanelHandle {
    title: String,
    icon: Option<&'static str>,
    view: AnyView,
}

impl DockPanelHandle {
    /// Creates a new dock panel handle.
    pub fn new(panel: &impl DockPanel) -> Self {
        Self {
            title: panel.title(),
            icon: panel.icon(),
            view: panel.to_any_view(),
        }
    }

    /// Returns the title of this panel.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the icon name for this panel.
    pub fn icon(&self) -> Option<&'static str> {
        self.icon
    }

    /// Returns the view for rendering.
    pub fn view(&self) -> &AnyView {
        &self.view
    }
}

/// A collapsible dock that contains multiple panels.
pub struct Dock {
    /// Position of this dock
    position: DockPosition,

    /// All panels in this dock
    panels: Vec<DockPanelHandle>,

    /// Index of the currently active panel
    active_panel_index: usize,

    /// Whether this dock is currently visible
    visible: bool,

    /// Width (for left/right) or height (for bottom) in pixels
    size: f32,
}

impl Dock {
    /// Creates a new dock at the specified position.
    pub fn new(position: DockPosition) -> Self {
        Self {
            position,
            panels: Vec::new(),
            active_panel_index: 0,
            visible: true,
            size: 300.0, // Default size
        }
    }

    /// Adds a panel to this dock.
    pub fn add_panel(&mut self, panel: DockPanelHandle) {
        self.panels.push(panel);
    }

    /// Returns the currently active panel, if any.
    pub fn active_panel(&self) -> Option<&DockPanelHandle> {
        self.panels.get(self.active_panel_index)
    }

    /// Returns all panels in this dock.
    pub fn panels(&self) -> &[DockPanelHandle] {
        &self.panels
    }

    /// Activates the panel at the given index.
    pub fn activate_panel(&mut self, index: usize) {
        if index < self.panels.len() {
            self.active_panel_index = index;
        }
    }

    /// Returns the position of this dock.
    pub fn position(&self) -> DockPosition {
        self.position
    }

    /// Returns whether this dock is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets the visibility of this dock.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Toggles the visibility of this dock.
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Returns the size of this dock (width for left/right, height for bottom).
    pub fn size(&self) -> f32 {
        self.size
    }

    /// Sets the size of this dock.
    pub fn set_size(&mut self, size: f32) {
        self.size = size.max(0.0);
    }

    /// Returns whether this dock is empty.
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }
}

impl Render for Dock {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let active_panel: Option<AnyView> = self.active_panel().map(|panel| panel.view().clone());

        // Render panel content directly without title bar
        match active_panel {
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
                .text_color(theme.text_secondary)
                .child("No panel available"),
        }
    }
}
