# Sidebar Compact Layout Design

**日期**: 2026-02-01
**状态**: 已批准
**作者**: Claude + Ryan

## 概述

改进 Yunara Player 侧边栏的响应式设计，在窄屏模式下不再隐藏文字，而是将文字显示在图标下方并缩小字体，提供更好的用户体验。

## 问题

当前侧边栏在窄屏模式（竖屏）下只显示图标，完全隐藏了文字标签。这导致：
- 用户需要记忆图标的含义
- 降低了可用性，特别是对新用户
- 与现代移动应用的常见模式不一致

## 设计方案

### 布局模式

定义两种布局模式：

1. **Horizontal（水平模式）**
   - 触发条件：`viewport_width >= viewport_height`
   - 图标和文字并排显示
   - 图标尺寸：24px
   - 文字大小：默认（约 14px）
   - 布局：flex-row，gap 12px
   - 内边距：px(12) 水平，px(10) 垂直

2. **Compact（紧凑模式）**
   - 触发条件：`viewport_height > viewport_width`
   - 图标在上，文字在下，垂直堆叠
   - 图标尺寸：20px（更平衡的视觉比例）
   - 文字大小：12px
   - 布局：flex-col，gap 4px
   - 容器：固定宽度 64px，高度自适应
   - 内边距：px(8) 上下

### 实现方案

**方案选择**: 统一的导航项渲染函数

**理由**: 在代码清晰度、可维护性和灵活性之间取得最好的平衡

#### 代码结构

1. **添加布局模式枚举**
```rust
/// Layout mode for navigation items
#[derive(Debug, Clone, Copy, PartialEq)]
enum NavItemMode {
    Horizontal,  // 图标和文字并排
    Compact,     // 图标在上，文字在下
}
```

2. **统一的渲染函数**
```rust
fn render_nav_item(
    nav: NavItem,
    icon_path: &'static str,
    label: &'static str,
    is_active: bool,
    mode: NavItemMode,  // 新增：布局模式参数
    weak_self: WeakEntity<Self>,
    cx: &Context<Self>,
) -> impl IntoElement
```

3. **动态样式应用**
   - 根据 `mode` 参数决定 flex 方向、尺寸、间距
   - 使用 `.when(mode == Compact, |el| ...)` 条件应用样式
   - 图标和文字始终渲染，只是样式不同

#### 调用方式

在 `Render::render` 中：

```rust
// 确定布局模式
let nav_mode = if viewport_width >= viewport_height {
    NavItemMode::Horizontal
} else {
    NavItemMode::Compact
};

// 统一调用渲染函数
div()
    .flex()
    .flex_col()
    .items_center()
    .py(px(8.0))
    .child(Self::render_nav_item(
        NavItem::Home,
        yunara_assets::icons::HOME,
        "Home",
        active_nav == NavItem::Home,
        nav_mode,
        weak_self.clone(),
        cx,
    ))
    // ... 其他导航项
```

### 代码变更清单

**添加**:
- `NavItemMode` 枚举定义
- 在 `render_nav_item` 中添加 `mode` 参数和条件样式逻辑

**删除**:
- `render_nav_icon_only` 函数（136-174 行）
- `show_labels` 变量（186 行）
- `.when(show_labels, ...)` 和 `.when(!show_labels, ...)` 条件分支（205-255 行）

**修改**:
- `render_nav_item` 函数：支持两种模式
- `Render::render` 中的导航渲染：简化为统一调用

## 边缘情况

1. **文字过长**
   - 当前标签（"Home"、"Explore"、"Library"）都很短
   - 64px 宽度对于 12px 文字足够
   - 未来如需更长标签，可添加文字截断或动态宽度

2. **活跃状态视觉**
   - 保持背景色高亮逻辑
   - Compact 模式下高亮区域更紧凑，视觉更突出

3. **点击热区**
   - Compact 模式容器：64px 宽 × ~50px 高
   - 符合移动端最小可点击区域标准（44x44px）

## 验证计划

- [ ] 在不同窗口尺寸下测试布局切换
- [ ] 确认 Compact 模式下文字清晰可读
- [ ] 验证点击区域大小合适
- [ ] 检查活跃状态和 hover 效果
- [ ] 测试竖屏和横屏切换的流畅性

## 未来扩展

- 可能支持用户自定义切换阈值
- 考虑添加平滑的过渡动画
- 支持更多布局模式（如超宽屏）
