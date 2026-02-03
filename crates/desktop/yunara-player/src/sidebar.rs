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

use std::time::{Duration, Instant};

use gpui::{
    Context, ElementId, InteractiveElement, IntoElement, ParentElement, Render, Rgba,
    ScrollHandle, StatefulInteractiveElement, Styled, WeakEntity, Window, div, img,
    prelude::FluentBuilder, px, svg,
};
use ytmapi_rs::common::YoutubeID;
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
    active_playlist_id: Option<String>,
    playlist_scrollbar_visible: bool,
    playlist_scroll_animating: bool,
    playlist_scroll_last_at: Option<Instant>,
    playlist_thumb_top: f32,
    playlist_thumb_height: f32,
    playlist_scroll_handle: ScrollHandle,
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
            active_playlist_id: None,
            playlist_scrollbar_visible: false,
            playlist_scroll_animating: false,
            playlist_scroll_last_at: None,
            playlist_thumb_top: 0.0,
            playlist_thumb_height: 0.0,
            playlist_scroll_handle: ScrollHandle::new(),
            workspace: None,
        }
    }

    /// Sets the workspace reference for navigation.
    pub fn set_workspace(&mut self, workspace: WeakEntity<crate::yunara_player::YunaraPlayer>) {
        self.workspace = Some(workspace);
    }

    /// Sets the active navigation item.
    pub fn set_active_nav(&mut self, nav: NavItem) { self.active_nav = nav; }

    pub fn set_active_playlist_id(&mut self, playlist_id: Option<String>) {
        self.active_playlist_id = playlist_id;
    }

    fn note_playlist_scroll(&mut self, cx: &mut Context<Self>) {
        self.playlist_scrollbar_visible = true;
        self.playlist_scroll_last_at = Some(Instant::now());
        cx.notify();

        if self.playlist_scroll_animating {
            return;
        }

        self.playlist_scroll_animating = true;
        cx.spawn(async move |this, cx| {
            let tick = Duration::from_millis(16);
            loop {
                cx.background_executor().timer(tick).await;

                let mut should_stop = false;
                let _ = this.update(cx, |sidebar, cx| {
                    let bounds = sidebar.playlist_scroll_handle.bounds();
                    let max_offset = sidebar.playlist_scroll_handle.max_offset();
                    let viewport_height = f32::from(bounds.size.height);
                    let content_height = f32::from(max_offset.height + bounds.size.height);

                    if viewport_height <= 0.0 || content_height <= viewport_height {
                        sidebar.playlist_scrollbar_visible = false;
                        sidebar.playlist_scroll_animating = false;
                        sidebar.playlist_thumb_height = 0.0;
                        sidebar.playlist_thumb_top = 0.0;
                        should_stop = true;
                        cx.notify();
                        return;
                    }

                    let thumb_height = viewport_height / 3.0;
                    let track_height = (viewport_height - thumb_height).max(0.0);
                    let scroll_offset =
                        -f32::from(sidebar.playlist_scroll_handle.offset().y);
                    let max_offset_y = f32::from(max_offset.height);
                    let scroll_ratio = if max_offset_y > 0.0 {
                        (scroll_offset / max_offset_y).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let target_top = scroll_ratio * track_height;
                    let current_top = sidebar.playlist_thumb_top;
                    let new_top = current_top + (target_top - current_top) * 0.2;

                    sidebar.playlist_thumb_height = thumb_height;
                    sidebar.playlist_thumb_top = new_top;
                    sidebar.playlist_scrollbar_visible = true;

                    let idle = sidebar
                        .playlist_scroll_last_at
                        .map(|t| t.elapsed() > Duration::from_millis(120))
                        .unwrap_or(false);
                    if idle && (target_top - new_top).abs() < 0.5 {
                        sidebar.playlist_scrollbar_visible = false;
                        sidebar.playlist_scroll_animating = false;
                        should_stop = true;
                    }

                    cx.notify();
                });

                if should_stop {
                    break;
                }
            }
        })
        .detach();
    }

    /// Handle navigation item click
    fn handle_nav_click(&mut self, nav: NavItem, cx: &mut Context<Self>) {
        let action = match nav {
            NavItem::Home => NavigateAction::Home,
            NavItem::Explore => NavigateAction::Explore,
            NavItem::Library => NavigateAction::Library,
        };

        let workspace = self.workspace.clone();
        cx.spawn(async move |_this, cx| {
            if let Some(workspace) = workspace {
                let _ = workspace.update(cx, |player, cx| {
                    player.handle_navigate(action, cx);
                });
            }
        })
        .detach();
    }

    /// Handle playlist item click
    fn handle_playlist_click(&mut self, id: String, name: String, cx: &mut Context<Self>) {
        let action = NavigateAction::Playlist { id, name };
        let workspace = self.workspace.clone();

        cx.spawn(async move |_this, cx| {
            if let Some(workspace) = workspace {
                let _ = workspace.update(cx, |player, cx| {
                    player.handle_navigate(action, cx);
                });
            }
        })
        .detach();
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
            .min_h(px(0.0))
            .bg(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            })
            .overflow_hidden()
            // Brand header (menu + logo)
            .child(
                div()
                    .h(px(56.0))
                    .flex()
                    .items_center()
                    .gap_4()
                    .px(px(12.0))
                    .child(
                        div()
                            .w(px(24.0))
                            .h(px(24.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .child(
                                svg()
                                    .path(yunara_assets::icons::MENU)
                                    .size(px(22.0))
                                    .text_color(theme.text_primary),
                            ),
                    )
                    .child(img(yunara_assets::icons::LOGO_DARK).w(px(77.0)).h(px(26.0))),
            )
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
                let playlists = self.app_state.playlist_service().get_playlists();
                let weak = self.weak_self.clone();
                let selected_playlist_id = self.active_playlist_id.clone();
                let show_scrollbar =
                    self.playlist_scrollbar_visible && self.playlist_thumb_height > 0.0;

                el.child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_h(px(0.0))
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
                        // Playlist items
                        .child(
                            div()
                                .flex_1()
                                .min_h(px(0.0))
                                .h_full()
                                .child(
                                    div()
                                        .id("playlist-list")
                                        .flex_1()
                                        .min_h(px(0.0))
                                        .h_full()
                                        .overflow_y_scroll()
                                        .track_scroll(&self.playlist_scroll_handle)
                                        .on_scroll_wheel(cx.listener(
                                            |sidebar, _event, _window, cx| {
                                                sidebar.note_playlist_scroll(cx);
                                            },
                                        ))
                                        .px(px(12.0))
                                        .when(playlists.is_empty(), |el| {
                                            el.child(
                                                div()
                                                    .text_color(theme.text_muted)
                                                    .text_sm()
                                                    .child("No playlists yet"),
                                            )
                                        })
                                        .children(playlists.into_iter().enumerate().map(
                                            |(idx, playlist)| {
                                        let playlist_id =
                                            playlist.playlist_id.get_raw().to_owned();
                                        let playlist_name = playlist.title.clone();
                                        let is_selected = selected_playlist_id
                                            .as_ref()
                                            .map(|selected| selected == &playlist_id)
                                            .unwrap_or(false);
                                        let thumbnail_url = playlist
                                            .thumbnails
                                            .iter()
                                            .max_by_key(|thumbnail| {
                                                thumbnail.width.saturating_mul(thumbnail.height)
                                            })
                                            .map(|thumbnail| thumbnail.url.clone());
                                        let has_thumbnail = thumbnail_url.is_some();
                                        let weak = weak.clone();
                                        let count_text = playlist
                                            .count
                                            .map(|c| format!("{} songs", c))
                                            .unwrap_or_default();

                                        div()
                                            .id(ElementId::Integer(idx as u64))
                                            .flex()
                                            .items_center()
                                            .gap_3()
                                            .px(px(12.0))
                                            .py(px(8.0))
                                            .rounded(px(8.0))
                                            .cursor_pointer()
                                            .when(is_selected, |el| el.bg(theme.active))
                                            .hover(|style| style.bg(theme.hover))
                                            .on_click(move |_event, _window, cx| {
                                                let id = playlist_id.clone();
                                                let name = playlist_name.clone();
                                                weak.update(cx, |sidebar, cx| {
                                                    sidebar.handle_playlist_click(id, name, cx);
                                                })
                                                .ok();
                                            })
                                            .child(
                                                div()
                                                    .w(px(36.0))
                                                    .h(px(36.0))
                                                    .rounded(px(6.0))
                                                    .bg(theme.background_elevated)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .text_color(theme.text_muted)
                                                    .when_some(thumbnail_url, |el, url| {
                                                        el.child(
                                                            img(url)
                                                                .w(px(36.0))
                                                                .h(px(36.0))
                                                                .rounded(px(6.0)),
                                                        )
                                                    })
                                                    .when(!has_thumbnail, |el| el.child("♪")),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .overflow_hidden()
                                                    .child(
                                                        div()
                                                            .text_size(px(15.0))
                                                            .text_color(theme.text_primary)
                                                            .overflow_hidden()
                                                            .child(playlist.title),
                                                    )
                                                    .when(!count_text.is_empty(), |el| {
                                                        el.child(
                                                            div()
                                                                .text_size(px(12.0))
                                                                .text_color(theme.text_muted)
                                                                .child(count_text),
                                                        )
                                                    }),
                                            )
                                        },
                                        )),
                                )
                                .when(show_scrollbar, |el| {
                                    el.child(
                                        div()
                                            .absolute()
                                            .top(px(self.playlist_thumb_top))
                                            .right_0()
                                            .h(px(self.playlist_thumb_height))
                                            .w(px(6.0))
                                            .rounded(px(6.0))
                                            .bg(theme.active)
                                            .opacity(1.0),
                                    )
                                }),
                        ),
                )
            })
    }
}
