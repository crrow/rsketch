# Dock/Panel 架构重构设计

## 概述

重构 yunara-player 的布局架构，使其更清晰地分离导航、内容和可隐藏面板的职责。

## 当前问题

1. **PlayerBar 不是 Dock 面板** - 硬编码在布局底部，没有使用 Dock 抽象
2. **LibraryPanel 职责混乱** - 内部维护多种视图状态，包含导航逻辑
3. **左侧 Dock 使用不当** - Home/Explore/Library 按钮应该控制中心内容，而不是切换 Dock 面板

## 目标架构

```
┌─────────────────────────────────────────────────────────┐
│                        Header                           │
├───────────┬─────────────────────────────┬───────────────┤
│           │                             │               │
│  Sidebar  │        Center Pane          │  Right Dock   │
│ (非 Dock)  │      (单一视图,替换)         │  QueuePanel   │
│           │                             │               │
│  • Home   │   HomeView / ExploreView    │               │
│  • Explore│   / PlaylistView            │               │
│  • Library│                             │               │
│  ─────────│                             │               │
│  播放列表  │                             │               │
│  (响应式)  │                             │               │
│           │                             │               │
├───────────┴─────────────────────────────┴───────────────┤
│                   Bottom Dock                           │
│                   PlayerBar                             │
└─────────────────────────────────────────────────────────┘
```

## 组件职责

| 区域 | 实现方式 | 职责 |
|------|----------|------|
| 左侧 | `Sidebar` 组件（非 Dock） | 固定导航，响应式显示播放列表 |
| 中心 | 简化的 `Pane`（单一视图） | 显示当前内容，直接替换 |
| 右侧 | `Dock` + `QueuePanel` | 可隐藏/显示的队列面板 |
| 底部 | `Dock` + `PlayerBarPanel` | 可隐藏/显示，便于未来扩展 |

## 详细设计

### 1. Sidebar 组件

```rust
pub struct Sidebar {
    app_state: AppState,
    expanded: bool,  // 响应式：窗口够大时展开显示播放列表
}

enum NavItem {
    Home,
    Explore,
    Library,
}
```

**布局结构：**

```
┌─────────────┐
│  🏠 Home    │  ← 点击 → 中心显示 HomeView
│  🔍 Explore │  ← 点击 → 中心显示 ExploreView
│  📚 Library │  ← 点击 → 中心显示 LibraryView
├─────────────┤
│  新建播放列表 │  ← 仅在 expanded=true 时显示
├─────────────┤
│  • 我喜欢    │
│  • 播放列表1 │  ← 点击 → 中心显示 PlaylistView
│  • 播放列表2 │
└─────────────┘
```

**响应式逻辑：**

```rust
fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let window_width = window.viewport_size().width;
    let show_playlists = window_width > px(900.0);
    // ...
}
```

**通信方式：** 使用 GPUI Action 系统分发 `NavigateAction`

### 2. 底部 Dock 与 PlayerBarPanel

PlayerBar 实现 DockPanel trait：

```rust
impl DockPanel for PlayerBar {
    fn title(&self) -> String {
        "Player".to_string()
    }

    fn icon(&self) -> Option<&'static str> {
        None
    }

    fn to_any_view(&self) -> AnyView {
        // 现有的渲染逻辑
    }
}
```

### 3. 中心 Pane 简化

单一视图模式，移除多标签支持：

```rust
pub struct Pane {
    current_item: Option<PaneItemHandle>,
}

impl Pane {
    pub fn navigate_to(&mut self, item: impl PaneItem) {
        self.current_item = Some(PaneItemHandle::new(item));
    }
}
```

### 4. NavigateAction 定义

```rust
#[derive(Clone, Debug)]
pub enum NavigateAction {
    Home,
    Explore,
    Library,
    Playlist { id: String },
}
```

### 5. YunaraPlayer 结构变更

```rust
pub struct YunaraPlayer {
    app_state: AppState,
    sidebar: Entity<Sidebar>,     // 替代 left_dock
    center: Entity<Pane>,         // 简化的单视图 Pane
    right_dock: Entity<Dock>,     // 保持不变
    bottom_dock: Entity<Dock>,    // 新增，包含 PlayerBar
}
```

## 文件变更清单

| 操作 | 文件 | 说明 |
|------|------|------|
| 新建 | `sidebar.rs` | 导航栏 + 响应式播放列表 |
| 新建 | `pane/items/explore_view.rs` | 探索视图 |
| 新建 | `pane/items/library_view.rs` | 音乐库视图 |
| 新建 | `actions.rs` | NavigateAction 定义 |
| 修改 | `player_bar.rs` | 实现 DockPanel trait |
| 修改 | `pane/pane.rs` | 简化为单视图模式 |
| 修改 | `yunara_player.rs` | 重构布局 |
| 修改 | `pane/items/playlist_view.rs` | 完善实现 |
| 删除 | `dock/panels/library_panel.rs` | 功能已拆分 |

## 实现顺序

1. 定义 NavigateAction
2. 简化 Pane 结构
3. 修改 PlayerBar 实现 DockPanel
4. 创建 Sidebar 组件
5. 创建 ExploreView、LibraryView
6. 重构 YunaraPlayer 布局
7. 删除 LibraryPanel
8. 测试验证

## 预期结果

- 架构清晰：Sidebar（导航）、Pane（内容）、Dock（可隐藏面板）
- 响应式：窄窗口只显示导航，宽窗口显示播放列表
- 可扩展：底部 Dock 未来可添加歌词面板等
