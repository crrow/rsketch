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

//! Pane system for managing split layouts and tabbed content.
//!
//! Inspired by Zed's pane architecture, this module provides:
//! - [`Pane`]: Container for multiple pane items with tab-like navigation
//! - [`PaneGroup`]: Recursive split layout support
//! - [`PaneItem`]: Trait for content that can be displayed in panes

mod pane;
mod pane_group;
mod pane_item;

pub mod items;

pub use pane::Pane;
pub use pane_group::{Axis, PaneGroup};
pub use pane_item::{PaneItem, PaneItemHandle};
