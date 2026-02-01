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

//! Sidebar component for navigation and playlist display.
//!
//! The sidebar shows navigation items (Home, Explore, Library) and
//! optionally displays the user's playlists when the window is wide enough.

use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement,
    Styled, WeakEntity, Window, div, prelude::FluentBuilder, px, svg,
};
use yunara_ui::components::theme::ThemeExt;

use crate::{actions::NavigateAction, app_state::AppState};

/// Navigation item in the sidebar
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavItem {
    Home,
    Explore,
    Library,
}

/// Layout mode for navigation items
#[derive(Debug, Clone, Copy, PartialEq)]
enum NavItemMode {
    Horizontal, // Icon and text side by side
    Compact,    // Icon on top, text below
}

/// Sidebar component for navigation and playlist display.
pub struct Sidebar {
    weak_self:  WeakEntity<Self>,
    app_state:  AppState,
    active_nav: NavItem,
    /// Reference to the workspace for navigation
    workspace:  Option<WeakEntity<crate::yunara_player::YunaraPlayer>>,
}

impl Sidebar {
    /// Creates a new sidebar.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            app_state,
            active_nav: NavItem::Home,
            workspace: None,
        }
    }

    /// Sets the workspace reference for navigation.
    pub fn set_workspace(&mut self, workspace: WeakEntity<crate::yunara_player::YunaraPlayer>) {
        self.workspace = Some(workspace);
    }

    /// Sets the active navigation item.
    pub fn set_active_nav(&mut self, nav: NavItem) { self.active_nav = nav; }

    /// Handle navigation item click
    fn handle_nav_click(&mut self, nav: NavItem, cx: &mut Context<Self>) {
        self.active_nav = nav;
        let action = match nav {
            NavItem::Home => NavigateAction::Home,
            NavItem::Explore => NavigateAction::Explore,
            NavItem::Library => NavigateAction::Library,
        };

        if let Some(ref workspace) = self.workspace {
            workspace
                .update(cx, |player, cx| {
                    player.handle_navigate(action, cx);
                })
                .ok();
        }

        cx.notify();
    }

    /// Render a navigation item with icon and label
    fn render_nav_item(
        nav: NavItem,
        icon_path: &'static str,
        icon_filled_path: &'static str,
        label: &'static str,
        is_active: bool,
        mode: NavItemMode,
        weak_self: WeakEntity<Self>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let selected_icon = if is_active {
            icon_filled_path
        } else {
            icon_path
        };

        div()
            .id(label)
            .flex()
            .items_center()
            .rounded(px(8.0))
            .cursor_pointer()
            .when(is_active, |el| el.bg(theme.active))
            .hover(|style| style.bg(theme.hover))
            .on_click(move |_event, _window, cx| {
                weak_self
                    .update(cx, |sidebar, cx| {
                        sidebar.handle_nav_click(nav, cx);
                    })
                    .ok();
            })
            // Apply mode-specific layout
            .when(mode == NavItemMode::Horizontal, |el| {
                // Horizontal: icon and text side by side
                el.flex_row()
                    .w_full()
                    .justify_start()
                    .gap_3()
                    .px(px(12.0))
                    .py(px(10.0))
            })
            .when(mode == NavItemMode::Compact, |el| {
                // Compact: icon on top, text below, vertically centered
                el.flex_col()
                    .justify_center()
                    .w(px(64.0))
                    .py(px(8.0))
                    .gap(px(4.0))
            })
            .child(
                div()
                    .w(px(24.0))
                    .h(px(24.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path(selected_icon)
                            .text_color(if is_active {
                                theme.text_primary
                            } else {
                                theme.text_secondary
                            })
                            .when(mode == NavItemMode::Horizontal, |el| el.size(px(24.0)))
                            .when(mode == NavItemMode::Compact, |el| el.size(px(20.0))),
                    ),
            )
            .child(
                div()
                    .text_color(if is_active {
                        theme.text_primary
                    } else {
                        theme.text_secondary
                    })
                    .when(mode == NavItemMode::Compact, |el| el.text_xs())
                    .child(label),
            )
    }
}

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let viewport_size = window.viewport_size();
        let viewport_width: f32 = viewport_size.width.into();
        let viewport_height: f32 = viewport_size.height.into();

        // Determine layout mode based on viewport aspect ratio
        let aspect_ratio = viewport_width / viewport_height;
        let nav_mode = if aspect_ratio >= crate::consts::NARROW_LAYOUT_ASPECT_RATIO {
            NavItemMode::Horizontal
        } else {
            NavItemMode::Compact
        };

        let show_playlists =
            aspect_ratio >= crate::consts::NARROW_LAYOUT_ASPECT_RATIO && viewport_width > 900.0;

        let active_nav = self.active_nav;
        let weak_self = self.weak_self.clone();

        div()
            .flex()
            .flex_col()
            .h_full()
            .bg(theme.background_primary)
            .overflow_hidden()
            // Navigation section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .py(px(8.0))
                    .when(show_playlists, |el| el.pb(px(8.0)))
                    .child(Self::render_nav_item(
                        NavItem::Home,
                        yunara_assets::icons::HOME,
                        yunara_assets::icons::HOME_FILLED,
                        "Home",
                        active_nav == NavItem::Home,
                        nav_mode,
                        weak_self.clone(),
                        cx,
                    ))
                    .child(Self::render_nav_item(
                        NavItem::Explore,
                        yunara_assets::icons::EXPLORE,
                        yunara_assets::icons::EXPLORE_FILLED,
                        "Explore",
                        active_nav == NavItem::Explore,
                        nav_mode,
                        weak_self.clone(),
                        cx,
                    ))
                    .child(Self::render_nav_item(
                        NavItem::Library,
                        yunara_assets::icons::LIBRARY,
                        yunara_assets::icons::LIBRARY_FILLED,
                        "Library",
                        active_nav == NavItem::Library,
                        nav_mode,
                        weak_self.clone(),
                        cx,
                    )),
            )
            // Playlists section (only when expanded)
            .when(show_playlists, |el| {
                el.child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .overflow_hidden()
                        // Divider between nav and playlists
                        .child(
                            div()
                                .h(px(1.0))
                                .bg(theme.border)
                                .mx(px(12.0))
                                .my(px(12.0)),
                        )
                        // New playlist button
                        .child(
                            div()
                                .px(px(12.0))
                                .py(px(12.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .px(px(16.0))
                                        .py(px(10.0))
                                        .rounded(px(20.0))
                                        .border_1()
                                        .border_color(theme.border)
                                        .cursor_pointer()
                                        .text_color(theme.text_primary)
                                        .text_sm()
                                        .hover(|style| style.bg(theme.hover))
                                        .child("+")
                                        .child("New playlist"),
                                ),
                        )
                        // Placeholder for playlist items
                        .child(
                            div()
                                .flex_1()
                                .px(px(12.0))
                                .text_color(theme.text_muted)
                                .text_sm()
                                .child("Playlists will appear here"),
                        ),
                )
            })
    }
}
