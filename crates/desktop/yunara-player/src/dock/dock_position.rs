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

//! Dock position enumeration.

/// Position where a dock can be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DockPosition {
    /// Left side of the workspace
    Left,
    /// Right side of the workspace
    Right,
    /// Bottom of the workspace
    Bottom,
}

impl DockPosition {
    /// Returns whether this dock is positioned horizontally (left or right).
    pub fn is_horizontal(&self) -> bool {
        matches!(self, DockPosition::Left | DockPosition::Right)
    }

    /// Returns whether this dock is positioned vertically (bottom).
    pub fn is_vertical(&self) -> bool {
        matches!(self, DockPosition::Bottom)
    }
}
