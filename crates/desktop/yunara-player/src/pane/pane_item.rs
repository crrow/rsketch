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

//! Trait for items that can be displayed in a pane.
//!
//! Similar to Zed's ItemHandle, this defines the interface for content
//! that can be shown in panes (playlists, albums, search results, etc).

use gpui::{AnyView, EntityId};

/// Trait for items that can be displayed in a pane.
///
/// Each pane item represents a distinct view or content area, such as:
/// - Playlist view
/// - Album details
/// - Search results
/// - Now playing queue
pub trait PaneItem: Send + Sync {
    /// Returns a unique identifier for this item.
    fn entity_id(&self) -> EntityId;

    /// Returns a human-readable title for this item (e.g., "Liked Songs", "Album: Dark Side").
    fn tab_title(&self) -> String;

    /// Renders the item's content as a view.
    fn to_any_view(&self) -> AnyView;

    /// Called when the item becomes the active item in its pane.
    fn on_focus(&mut self) {}

    /// Called when the item loses focus.
    fn on_blur(&mut self) {}

    /// Whether this item can be closed by the user.
    fn can_close(&self) -> bool {
        true
    }

    /// Whether this item has unsaved changes.
    fn is_dirty(&self) -> bool {
        false
    }
}

/// Type-erased handle to a pane item.
pub struct PaneItemHandle {
    entity_id: EntityId,
    title: String,
    view: AnyView,
}

impl PaneItemHandle {
    /// Creates a new pane item handle from a type implementing PaneItem.
    pub fn new(item: &impl PaneItem) -> Self {
        Self {
            entity_id: item.entity_id(),
            title: item.tab_title(),
            view: item.to_any_view(),
        }
    }

    /// Returns the entity ID of this item.
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// Returns the title of this item.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the view for rendering.
    pub fn view(&self) -> &AnyView {
        &self.view
    }
}
