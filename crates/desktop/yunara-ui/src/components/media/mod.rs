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

/// Media display components for tracks and playlists.
///
/// Contains reusable components for displaying music content:
/// - `TrackItem`: Single track in a list
/// - `CoverMosaic`: Playlist cover art grid
/// - `PlaylistItem`: Playlist entry in sidebar
mod cover_mosaic;
mod playlist_item;
mod track_item;

pub use cover_mosaic::CoverMosaic;
pub use playlist_item::PlaylistItem;
pub use track_item::TrackItem;
