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

use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, WeakEntity, px,
};
use yunara_ui::components::{layout::Header, theme::ThemeExt};

use crate::{
    app_state::AppState,
    dock::{
        Dock, DockPanelHandle, DockPosition,
        panels::{LibraryPanel, QueuePanel},
    },
    pane::{Pane, PaneGroup, PaneItemHandle, items::HomeView},
    player_bar::PlayerBar,
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
    app_state: AppState,

    /// Center pane group (main content area with split support)
    center: PaneGroup,

    /// Left dock (e.g., playlist library, folders)
    left_dock: Entity<Dock>,

    /// Right dock (e.g., lyrics, album art)
    right_dock: Entity<Dock>,

    /// Player bar (bottom controls)
    player_bar: Entity<PlayerBar>,

    /// All panes in the workspace
    panes: Vec<Entity<Pane>>,

    /// Currently active pane
    active_pane: Entity<Pane>,
}

impl YunaraPlayer {
    /// Creates a new Yunara Player workspace.
    ///
    /// # Arguments
    /// * `app_state` - Arc containing the application state
    /// * `cx` - GPUI context for creating entities
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        // Create the initial center pane
        let center_pane = cx.new(|_cx| Pane::new());

        // Create docks at each position
        let left_dock = cx.new(|_cx| Dock::new(DockPosition::Left));
        let right_dock = cx.new(|_cx| Dock::new(DockPosition::Right));

        let home_view = cx.new(|cx| HomeView::new(app_state.clone(), cx));
        let home_handle = home_view.update(cx, |view, _| PaneItemHandle::new(view));

        center_pane.update(cx, |pane, _| pane.add_item(home_handle, true));

        let library_panel = cx.new(|cx| LibraryPanel::new(app_state.clone(), cx));
        let library_handle = library_panel.update(cx, |panel, _| DockPanelHandle::new(panel));
        left_dock.update(cx, |dock, _| dock.add_panel(library_handle));

        let queue_panel = cx.new(|cx| QueuePanel::new(app_state.clone(), cx));
        let queue_handle = queue_panel.update(cx, |panel, _| DockPanelHandle::new(panel));
        right_dock.update(cx, |dock, _| dock.add_panel(queue_handle));

        // Create the center pane group
        let center = PaneGroup::new(center_pane.clone());

        // Create player bar with app_state reference
        let player_bar = cx.new(|cx| PlayerBar::new(app_state.clone(), cx));

        Self {
            weak_self: cx.weak_entity(),
            app_state,
            center,
            left_dock,
            right_dock,
            player_bar,
            panes: vec![center_pane.clone()],
            active_pane: center_pane,
        }
    }

    /// Returns a reference to the application state.
    pub fn app_state(&self) -> AppState { self.app_state.clone() }

    /// Returns the center pane group.
    pub fn center(&self) -> &PaneGroup { &self.center }

    /// Returns a mutable reference to the center pane group.
    pub fn center_mut(&mut self) -> &mut PaneGroup { &mut self.center }

    /// Returns the left dock.
    pub fn left_dock(&self) -> &Entity<Dock> { &self.left_dock }

    /// Returns the right dock.
    pub fn right_dock(&self) -> &Entity<Dock> { &self.right_dock }

    /// Returns all panes in the workspace.
    pub fn panes(&self) -> &[Entity<Pane>] { &self.panes }

    /// Returns the currently active pane.
    pub fn active_pane(&self) -> &Entity<Pane> { &self.active_pane }

    /// Sets the active pane.
    pub fn set_active_pane(&mut self, pane: Entity<Pane>) { self.active_pane = pane; }
}

impl Render for YunaraPlayer {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = _cx.theme();

        let header = Header::new("app-header").logo(yunara_assets::icons::LOGO_DARK);

        let content = gpui::div()
            .flex_1()
            .flex()
            .overflow_hidden()
            .child(
                gpui::div()
                    .w(px(240.0))
                    .h_full()
                    .child(gpui::AnyView::from(self.left_dock.clone())),
            )
            .child(
                gpui::div()
                    .flex_1()
                    .h_full()
                    .bg(theme.background_primary)
                    .child(self.center.render_element()),
            )
            .child(
                gpui::div()
                    .w(px(320.0))
                    .h_full()
                    .child(gpui::AnyView::from(self.right_dock.clone())),
            );

        gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(theme.background_primary)
            .child(header)
            .child(content)
            .child(gpui::AnyView::from(self.player_bar.clone()))
    }
}
