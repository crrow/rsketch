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
    AppContext, ClickEvent, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, StatefulInteractiveElement, Styled, WeakEntity, img, px,
};
use yunara_ui::components::theme::ThemeExt;

use crate::{
    app_state::AppState,
    dock::{
        Dock, DockPanelHandle, DockPosition,
        panels::{LibraryPanel, QueuePanel},
    },
    pane::{Pane, PaneGroup, PaneItemHandle, items::HomeView},
    state::PlayerState,
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

    /// Player state (playback, volume, queue, etc.)
    player_state: PlayerState,

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
    /// * `app_state` - Arc containing the application state
    /// * `cx` - GPUI context for creating entities
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        // Create the initial center pane
        let center_pane = cx.new(|_cx| Pane::new());

        // Create docks at each position
        let left_dock = cx.new(|_cx| Dock::new(DockPosition::Left));
        let right_dock = cx.new(|_cx| Dock::new(DockPosition::Right));
        let bottom_dock = cx.new(|_cx| Dock::new(DockPosition::Bottom));

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

        Self {
            weak_self: cx.weak_entity(),
            app_state,
            player_state: PlayerState::new(),
            center,
            left_dock,
            right_dock,
            bottom_dock,
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

    /// Returns the bottom dock.
    pub fn bottom_dock(&self) -> &Entity<Dock> { &self.bottom_dock }

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

        let control_button = |path: &str, size: f32| {
            gpui::div()
                .w(px(size))
                .h(px(size))
                .rounded_full()
                .bg(theme.text_primary)
                .flex()
                .items_center()
                .justify_center()
                .child(img(path).w(px(size * 0.5)).h(px(size * 0.5)))
        };

        let header = gpui::div()
            .h(px(56.0))
            .px(px(16.0))
            .flex()
            .items_center()
            .gap_4()
            .bg(theme.background_primary)
            .border_b_1()
            .border_color(theme.border)
            // Hamburger menu
            .child(
                gpui::div()
                    .text_color(theme.text_primary)
                    .text_xl()
                    .cursor_pointer()
                    .child("≡"),
            )
            // Logo
            .child(
                img(yunara_assets::icons::LOGO_DARK)
                    .w(px(77.0))
                    .h(px(26.0)),
            )
            .child(
                gpui::div().flex_1().flex().justify_center().child(
                    gpui::div()
                        .w(px(420.0))
                        .h(px(30.0))
                        .rounded(px(16.0))
                        .bg(theme.background_elevated)
                        .text_color(theme.text_muted)
                        .px(px(12.0))
                        .flex()
                        .items_center()
                        .child("Search songs, albums, artists, podcasts"),
                ),
            )
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .text_color(theme.text_secondary)
                    .child("←")
                    .child("→")
                    .child("🙂"),
            );

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

        let player_bar = gpui::div()
            .h(px(90.0))
            .flex()
            .flex_col()
            .bg(theme.background_secondary)
            // Progress bar at top of player bar
            .child(
                gpui::div()
                    .w_full()
                    .h(px(4.0))
                    .bg(theme.progress_track)
                    .cursor_pointer()
                    .child(
                        gpui::div()
                            .w(px(180.0)) // ~25% progress (placeholder)
                            .h(px(4.0))
                            .bg(theme.progress_fill),
                    ),
            )
            // Main player bar content
            .child(
                gpui::div()
                    .flex_1()
                    .px(px(16.0))
                    .flex()
                    .items_center()
                    // Left: Album art + song info
                    .child(
                        gpui::div()
                            .w(px(200.0))
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                gpui::div()
                                    .w(px(56.0))
                                    .h(px(56.0))
                                    .rounded(px(4.0))
                                    .bg(theme.background_elevated)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(theme.text_muted)
                                    .child("♪"),
                            )
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .overflow_hidden()
                                    .child(
                                        gpui::div()
                                            .text_color(theme.text_primary)
                                            .text_sm()
                                            .child("逆転劇 - Gyakutengeki"),
                                    )
                                    .child(
                                        gpui::div()
                                            .text_xs()
                                            .text_color(theme.text_secondary)
                                            .child("Tsukuyomi"),
                                    ),
                            ),
                    )
                    // Center: Playback controls + time
                    .child(
                        gpui::div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            // Controls row
                            .child(
                                gpui::div()
                                    .flex()
                                    .items_center()
                                    .gap_4()
                                    .child(control_button("icons/media-icons-black/previous.png", 32.0))
                                    .child(control_button("icons/media-icons-black/play.png", 40.0))
                                    .child(control_button("icons/media-icons-black/next.png", 32.0)),
                            )
                            // Time display
                            .child(
                                gpui::div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child("1:01")
                                    .child("/")
                                    .child("4:03"),
                            ),
                    )
                    // Right: Volume control (slider appears on hover)
                    .child(
                        gpui::div()
                            .id("volume-control")
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_2()
                            .group("volume")
                            // Volume slider - hidden by default, shown on hover
                            .child({
                                let effective_volume = self.player_state.volume.effective_volume();
                                let slider_fill_width = effective_volume * 100.0;
                                gpui::div()
                                    .invisible()
                                    .group_hover("volume", |style| style.visible())
                                    .flex()
                                    .items_center()
                                    .child(
                                        gpui::div()
                                            .w(px(100.0))
                                            .h(px(4.0))
                                            .rounded_full()
                                            .bg(theme.progress_track)
                                            .cursor_pointer()
                                            .child(
                                                gpui::div()
                                                    .w(px(slider_fill_width))
                                                    .h(px(4.0))
                                                    .rounded_full()
                                                    .bg(theme.text_primary),
                                            ),
                                    )
                            })
                            // Volume icon - clickable for mute
                            .child({
                                let is_muted = self.player_state.volume.is_muted;
                                let icon_path = if is_muted {
                                    yunara_assets::icons::VOLUME_MUTED
                                } else {
                                    yunara_assets::icons::VOLUME
                                };
                                gpui::div()
                                    .id("volume-icon")
                                    .w(px(36.0))
                                    .h(px(36.0))
                                    .rounded_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .hover(|style| style.bg(theme.hover))
                                    .on_click(_cx.listener(|this, _event: &ClickEvent, _window, cx| {
                                        this.player_state.volume.toggle_mute();
                                        cx.notify();
                                    }))
                                    .child(
                                        gpui::svg()
                                            .path(icon_path)
                                            .size(px(20.0))
                                            .text_color(theme.text_secondary),
                                    )
                            }),
                    ),
            );

        gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(theme.background_primary)
            .child(header)
            .child(content)
            .child(player_bar)
    }
}
