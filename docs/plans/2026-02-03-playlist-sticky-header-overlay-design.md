# Playlist Sticky Header Overlay Design

**Date**: 2026-02-03
**Status**: Approved
**Author**: Claude (Brainstorming Session)

## Overview

实现滚动驱动的粘性浮层头部（Sticky/Collapsing Header Overlay）功能，当用户在播放列表详情页向下滚动超过 Hero 区域后，在窗口顶部以浮层形式显示一个半透明玻璃态头部，保持页面上下文信息。

## Requirements

### 功能需求

- **触发时机**: 当 Hero 区域的 bottom 接近并越过视口顶部时，头部开始渐显
- **内容显示**: 小缩略图（40px）+ 播放列表标题
- **视觉形式**: 带圆角的浮动卡片/药丸形状
- **滚动区域**: 整个播放列表视图（包括 Hero）作为一个滚动容器
- **浮层特性**: Header 以 overlay 形式叠加，不占据布局高度

### 视觉需求

- **玻璃态效果**: 半透明背景 + 轻微模糊感（使用透明度模拟）
- **渐变动画**: 透明度和模糊强度随滚动进度平滑过渡
- **遮挡优化**: 使用渐隐遮罩减少对首行曲目的视觉遮挡
- **层级关系**: Header 在 Z 轴上位于滚动内容之上

## Architecture

### 布局变化

```
滚动前:                          滚动后:
┌─────────────────────┐         ┌─────────────────────┐
│  [Hero 区域]        │         │ ╔═══════════════╗ ◄─ 浮动头部
│  - 大封面 (260px)   │    →    │ ║ 🎵 播放列表   ║   (overlay,
│  - 标题/元数据      │         │ ╚═══════════════╝    玻璃效果)
│  - 操作按钮         │         │        ↓            │
├─────────────────────┤         │   [渐隐遮罩]        │
│ [曲目列表]          │         │ [曲目列表]          │
│  🎵 歌曲 1          │         │  在头部下方流动      │
│  🎵 歌曲 2          │         │  (z-index 分层)     │
└─────────────────────┘         └─────────────────────┘
```

### 核心组件

1. **滚动状态追踪** - 监控 `scroll_offset`，检测 Hero 区域何时离开视口
2. **头部可见性逻辑** - 根据滚动距离计算 `header_progress` [0.0, 1.0]
3. **Z 轴分层渲染** - 头部使用 `absolute()` 定位，作为浮层渲染

## Component Design

### 1. 状态管理

**新增字段到 `PlaylistView`：**

```rust
pub struct PlaylistView {
    // ... 现有字段 ...
    scroll_offset: f32,         // 已有：当前滚动位置（像素）

    // NEW: 头部相关状态
    hero_height: f32,           // Hero 区域总高度（静态计算）
    header_progress: f32,       // 头部显示进度 [0.0, 1.0]
    header_visible: bool,       // 是否应该渲染头部（优化性能）
}
```

**常量定义：**

```rust
// Hero 区域尺寸（静态计算）
const HERO_CARD_SIZE: f32 = 260.0;
const HERO_PADDING: f32 = 18.0;
const HERO_TITLE_HEIGHT: f32 = 40.0;     // 标题行
const HERO_META_HEIGHT: f32 = 100.0;     // 元数据 + 按钮
const HERO_TOTAL_HEIGHT: f32 = HERO_CARD_SIZE + HERO_PADDING * 2
                                + HERO_TITLE_HEIGHT + HERO_META_HEIGHT;
// ≈ 436px

// 浮动头部尺寸
const FLOATING_HEADER_HEIGHT: f32 = 64.0;
const FLOATING_HEADER_WIDTH: f32 = 600.0;
const FLOATING_HEADER_RADIUS: f32 = 12.0;
const FLOATING_HEADER_TOP_MARGIN: f32 = 12.0;  // 距离窗口顶部的间距

// 过渡参数
const HEADER_FADE_START_OFFSET: f32 = 50.0;    // Hero bottom 距离顶部多少像素开始渐显
const HEADER_FADE_DISTANCE: f32 = 100.0;       // 渐显动画的距离范围（100px 过渡）
```

**进度计算逻辑：**

```rust
fn update_header_progress(&mut self) {
    // Hero 底部相对于视口顶部的距离
    let hero_bottom_offset = self.hero_height - self.scroll_offset;

    // 当 hero_bottom_offset 从 FADE_START 降到 (FADE_START - FADE_DISTANCE) 时
    // progress 从 0.0 增加到 1.0
    let progress = if hero_bottom_offset > HEADER_FADE_START_OFFSET {
        0.0  // Hero 还很靠上，头部完全透明
    } else if hero_bottom_offset < (HEADER_FADE_START_OFFSET - HEADER_FADE_DISTANCE) {
        1.0  // Hero 已经滚出很多，头部完全显示
    } else {
        // 线性插值
        (HEADER_FADE_START_OFFSET - hero_bottom_offset) / HEADER_FADE_DISTANCE
    };

    self.header_progress = progress;
    self.header_visible = progress > 0.01;  // 超过 1% 才渲染，优化性能
}
```

### 2. 渲染层实现

**整体结构调整：**

```rust
fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    // ... 现有的颜色计算 ...

    // 初始化 hero 高度
    self.hero_height = HERO_TOTAL_HEIGHT;

    // 根据滚动更新头部进度
    self.update_header_progress();

    gpui::div()
        .id("playlist-view-root")
        .relative()  // 关键：让浮动头部可以 absolute 定位
        .w_full()
        .h_full()
        .overflow_hidden()
        // 主滚动容器
        .child(
            gpui::div()
                .id("playlist-scroll-container")
                .w_full()
                .h_full()
                .overflow_y_scroll()
                .on_scroll(cx.listener(|this, event, cx| {
                    this.scroll_offset = event.position.y;
                    this.check_load_more(event.bounds.size.height.0, cx);
                    cx.notify();  // 触发重渲染以更新头部
                }))
                .child(self.render_hero_section(theme, cx))
                .child(self.render_track_list(theme, cx))
        )
        // 浮动头部（在滚动容器之后，确保 z-index 更高）
        .when(self.header_visible, |el| {
            el.child(self.render_floating_header(theme, cx))
        })
}
```

**浮动头部组件：**

```rust
fn render_floating_header(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
    let progress = self.header_progress;

    // 玻璃态效果：随进度增强
    let bg_alpha = 0.65 * progress;        // 最终 65% 不透明
    let border_alpha = 0.15 * progress;    // 边框透明度

    let glass_bg = with_alpha(theme.background_primary, bg_alpha);
    let glass_border = with_alpha(theme.text_primary, border_alpha);

    gpui::div()
        .absolute()  // 不占布局空间
        .top(px(FLOATING_HEADER_TOP_MARGIN))
        .left_1_2()  // 水平居中（50%）
        .w(px(FLOATING_HEADER_WIDTH))
        .h(px(FLOATING_HEADER_HEIGHT))
        .rounded(px(FLOATING_HEADER_RADIUS))
        .bg(glass_bg)
        .border_1()
        .border_color(glass_border)
        .shadow_lg()
        .opacity(progress)  // 整体透明度随进度变化
        .flex()
        .items_center()
        .gap_3()
        .px_4()
        // 小缩略图 + 标题
        .child(/* thumbnail */)
        .child(/* title */)
        // 渐隐遮罩（避免遮挡首行曲目）
        .child(self.render_header_fade_mask(theme, progress))
}
```

## Data Flow

```
用户打开播放列表
    ↓
PlaylistView::new()
    hero_height = 436.0
    header_progress = 0.0
    header_visible = false
    ↓
render() 第一次
    渲染 Hero + TrackList
    浮动头部不显示
    ↓
用户向下滚动
    ↓
on_scroll → scroll_offset 更新 → cx.notify()
    ↓
render() 再次调用
    ↓
update_header_progress()
    计算 progress 基于 hero_bottom_offset
    ↓
当 progress > 0.01 时
    header_visible = true
    render_floating_header() 渲染
    opacity 和 bg_alpha 随 progress 变化
    ↓
平滑渐显效果
```

## Implementation Order

1. **Phase 1**: 添加状态字段和常量
2. **Phase 2**: 实现 `update_header_progress()` 方法
3. **Phase 3**: 重构 `render()` 结构，拆分 Hero 和 TrackList
4. **Phase 4**: 实现 `render_floating_header()` 和渐隐遮罩
5. **Phase 5**: 集成滚动事件，测试和调优

## Key Parameters

| 参数 | 值 | 说明 |
|------|-----|------|
| Hero 总高度 | ~436px | 封面 + padding + 元数据 |
| 触发起点 | Hero bottom 距顶部 50px | 开始渐显 |
| 过渡距离 | 100px | 从 0% 到 100% 的滚动距离 |
| 头部高度 | 64px | 浮动头部卡片高度 |
| 头部宽度 | 600px | 浮动头部卡片宽度 |
| 最大背景透明度 | 65% | 完全显示时的不透明度 |
