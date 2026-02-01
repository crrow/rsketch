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
    div, px, svg, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, WeakEntity, Window, prelude::FluentBuilder,
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

/// Sidebar component for navigation and playlist display.
pub struct Sidebar {
    weak_self: WeakEntity<Self>,
    app_state: AppState,
    active_nav: NavItem,
    /// Reference to the workspace for navigation
    workspace: Option<WeakEntity<crate::yunara_player::YunaraPlayer>>,
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
    pub fn set_active_nav(&mut self, nav: NavItem) {
        self.active_nav = nav;
    }

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

    /// Render a navigation item
    fn render_nav_item(
        nav: NavItem,
        icon_path: &'static str,
        label: &'static str,
        is_active: bool,
        weak_self: WeakEntity<Self>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .id(label)
            .flex()
            .items_center()
            .gap_3()
            .px(px(12.0))
            .py(px(10.0))
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
            .child(
                svg()
                    .path(icon_path)
                    .size(px(24.0))
                    .text_color(if is_active {
                        theme.text_primary
                    } else {
                        theme.text_secondary
                    }),
            )
            .child(
                div()
                    .text_color(if is_active {
                        theme.text_primary
                    } else {
                        theme.text_secondary
                    })
                    .child(label),
            )
    }
}

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let viewport_width: f32 = window.viewport_size().width.into();
        let show_playlists = viewport_width > 900.0;

        let active_nav = self.active_nav;
        let weak_self = self.weak_self.clone();

        div()
            .flex()
            .flex_col()
            .h_full()
            .bg(theme.background_secondary)
            .overflow_hidden()
            // Navigation section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .py(px(8.0))
                    .child(Self::render_nav_item(
                        NavItem::Home,
                        yunara_assets::icons::HOME,
                        "Home",
                        active_nav == NavItem::Home,
                        weak_self.clone(),
                        cx,
                    ))
                    .child(Self::render_nav_item(
                        NavItem::Explore,
                        yunara_assets::icons::EXPLORE,
                        "Explore",
                        active_nav == NavItem::Explore,
                        weak_self.clone(),
                        cx,
                    ))
                    .child(Self::render_nav_item(
                        NavItem::Library,
                        yunara_assets::icons::LIBRARY,
                        "Library",
                        active_nav == NavItem::Library,
                        weak_self,
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
