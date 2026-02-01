# Layout Agent

## 目标

描述 Yunara Player 的整体布局结构与关键对齐规则，作为 UI 布局修改的约束说明。

## 布局结构

1. **Top Row（顶部行）**
   - 左侧：Sidebar 的品牌区域（菜单 + Logo）。
   - 右侧：Header（搜索框 + 右侧控制区）。
   - 两者处于同一水平线，并对齐高度（当前 56px）。

2. **Main Row（主体行）**
   - 左侧：Sidebar（导航 + 播放列表）。
   - 中间：Center 内容区。
   - 右侧：Queue Dock（仅在宽屏布局时显示在右侧）。

3. **Bottom Row（底部）**
   - PlayerBar 固定高度（72px），占满窗口宽度。

## 对齐与间距规则

1. **Header 左边界**
   - Header 不覆盖 Sidebar 宽度。
   - Header 只出现在 Center + Right 区域，从 Sidebar 右侧开始。

2. **Sidebar 顶部区域**
   - 菜单与 Logo 位于 Sidebar 顶部。
   - 与 Header 保持同一水平线。

3. **Nav 与 Playlists**
   - Nav Item 有统一左侧内边距（当前 12px）。
   - Nav 与 Playlist 之间有分割线与 padding。
   - 窄屏布局时，Playlist 与分割线隐藏。

## 宽窄屏切换

- 使用统一阈值常量：

```rust
pub const NARROW_LAYOUT_ASPECT_RATIO: f32 = 0.8;
```

- 当 `aspect_ratio < NARROW_LAYOUT_ASPECT_RATIO` 进入窄屏布局：
  - Right Dock 移至下方。
  - Sidebar 切换为紧凑模式。
  - Playlist 区域隐藏。

## 变更建议

- 若修改 Header 高度，需同步调整 Sidebar 顶部品牌区高度。
- 若修改 Sidebar 宽度，需保持 Header 左边界从 Sidebar 右侧开始。
