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
//! Manages the application layout:
//! - Sidebar (left): Navigation and playlists
//! - Center Pane: Main content (HomeView, ExploreView, LibraryView,
//!   PlaylistView)
//! - Right Dock: Queue panel (collapsible)
//! - Bottom Dock: Player bar (collapsible)

use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, WeakEntity,
    prelude::FluentBuilder, px,
};
use yunara_ui::components::{layout::Header, theme::ThemeExt};

use crate::{
    actions::NavigateAction,
    app_state::AppState,
    consts,
    dock::{Dock, DockPanelHandle, DockPosition, panels::QueuePanel},
    pane::{
        Pane, PaneItemHandle,
        items::{ExploreView, HomeView, LibraryView, PlaylistView},
    },
    player_bar::PlayerBar,
    sidebar::Sidebar,
};

/// Main application workspace.
pub struct YunaraPlayer {
    /// Weak reference to self for storing in closures
    weak_self: WeakEntity<Self>,

    /// Reference to the global application state
    app_state: AppState,

    /// Left sidebar for navigation
    sidebar: Entity<Sidebar>,

    /// Center pane (main content area, single view)
    center: Entity<Pane>,

    /// Right dock (queue panel)
    right_dock: Entity<Dock>,

    /// Bottom dock (player bar)
    bottom_dock: Entity<Dock>,
}

impl YunaraPlayer {
    /// Creates a new Yunara Player workspace.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        let weak_self = cx.weak_entity();

        // Create the center pane
        let center = cx.new(|_cx| Pane::new());

        // Create and add initial HomeView to center pane
        let home_view = cx.new(|cx| HomeView::new(app_state.clone(), cx));
        let home_handle = home_view.update(cx, |view, _| PaneItemHandle::new(view));
        center.update(cx, |pane, _| pane.navigate_to(home_handle));

        // Create sidebar
        let sidebar = cx.new(|cx| Sidebar::new(app_state.clone(), cx));

        // Set workspace reference on sidebar for navigation
        let weak_self_for_sidebar = weak_self.clone();
        sidebar.update(cx, |sidebar, _| {
            sidebar.set_workspace(weak_self_for_sidebar);
        });

        // Create right dock with QueuePanel
        let right_dock = cx.new(|_cx| Dock::new(DockPosition::Right));
        let queue_panel = cx.new(|cx| QueuePanel::new(app_state.clone(), cx));
        let queue_handle = queue_panel.update(cx, |panel, _| DockPanelHandle::new(panel));
        right_dock.update(cx, |dock, _| dock.add_panel(queue_handle));

        // Create bottom dock with PlayerBar
        let bottom_dock = cx.new(|_cx| {
            let mut dock = Dock::new(DockPosition::Bottom);
            dock.set_size(72.0); // PlayerBar height
            dock
        });
        let player_bar = cx.new(|cx| PlayerBar::new(app_state.clone(), cx));
        let player_handle = player_bar.update(cx, |panel, _| DockPanelHandle::new(panel));
        bottom_dock.update(cx, |dock, _| dock.add_panel(player_handle));

        // Load playlists in background on Tokio runtime, notify sidebar when done
        {
            let service = app_state.playlist_service().clone();
            let sidebar = sidebar.clone();
            let tokio_task = gpui_tokio::Tokio::spawn(cx, async move {
                service.load_playlists().await
            });
            cx.spawn(async move |_this, cx| {
                match tokio_task.await {
                    Ok(Ok(())) => {
                        let _ = cx.update(|cx| {
                            sidebar.update(cx, |_sidebar, cx| cx.notify());
                        });
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Failed to load playlists on startup: {}", e);
                    }
                    Err(e) => {
                        tracing::error!("Playlist load task panicked: {}", e);
                    }
                }
            })
            .detach();
        }

        Self {
            weak_self,
            app_state,
            sidebar,
            center,
            right_dock,
            bottom_dock,
        }
    }

    /// Handle navigation action from sidebar
    pub fn handle_navigate(&mut self, action: NavigateAction, cx: &mut Context<Self>) {
        let app_state = self.app_state.clone();

        match action {
            NavigateAction::Home => {
                let home_view = cx.new(|cx| HomeView::new(app_state, cx));
                let handle = home_view.update(cx, |view, _| PaneItemHandle::new(view));
                self.center.update(cx, |pane, _| pane.navigate_to(handle));
                self.sidebar.update(cx, |sidebar, _| {
                    sidebar.set_active_nav(crate::sidebar::NavItem::Home);
                    sidebar.set_active_playlist_id(None);
                });
            }
            NavigateAction::Explore => {
                let explore_view = cx.new(|cx| ExploreView::new(app_state, cx));
                let handle = explore_view.update(cx, |view, _| PaneItemHandle::new(view));
                self.center.update(cx, |pane, _| pane.navigate_to(handle));
                self.sidebar.update(cx, |sidebar, _| {
                    sidebar.set_active_nav(crate::sidebar::NavItem::Explore);
                    sidebar.set_active_playlist_id(None);
                });
            }
            NavigateAction::Library => {
                let library_view = cx.new(|cx| LibraryView::new(app_state, cx));
                let handle = library_view.update(cx, |view, _| PaneItemHandle::new(view));
                self.center.update(cx, |pane, _| pane.navigate_to(handle));
                self.sidebar.update(cx, |sidebar, _| {
                    sidebar.set_active_nav(crate::sidebar::NavItem::Library);
                    sidebar.set_active_playlist_id(None);
                });
            }
            NavigateAction::Playlist { id, name } => {
                let playlist_id = id.clone();
                let playlist_view = cx.new(|cx| PlaylistView::new(app_state, id, name, cx));
                let handle = playlist_view.update(cx, |view, _| PaneItemHandle::new(view));
                self.center.update(cx, |pane, _| pane.navigate_to(handle));
                self.sidebar.update(cx, |sidebar, _| {
                    sidebar.set_active_playlist_id(Some(playlist_id));
                });
            }
        }

        cx.notify();
    }
}

impl Render for YunaraPlayer {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let viewport_size = window.viewport_size();

        // Calculate aspect ratio to determine layout orientation
        let width_f32: f32 = viewport_size.width.into();
        let height_f32: f32 = viewport_size.height.into();
        let aspect_ratio = width_f32 / height_f32;
        let show_right_on_side = aspect_ratio >= consts::NARROW_LAYOUT_ASPECT_RATIO;

        // Sidebar width
        let sidebar_width = if width_f32 > 900.0 { 240.0 } else { 72.0 };

        let main_content = gpui::div()
            .flex()
            .flex_1()
            .overflow_hidden()
            // Sidebar
            .child(
                gpui::div()
                    .w(px(sidebar_width))
                    .h_full()
                    .when(cfg!(target_os = "macos"), |el| el.pt(px(28.0)))
                    .child(gpui::AnyView::from(self.sidebar.clone())),
            )
            // Center + Right (header above, content below)
            .child(
                gpui::div()
                    .flex_1()
                    .h_full()
                    .bg(theme.background_primary)
                    .when(cfg!(target_os = "macos"), |el| el.pt(px(28.0)))
                    .child(Header::new("app-header"))
                    .child(
                        gpui::div()
                            .flex_1()
                            .flex()
                            .overflow_hidden()
                            // Center pane
                            .child(
                                gpui::div()
                                    .flex_1()
                                    .h_full()
                                    .bg(theme.background_primary)
                                    .child(gpui::AnyView::from(self.center.clone())),
                            )
                            // Right dock (when showing on side)
                            .when(show_right_on_side, |div| {
                                div.child(
                                    gpui::div()
                                        .w(px(320.0))
                                        .h_full()
                                        .child(gpui::AnyView::from(self.right_dock.clone())),
                                )
                            }),
                    ),
            );

        let content = if show_right_on_side {
            // Wide layout: sidebar | center | right
            gpui::div()
                .flex_1()
                .flex()
                .overflow_hidden()
                .child(main_content)
        } else {
            // Narrow layout: (sidebar | center) / right-below
            gpui::div()
                .flex_1()
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(main_content)
                .child(
                    gpui::div()
                        .w_full()
                        .h(px(280.0))
                        .child(gpui::AnyView::from(self.right_dock.clone())),
                )
        };

        gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(theme.background_primary)
            .child(content)
            // Bottom dock (PlayerBar) - wrap in fixed height container
            .child(
                gpui::div()
                    .w_full()
                    .h(px(72.0))
                    .child(gpui::AnyView::from(self.bottom_dock.clone())),
            )
    }
}
