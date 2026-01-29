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

//! Main Yunara Player workspace.
//!
//! Similar to Zed's Workspace, this is the root view that manages:
//! - Center pane group (main content with split support)
//! - Left/right/bottom docks (collapsible panels)
//! - Active pane tracking
//! - Global app state

use gpui::{AppContext, Context, Entity, IntoElement, ParentElement, Render, WeakEntity};

use crate::{
    app_state::AppState,
    dock::{Dock, DockPosition},
    pane::{Pane, PaneGroup},
};

/// Main application workspace.
///
/// Inspired by Zed's Workspace, this manages the entire UI layout including:
/// - A central pane group that can be split horizontally or vertically
/// - Three docks (left, right, bottom) for collapsible panels
/// - Active pane tracking
/// - Reference to global application state
pub struct YunaraPlayer {
    /// Weak reference to self for storing in closures
    weak_self: WeakEntity<Self>,

    /// Reference to the global application state
    app_state: Entity<AppState>,

    /// Center pane group (main content area with split support)
    center: PaneGroup,

    /// Left dock (e.g., playlist library, folders)
    left_dock: Entity<Dock>,

    /// Right dock (e.g., lyrics, album art)
    right_dock: Entity<Dock>,

    /// Bottom dock (e.g., play queue, search results)
    bottom_dock: Entity<Dock>,

    /// All panes in the workspace
    panes: Vec<Entity<Pane>>,

    /// Currently active pane
    active_pane: Entity<Pane>,
}

impl YunaraPlayer {
    /// Creates a new Yunara Player workspace.
    ///
    /// # Arguments
    /// * `app_state` - Entity containing the application state
    /// * `cx` - GPUI context for creating entities
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        // Create the initial center pane
        let center_pane = cx.new(|_cx| Pane::new());

        // Create docks at each position
        let left_dock = cx.new(|_cx| Dock::new(DockPosition::Left));
        let right_dock = cx.new(|_cx| Dock::new(DockPosition::Right));
        let bottom_dock = cx.new(|_cx| Dock::new(DockPosition::Bottom));

        // Create the center pane group
        let center = PaneGroup::new(center_pane.clone());

        Self {
            weak_self: cx.weak_entity(),
            app_state,
            center,
            left_dock,
            right_dock,
            bottom_dock,
            panes: vec![center_pane.clone()],
            active_pane: center_pane,
        }
    }

    /// Returns a reference to the application state.
    pub fn app_state(&self) -> &Entity<AppState> {
        &self.app_state
    }

    /// Returns the center pane group.
    pub fn center(&self) -> &PaneGroup {
        &self.center
    }

    /// Returns a mutable reference to the center pane group.
    pub fn center_mut(&mut self) -> &mut PaneGroup {
        &mut self.center
    }

    /// Returns the left dock.
    pub fn left_dock(&self) -> &Entity<Dock> {
        &self.left_dock
    }

    /// Returns the right dock.
    pub fn right_dock(&self) -> &Entity<Dock> {
        &self.right_dock
    }

    /// Returns the bottom dock.
    pub fn bottom_dock(&self) -> &Entity<Dock> {
        &self.bottom_dock
    }

    /// Returns all panes in the workspace.
    pub fn panes(&self) -> &[Entity<Pane>] {
        &self.panes
    }

    /// Returns the currently active pane.
    pub fn active_pane(&self) -> &Entity<Pane> {
        &self.active_pane
    }

    /// Sets the active pane.
    pub fn set_active_pane(&mut self, pane: Entity<Pane>) {
        self.active_pane = pane;
    }
}

impl Render for YunaraPlayer {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        // TODO: Implement proper layout with:
        // - Flexbox for dock positioning
        // - Center pane group rendering
        // - Dock visibility toggles
        // - Split handles for resizing

        gpui::div()
            .child("YunaraPlayer workspace")
            .child("TODO: Implement pane/dock layout")
    }
}
