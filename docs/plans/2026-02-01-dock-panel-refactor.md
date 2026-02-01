# Dock/Panel 架构重构实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 重构 yunara-player 布局架构，将 PlayerBar 纳入底部 Dock，左侧改为固定 Sidebar 组件。

**Architecture:** 左侧使用 Sidebar 组件（非 Dock）控制中心 Pane 内容切换；底部使用 Dock 包含 PlayerBar；中心 Pane 简化为单视图替换模式。

**Tech Stack:** Rust, GPUI, yunara-ui

---

## Task 1: 定义 NavigateAction

**Files:**
- Create: `crates/desktop/yunara-player/src/actions.rs`
- Modify: `crates/desktop/yunara-player/src/lib.rs`

**Step 1: 创建 actions.rs 文件**

```rust
// crates/desktop/yunara-player/src/actions.rs

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

//! Application actions for navigation and state changes.

/// Navigation actions for switching views in the center pane.
#[derive(Debug, Clone, PartialEq)]
pub enum NavigateAction {
    /// Navigate to the home view
    Home,
    /// Navigate to the explore view
    Explore,
    /// Navigate to the library view
    Library,
    /// Navigate to a specific playlist
    Playlist { id: String, name: String },
}
```

**Step 2: 在 lib.rs 中添加模块导出**

在 `crates/desktop/yunara-player/src/lib.rs` 的模块声明部分添加：

```rust
pub mod actions;
```

在 pub use 部分添加：

```rust
pub use actions::NavigateAction;
```

**Step 3: 验证编译**

Run: `cargo check -p yunara-player`
Expected: 编译成功

**Step 4: 提交**

```bash
git add crates/desktop/yunara-player/src/actions.rs crates/desktop/yunara-player/src/lib.rs
git commit -m "feat(yunara): add NavigateAction for view switching"
```

---

## Task 2: 简化 Pane 为单视图模式

**Files:**
- Modify: `crates/desktop/yunara-player/src/pane/pane.rs`

**Step 1: 重构 Pane 结构体**

将 `crates/desktop/yunara-player/src/pane/pane.rs` 中的 Pane 结构简化：

```rust
/// A pane that displays a single content view.
///
/// Simplified from multi-tab to single-view mode for this music player use case.
pub struct Pane {
    /// Current item in this pane
    current_item: Option<PaneItemHandle>,
}

impl Pane {
    /// Creates a new empty pane.
    pub fn new() -> Self {
        Self { current_item: None }
    }

    /// Navigates to a new item, replacing the current one.
    pub fn navigate_to(&mut self, item: PaneItemHandle) {
        self.current_item = Some(item);
    }

    /// Returns the current item, if any.
    pub fn current_item(&self) -> Option<&PaneItemHandle> {
        self.current_item.as_ref()
    }

    /// Returns whether this pane is empty.
    pub fn is_empty(&self) -> bool {
        self.current_item.is_none()
    }

    /// Clears the current item.
    pub fn clear(&mut self) {
        self.current_item = None;
    }
}

impl Default for Pane {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for Pane {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let active_view = self.current_item().map(|item| item.view().clone());

        match active_view {
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
                .flex()
                .items_center()
                .justify_center()
                .text_color(theme.text_secondary)
                .child("No content"),
        }
    }
}
```

**Step 2: 验证编译**

Run: `cargo check -p yunara-player`
Expected: 编译成功（可能有一些警告关于未使用的方法，稍后处理）

**Step 3: 提交**

```bash
git add crates/desktop/yunara-player/src/pane/pane.rs
git commit -m "refactor(yunara): simplify Pane to single-view mode"
```

---

## Task 3: PlayerBar 实现 DockPanel trait

**Files:**
- Modify: `crates/desktop/yunara-player/src/player_bar.rs`

**Step 1: 添加 DockPanel 实现**

在 `player_bar.rs` 文件末尾添加：

```rust
use crate::dock::DockPanel;

impl DockPanel for PlayerBar {
    fn title(&self) -> String {
        "Player".to_string()
    }

    fn icon(&self) -> Option<&'static str> {
        None
    }

    fn to_any_view(&self) -> gpui::AnyView {
        self.weak_self
            .upgrade()
            .map(gpui::AnyView::from)
            .expect("PlayerBar view should still be alive")
    }
}
```

**Step 2: 验证编译**

Run: `cargo check -p yunara-player`
Expected: 编译成功

**Step 3: 提交**

```bash
git add crates/desktop/yunara-player/src/player_bar.rs
git commit -m "feat(yunara): implement DockPanel trait for PlayerBar"
```

---

## Task 4: 创建 Sidebar 组件

**Files:**
- Create: `crates/desktop/yunara-player/src/sidebar.rs`
- Modify: `crates/desktop/yunara-player/src/lib.rs`

**Step 1: 创建 sidebar.rs**

```rust
// crates/desktop/yunara-player/src/sidebar.rs

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
    /// Callback when a navigation item is clicked
    on_navigate: Option<Box<dyn Fn(NavigateAction) + Send + Sync>>,
}

impl Sidebar {
    /// Creates a new sidebar.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            app_state,
            active_nav: NavItem::Home,
            on_navigate: None,
        }
    }

    /// Sets the navigation callback.
    pub fn on_navigate(mut self, callback: impl Fn(NavigateAction) + Send + Sync + 'static) -> Self {
        self.on_navigate = Some(Box::new(callback));
        self
    }

    /// Sets the active navigation item.
    pub fn set_active_nav(&mut self, nav: NavItem) {
        self.active_nav = nav;
    }

    /// Handle navigation item click
    fn handle_nav_click(&mut self, nav: NavItem, cx: &mut Context<Self>) {
        self.active_nav = nav;
        if let Some(ref callback) = self.on_navigate {
            let action = match nav {
                NavItem::Home => NavigateAction::Home,
                NavItem::Explore => NavigateAction::Explore,
                NavItem::Library => NavigateAction::Library,
            };
            callback(action);
        }
        cx.notify();
    }

    /// Handle playlist click
    fn handle_playlist_click(&self, id: String, name: String) {
        if let Some(ref callback) = self.on_navigate {
            callback(NavigateAction::Playlist { id, name });
        }
    }

    /// Render a navigation item
    fn render_nav_item(
        &self,
        nav: NavItem,
        icon_path: &'static str,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let is_active = self.active_nav == nav;
        let weak_self = self.weak_self.clone();

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
                    .child(self.render_nav_item(
                        NavItem::Home,
                        yunara_assets::icons::HOME,
                        "Home",
                        cx,
                    ))
                    .child(self.render_nav_item(
                        NavItem::Explore,
                        yunara_assets::icons::EXPLORE,
                        "Explore",
                        cx,
                    ))
                    .child(self.render_nav_item(
                        NavItem::Library,
                        yunara_assets::icons::LIBRARY,
                        "Library",
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
```

**Step 2: 在 lib.rs 添加模块**

```rust
pub mod sidebar;
pub use sidebar::Sidebar;
```

**Step 3: 验证编译**

Run: `cargo check -p yunara-player`
Expected: 编译成功

**Step 4: 提交**

```bash
git add crates/desktop/yunara-player/src/sidebar.rs crates/desktop/yunara-player/src/lib.rs
git commit -m "feat(yunara): add Sidebar component for navigation"
```

---

## Task 5: 创建 ExploreView 和 LibraryView

**Files:**
- Create: `crates/desktop/yunara-player/src/pane/items/explore_view.rs`
- Create: `crates/desktop/yunara-player/src/pane/items/library_view.rs`
- Modify: `crates/desktop/yunara-player/src/pane/items/mod.rs`

**Step 1: 创建 explore_view.rs**

```rust
// crates/desktop/yunara-player/src/pane/items/explore_view.rs

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

//! Explore view pane item.
//!
//! Displays discovery content, trending music, and recommendations.

use gpui::{
    div, px, AnyView, Context, EntityId, IntoElement, ParentElement, Render, Styled, WeakEntity,
};
use yunara_ui::components::theme::ThemeExt;

use crate::{app_state::AppState, pane::PaneItem};

/// Explore view for discovering new music.
pub struct ExploreView {
    weak_self: WeakEntity<Self>,
    app_state: AppState,
}

impl ExploreView {
    /// Creates a new explore view.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            app_state,
        }
    }
}

impl PaneItem for ExploreView {
    fn entity_id(&self) -> EntityId {
        self.weak_self.entity_id()
    }

    fn tab_title(&self) -> String {
        "Explore".to_string()
    }

    fn to_any_view(&self) -> AnyView {
        self.weak_self
            .upgrade()
            .map(AnyView::from)
            .expect("ExploreView should still be alive")
    }

    fn can_close(&self) -> bool {
        false
    }
}

impl Render for ExploreView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p_4()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .child("Explore"),
            )
            .child(
                div()
                    .text_color(theme.text_secondary)
                    .child("Discover new music, trending tracks, and personalized recommendations."),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.text_muted)
                    .child("Content coming soon..."),
            )
    }
}
```

**Step 2: 创建 library_view.rs**

```rust
// crates/desktop/yunara-player/src/pane/items/library_view.rs

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

//! Library view pane item.
//!
//! Displays the user's music library including songs, albums, artists, and playlists.

use gpui::{
    div, px, AnyView, Context, EntityId, IntoElement, ParentElement, Render, Styled, WeakEntity,
};
use yunara_ui::components::theme::ThemeExt;

use crate::{app_state::AppState, pane::PaneItem};

/// Library view for browsing user's music collection.
pub struct LibraryView {
    weak_self: WeakEntity<Self>,
    app_state: AppState,
}

impl LibraryView {
    /// Creates a new library view.
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        Self {
            weak_self: cx.weak_entity(),
            app_state,
        }
    }
}

impl PaneItem for LibraryView {
    fn entity_id(&self) -> EntityId {
        self.weak_self.entity_id()
    }

    fn tab_title(&self) -> String {
        "Library".to_string()
    }

    fn to_any_view(&self) -> AnyView {
        self.weak_self
            .upgrade()
            .map(AnyView::from)
            .expect("LibraryView should still be alive")
    }

    fn can_close(&self) -> bool {
        false
    }
}

impl Render for LibraryView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p_4()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .child("Your Library"),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        div()
                            .px(px(16.0))
                            .py(px(8.0))
                            .rounded(px(20.0))
                            .bg(theme.text_primary)
                            .text_color(theme.background_primary)
                            .text_sm()
                            .cursor_pointer()
                            .child("Playlists"),
                    )
                    .child(
                        div()
                            .px(px(16.0))
                            .py(px(8.0))
                            .rounded(px(20.0))
                            .bg(theme.background_elevated)
                            .text_color(theme.text_secondary)
                            .text_sm()
                            .cursor_pointer()
                            .child("Albums"),
                    )
                    .child(
                        div()
                            .px(px(16.0))
                            .py(px(8.0))
                            .rounded(px(20.0))
                            .bg(theme.background_elevated)
                            .text_color(theme.text_secondary)
                            .text_sm()
                            .cursor_pointer()
                            .child("Artists"),
                    )
                    .child(
                        div()
                            .px(px(16.0))
                            .py(px(8.0))
                            .rounded(px(20.0))
                            .bg(theme.background_elevated)
                            .text_color(theme.text_secondary)
                            .text_sm()
                            .cursor_pointer()
                            .child("Songs"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.text_muted)
                    .child("Library content coming soon..."),
            )
    }
}
```

**Step 3: 更新 pane/items/mod.rs**

```rust
// crates/desktop/yunara-player/src/pane/items/mod.rs

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

//! Pane item implementations.

mod explore_view;
mod home_view;
mod library_view;
mod playlist_view;

pub use explore_view::ExploreView;
pub use home_view::HomeView;
pub use library_view::LibraryView;
pub use playlist_view::PlaylistView;
```

**Step 4: 验证编译**

Run: `cargo check -p yunara-player`
Expected: 编译成功

**Step 5: 提交**

```bash
git add crates/desktop/yunara-player/src/pane/items/
git commit -m "feat(yunara): add ExploreView and LibraryView pane items"
```

---

## Task 6: 重构 YunaraPlayer 布局

**Files:**
- Modify: `crates/desktop/yunara-player/src/yunara_player.rs`

**Step 1: 重构 YunaraPlayer 结构体和初始化**

完整替换 `yunara_player.rs` 内容：

```rust
// crates/desktop/yunara-player/src/yunara_player.rs

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
//! - Center Pane: Main content (HomeView, ExploreView, LibraryView, PlaylistView)
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
    dock::{Dock, DockPanelHandle, DockPosition, panels::QueuePanel},
    pane::{Pane, PaneItemHandle, items::{HomeView, ExploreView, LibraryView}},
    player_bar::PlayerBar,
    sidebar::{NavItem, Sidebar},
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

        // Create sidebar with navigation callback
        let center_for_callback = center.clone();
        let app_state_for_callback = app_state.clone();
        let sidebar = cx.new(|cx| {
            let mut sidebar = Sidebar::new(app_state.clone(), cx);
            sidebar
        });

        // Create right dock with QueuePanel
        let right_dock = cx.new(|_cx| Dock::new(DockPosition::Right));
        let queue_panel = cx.new(|cx| QueuePanel::new(app_state.clone(), cx));
        let queue_handle = queue_panel.update(cx, |panel, _| DockPanelHandle::new(panel));
        right_dock.update(cx, |dock, _| dock.add_panel(queue_handle));

        // Create bottom dock with PlayerBar
        let bottom_dock = cx.new(|_cx| {
            let mut dock = Dock::new(DockPosition::Bottom);
            dock.set_size(90.0); // PlayerBar height
            dock
        });
        let player_bar = cx.new(|cx| PlayerBar::new(app_state.clone(), cx));
        let player_handle = player_bar.update(cx, |panel, _| DockPanelHandle::new(panel));
        bottom_dock.update(cx, |dock, _| dock.add_panel(player_handle));

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
                self.sidebar.update(cx, |sidebar, _| sidebar.set_active_nav(NavItem::Home));
            }
            NavigateAction::Explore => {
                let explore_view = cx.new(|cx| ExploreView::new(app_state, cx));
                let handle = explore_view.update(cx, |view, _| PaneItemHandle::new(view));
                self.center.update(cx, |pane, _| pane.navigate_to(handle));
                self.sidebar.update(cx, |sidebar, _| sidebar.set_active_nav(NavItem::Explore));
            }
            NavigateAction::Library => {
                let library_view = cx.new(|cx| LibraryView::new(app_state, cx));
                let handle = library_view.update(cx, |view, _| PaneItemHandle::new(view));
                self.center.update(cx, |pane, _| pane.navigate_to(handle));
                self.sidebar.update(cx, |sidebar, _| sidebar.set_active_nav(NavItem::Library));
            }
            NavigateAction::Playlist { id, name } => {
                // TODO: Create PlaylistView with proper parameters
                // For now, navigate to Library view as placeholder
                let library_view = cx.new(|cx| LibraryView::new(app_state, cx));
                let handle = library_view.update(cx, |view, _| PaneItemHandle::new(view));
                self.center.update(cx, |pane, _| pane.navigate_to(handle));
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
        let show_right_on_side = aspect_ratio >= 1.5;

        let header = Header::new("app-header").logo(yunara_assets::icons::LOGO_DARK);

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
                    .child(gpui::AnyView::from(self.sidebar.clone())),
            )
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
            });

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
            .child(header)
            .child(content)
            // Bottom dock (PlayerBar)
            .child(gpui::AnyView::from(self.bottom_dock.clone()))
    }
}
```

**Step 2: 验证编译**

Run: `cargo check -p yunara-player`
Expected: 编译成功

**Step 3: 提交**

```bash
git add crates/desktop/yunara-player/src/yunara_player.rs
git commit -m "refactor(yunara): restructure layout with Sidebar and bottom Dock"
```

---

## Task 7: 连接 Sidebar 导航回调

**Files:**
- Modify: `crates/desktop/yunara-player/src/sidebar.rs`
- Modify: `crates/desktop/yunara-player/src/yunara_player.rs`

**Step 1: 更新 Sidebar 使用全局 action 分发**

修改 `sidebar.rs` 中的 `handle_nav_click` 方法，改用 Entity 通信：

在 Sidebar 中添加一个 `workspace` 字段来引用 YunaraPlayer：

```rust
// 在 Sidebar struct 中添加
workspace: Option<WeakEntity<crate::yunara_player::YunaraPlayer>>,

// 在 new 方法中初始化
workspace: None,

// 添加设置方法
pub fn set_workspace(&mut self, workspace: WeakEntity<crate::yunara_player::YunaraPlayer>) {
    self.workspace = Some(workspace);
}

// 修改 handle_nav_click
fn handle_nav_click(&mut self, nav: NavItem, cx: &mut Context<Self>) {
    self.active_nav = nav;
    let action = match nav {
        NavItem::Home => NavigateAction::Home,
        NavItem::Explore => NavigateAction::Explore,
        NavItem::Library => NavigateAction::Library,
    };

    if let Some(ref workspace) = self.workspace {
        workspace.update(cx, |player, cx| {
            player.handle_navigate(action, cx);
        }).ok();
    }

    cx.notify();
}
```

**Step 2: 在 YunaraPlayer 中设置 workspace 引用**

在 `yunara_player.rs` 的 `new` 方法中，创建 sidebar 后设置 workspace：

```rust
// 在创建 sidebar 后添加
let weak_self_for_sidebar = weak_self.clone();
sidebar.update(cx, |sidebar, _| {
    sidebar.set_workspace(weak_self_for_sidebar);
});
```

**Step 3: 验证编译**

Run: `cargo check -p yunara-player`
Expected: 编译成功

**Step 4: 提交**

```bash
git add crates/desktop/yunara-player/src/sidebar.rs crates/desktop/yunara-player/src/yunara_player.rs
git commit -m "feat(yunara): connect Sidebar navigation to YunaraPlayer"
```

---

## Task 8: 清理旧代码

**Files:**
- Delete: `crates/desktop/yunara-player/src/dock/panels/library_panel.rs`
- Modify: `crates/desktop/yunara-player/src/dock/panels/mod.rs`
- Modify: `crates/desktop/yunara-player/src/lib.rs`

**Step 1: 删除 library_panel.rs**

```bash
rm crates/desktop/yunara-player/src/dock/panels/library_panel.rs
```

**Step 2: 更新 dock/panels/mod.rs**

```rust
// crates/desktop/yunara-player/src/dock/panels/mod.rs

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

//! Dock panel implementations.

mod queue_panel;

pub use queue_panel::QueuePanel;
```

**Step 3: 更新 lib.rs，移除 LibraryPanel 引用**

确保 `lib.rs` 不再导出 `LibraryPanel`。

**Step 4: 清理 pane_group.rs 和其他未使用代码（如有需要）**

检查并移除不再使用的 PaneGroup 相关代码。

**Step 5: 验证编译**

Run: `cargo check -p yunara-player`
Expected: 编译成功

**Step 6: 提交**

```bash
git add -A
git commit -m "refactor(yunara): remove LibraryPanel, clean up unused code"
```

---

## Task 9: 最终验证

**Step 1: 完整编译检查**

Run: `cargo build -p yunara-player`
Expected: 编译成功

**Step 2: 运行应用测试（如果有）**

Run: `cargo test -p yunara-player`
Expected: 所有测试通过

**Step 3: 提交**

如果有任何修复：
```bash
git add -A
git commit -m "fix(yunara): address compilation issues from refactor"
```

---

## 总结

完成后的文件结构：

```
yunara-player/src/
├── actions.rs              # NEW: NavigateAction 定义
├── sidebar.rs              # NEW: Sidebar 组件
├── player_bar.rs           # MODIFIED: 实现 DockPanel
├── yunara_player.rs        # MODIFIED: 新布局结构
├── dock/
│   ├── panels/
│   │   ├── mod.rs          # MODIFIED: 移除 LibraryPanel
│   │   └── queue_panel.rs  # 保持不变
│   └── ...
├── pane/
│   ├── pane.rs             # MODIFIED: 简化为单视图
│   ├── items/
│   │   ├── mod.rs          # MODIFIED: 添加新视图
│   │   ├── home_view.rs    # 保持不变
│   │   ├── explore_view.rs # NEW
│   │   ├── library_view.rs # NEW
│   │   └── playlist_view.rs # 保持不变
│   └── ...
└── lib.rs                  # MODIFIED: 添加新模块导出
```
