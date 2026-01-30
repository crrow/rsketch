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

/// Data models shared across UI components.
///
/// This module contains the core data structures used throughout the Yunara
/// music player UI, including track metadata, playlist information, and
/// navigation types.
mod navigation;
mod playlist;
mod track;

pub use navigation::{Icon, NavItem, Route};
pub use playlist::{Playlist, PlaylistSummary, SortOrder, Visibility};
pub use track::Track;
