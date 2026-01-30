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

//! Pane group implementation for split layouts.
//!
//! Similar to Zed's PaneGroup, this enables hierarchical split layouts
//! where panes can be arranged horizontally or vertically.

use gpui::{Context, Entity, IntoElement, ParentElement, Render, Styled};

use super::pane::Pane;

/// Axis for splitting panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Horizontal split (side-by-side)
    Horizontal,
    /// Vertical split (top-bottom)
    Vertical,
}

/// A recursive structure representing either a single pane or a split of pane
/// groups.
///
/// This enables flexible layouts like:
/// - Single pane
/// - Two panes side by side (Horizontal)
/// - Two panes top-bottom (Vertical)
/// - Complex nested splits
pub enum PaneGroup {
    /// A single pane
    Pane(Entity<Pane>),

    /// A split containing two pane groups
    Split {
        /// Direction of the split
        axis:   Axis,
        /// First group (left or top)
        first:  Box<PaneGroup>,
        /// Second group (right or bottom)
        second: Box<PaneGroup>,
        /// Split ratio (0.0 to 1.0, where first group takes this proportion)
        ratio:  f32,
    },
}

impl PaneGroup {
    pub fn render_element(&self) -> impl IntoElement {
        match self {
            PaneGroup::Pane(pane) => gpui::div()
                .flex_1()
                .w_full()
                .h_full()
                .child(gpui::AnyView::from(pane.clone())),
            PaneGroup::Split {
                axis,
                first,
                second,
                ..
            } => {
                let mut container = gpui::div().flex().w_full().h_full();
                if *axis == Axis::Vertical {
                    container = container.flex_col();
                }
                container
                    .child(first.render_element())
                    .child(second.render_element())
            }
        }
    }

    /// Creates a new pane group with a single pane.
    pub fn new(pane: Entity<Pane>) -> Self { PaneGroup::Pane(pane) }

    /// Creates a horizontal split between two groups.
    pub fn horizontal_split(first: PaneGroup, second: PaneGroup, ratio: f32) -> Self {
        PaneGroup::Split {
            axis:   Axis::Horizontal,
            first:  Box::new(first),
            second: Box::new(second),
            ratio:  ratio.clamp(0.0, 1.0),
        }
    }

    /// Creates a vertical split between two groups.
    pub fn vertical_split(first: PaneGroup, second: PaneGroup, ratio: f32) -> Self {
        PaneGroup::Split {
            axis:   Axis::Vertical,
            first:  Box::new(first),
            second: Box::new(second),
            ratio:  ratio.clamp(0.0, 1.0),
        }
    }

    /// Returns all panes in this group (recursively).
    pub fn panes(&self) -> Vec<&Entity<Pane>> {
        match self {
            PaneGroup::Pane(pane) => vec![pane],
            PaneGroup::Split { first, second, .. } => {
                let mut panes = first.panes();
                panes.extend(second.panes());
                panes
            }
        }
    }

    /// Returns a mutable reference to all panes in this group.
    pub fn panes_mut(&mut self) -> Vec<&mut Entity<Pane>> {
        match self {
            PaneGroup::Pane(pane) => vec![pane],
            PaneGroup::Split { first, second, .. } => {
                let mut panes = first.panes_mut();
                panes.extend(second.panes_mut());
                panes
            }
        }
    }

    /// Returns whether this group is a split.
    pub fn is_split(&self) -> bool { matches!(self, PaneGroup::Split { .. }) }

    /// Returns the split axis if this is a split, otherwise None.
    pub fn axis(&self) -> Option<Axis> {
        match self {
            PaneGroup::Split { axis, .. } => Some(*axis),
            PaneGroup::Pane(_) => None,
        }
    }

    /// Returns the split ratio if this is a split, otherwise None.
    pub fn ratio(&self) -> Option<f32> {
        match self {
            PaneGroup::Split { ratio, .. } => Some(*ratio),
            PaneGroup::Pane(_) => None,
        }
    }
}

impl Render for PaneGroup {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
