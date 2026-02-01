# YouTube Music UI 改进设计

**日期**: 2026-02-01
**状态**: 已批准
**作者**: Claude + Ryan

## 概述

改进 Yunara Player 的 UI，使其更接近 YouTube Music 的外观和体验。包括导航图标的双态设计、Playbar 布局重组和高度调整。

## 背景

当前实现与 YouTube Music 相比存在以下差异：
1. 导航图标在激活状态下没有视觉变化（应该显示填充版本）
2. Playbar 布局不符合 YouTube Music 的标准（控制在中间，应该在左侧）
3. Playbar 高度过大（90px vs YouTube Music 的约 72px）
4. 窗口样式与 YouTube Music 网页版不同（后续处理）

## 设计方案

### 1. 导航图标的双态设计

**目标**: 为每个导航项提供清晰的视觉反馈，激活状态使用填充图标。

**实现方式**:

#### 图标资源
在 `yunara-assets/src/lib.rs` 的 `icons` 模块中添加填充版本：
```rust
// Navigation icons - filled versions for active state
pub const HOME_FILLED: &str = "icons/home-filled.svg";
pub const EXPLORE_FILLED: &str = "icons/explore-filled.svg";
pub const LIBRARY_FILLED: &str = "icons/library-filled.svg";
```

#### Sidebar 修改
修改 `render_nav_item` 函数签名：
```rust
fn render_nav_item(
    nav: NavItem,
    icon_path: &'static str,
    icon_filled_path: &'static str,  // 新增参数
    label: &'static str,
    is_active: bool,
    mode: NavItemMode,
    weak_self: WeakEntity<Self>,
    cx: &Context<Self>,
) -> impl IntoElement
```

在函数内部根据 `is_active` 选择图标：
```rust
let selected_icon = if is_active { icon_filled_path } else { icon_path };
```

#### 调用处更新
在 `Render::render` 中调用时传入两个图标路径：
```rust
Self::render_nav_item(
    NavItem::Home,
    yunara_assets::icons::HOME,
    yunara_assets::icons::HOME_FILLED,  // 新增
    "Home",
    active_nav == NavItem::Home,
    nav_mode,
    weak_self.clone(),
    cx,
)
```

### 2. Playbar 布局重组

**目标**: 重新设计 Playbar 布局，匹配 YouTube Music 的标准布局。

**当前布局** (需要修改):
- 左侧：专辑封面 + 歌曲信息 (200px)
- 中间：播放控制 + 时间 (flex-1)
- 右侧：音量控制 (固定宽度)

**新布局**:
- 左侧：播放控制按钮 (约 200px)
- 中间：专辑封面 + 歌曲信息 (flex-1)
- 右侧：音量控制 + 其他按钮 (约 200px)

#### 左侧区域（播放控制）
```rust
div()
    .flex()
    .items_center()
    .justify_center()
    .w(px(200.0))
    .gap_4()
    .child(control_button("prev-btn", MEDIA_PREVIOUS, 32.0, 16.0))
    .child(control_button("play-btn", play_icon, 40.0, 20.0))
    .child(control_button("next-btn", MEDIA_NEXT, 32.0, 16.0))
```

#### 中间区域（歌曲信息）
```rust
div()
    .flex_1()
    .flex()
    .items_center()
    .gap_3()
    .px(px(16.0))
    // 专辑封面 (48x48)
    .child(album_cover)
    // 歌曲信息
    .child(track_info)
```

#### 右侧区域（音量控制）
```rust
div()
    .flex()
    .items_center()
    .justify_end()
    .w(px(200.0))
    .pr(px(16.0))
    .child(volume_control)
```

### 3. 尺寸调整

**Playbar 高度**:
- 当前：90px
- 新值：72px
- 原因：更接近 YouTube Music 的紧凑设计

**专辑封面**:
- 当前：56px × 56px
- 新值：48px × 48px
- 原因：配合新的 72px 高度，保持比例协调

**进度条**:
- 保持：4px 高度
- 位置：Playbar 顶部
- 主内容区域：68px (72px - 4px)

**播放按钮**:
- 播放/暂停：40px 直径
- 上一曲/下一曲：32px 直径
- 内部图标：20px / 16px

### 4. 窗口配置（暂缓实施）

**目标**: 实现无边框窗口，Header 从顶部开始。

**决策**: 采用方案 C - 暂时保留当前窗口配置，先完成图标和 Playbar 改进。

**原因**:
- 窗口配置涉及 GPUI 平台特定 API
- 需要处理窗口拖拽区域和控制按钮
- 复杂度较高，适合单独处理

**后续**: 在完成当前改进后，单独设计和实现窗口样式改进。

## 代码变更清单

### 新增文件
1. `icons/home-filled.svg` - Home 图标填充版本
2. `icons/explore-filled.svg` - Explore 图标填充版本
3. `icons/library-filled.svg` - Library 图标填充版本

### 修改文件

#### yunara-assets/src/lib.rs
- 添加三个填充图标常量

#### sidebar.rs
- 修改 `render_nav_item` 函数签名（添加 `icon_filled_path` 参数）
- 在函数内部根据 `is_active` 选择图标
- 更新所有调用处，传入两个图标路径

#### player_bar.rs
- 调整容器高度：`h(px(90.0))` → `h(px(72.0))`
- 重新排列三个区域：左侧播放控制、中间歌曲信息、右侧音量
- 缩小专辑封面：`w(px(56.0)).h(px(56.0))` → `w(px(48.0)).h(px(48.0))`
- 调整左侧区域为播放控制按钮
- 调整中间区域为歌曲信息 + 封面
- 保持右侧音量控制不变

#### yunara_player.rs
- 更新 bottom_dock 高度设置：`dock.set_size(90.0)` → `dock.set_size(72.0)`
- 更新固定高度容器：`h(px(90.0))` → `h(px(72.0))`

## 实现优先级

1. **优先级 1**: 导航图标双态（需要先创建 SVG 文件）
2. **优先级 2**: Playbar 布局重组
3. **优先级 3**: Playbar 高度调整
4. **后续**: 窗口配置改进（单独任务）

## 验证计划

- [ ] 导航图标在激活/非激活状态下正确显示
- [ ] Playbar 布局符合设计（左中右三区域）
- [ ] Playbar 高度为 72px，视觉紧凑
- [ ] 专辑封面 48px，与整体比例协调
- [ ] 播放控制按钮在左侧，大小合适，易于点击
- [ ] 歌曲信息在中间，正确显示
- [ ] 音量控制在右侧，功能正常

## 依赖和风险

**图标资源依赖**:
- 需要创建或获取三个填充版本的 SVG 图标
- 图标样式需要与 YouTube Music 一致

**布局风险**:
- 不同窗口尺寸下需要测试布局是否正常
- 窄屏模式下可能需要特殊处理

**后续工作**:
- 窗口配置改进需要深入研究 GPUI API
- 可能需要平台特定代码（macOS/Windows/Linux）
