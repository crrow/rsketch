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

//! Dock system for collapsible side panels.
//!
//! Provides a dock system similar to Zed's, with:
//! - [`Dock`]: Collapsible panel container
//! - [`DockPanel`]: Trait for content that can be displayed in docks
//! - [`DockPosition`]: Where docks can be positioned (left, right, bottom)

mod dock;
mod dock_position;

pub use dock::{Dock, DockPanel, DockPanelHandle};
pub use dock_position::DockPosition;
