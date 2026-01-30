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

/// Playlist data models for representing playlist metadata and summaries.
///
/// Contains both the full Playlist model (used in detail views) and the
/// lightweight PlaylistSummary (used in sidebar listings).
use std::time::Duration;

use gpui::SharedString;

/// Visibility setting for a playlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    /// Playlist is publicly accessible
    #[default]
    Public,
    /// Playlist is private to the owner
    Private,
    /// Playlist is accessible via link but not listed
    Unlisted,
}

impl Visibility {
    /// Returns the display label for this visibility setting.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Public => "Public",
            Self::Private => "Private",
            Self::Unlisted => "Unlisted",
        }
    }
}

/// Sort order options for tracks within a playlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    /// Original playlist order (no sorting applied)
    #[default]
    Default,
    /// Sort alphabetically by track title
    Title,
    /// Sort alphabetically by artist name
    Artist,
    /// Sort by date added to playlist (most recent first)
    DateAdded,
}

impl SortOrder {
    /// Returns the display label for this sort order.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Title => "Title",
            Self::Artist => "Artist",
            Self::DateAdded => "Date added",
        }
    }
}

/// Full playlist model with all metadata and statistics.
///
/// Used in the playlist detail view where complete information is displayed.
#[derive(Debug, Clone)]
pub struct Playlist {
    /// Unique identifier for the playlist
    pub id:             String,
    /// Playlist name/title
    pub name:           SharedString,
    /// Owner/creator username
    pub owner:          SharedString,
    /// Visibility setting (public, private, unlisted)
    pub visibility:     Visibility,
    /// Year the playlist was created
    pub year:           u16,
    /// Number of times the playlist has been viewed
    pub view_count:     u32,
    /// Number of tracks in the playlist
    pub track_count:    usize,
    /// Total duration of all tracks combined
    pub total_duration: Duration,
    /// URLs to cover images (1-4 images for mosaic display)
    pub cover_images:   Vec<SharedString>,
}

impl Playlist {
    /// Formats the total duration as a human-readable string.
    ///
    /// Returns "X hours Y minutes" or "X minutes" depending on length.
    pub fn formatted_duration(&self) -> String {
        let total_mins = self.total_duration.as_secs() / 60;
        let hours = total_mins / 60;
        let minutes = total_mins % 60;

        if hours > 0 {
            format!("{} hours {} minutes", hours, minutes)
        } else {
            format!("{} minutes", minutes)
        }
    }

    /// Returns a summary of playlist statistics for display.
    ///
    /// Format: "X views • Y tracks • Z minutes/hours"
    pub fn stats_summary(&self) -> String {
        format!(
            "{} views • {} tracks • {}",
            self.view_count,
            self.track_count,
            self.formatted_duration()
        )
    }
}

/// Lightweight playlist summary for sidebar listings.
///
/// Contains only the essential information needed for displaying
/// playlists in the sidebar navigation.
#[derive(Debug, Clone)]
pub struct PlaylistSummary {
    /// Unique identifier for the playlist
    pub id:          String,
    /// Playlist name/title
    pub name:        SharedString,
    /// Owner/creator username
    pub owner:       SharedString,
    /// Number of tracks in the playlist
    pub track_count: usize,
}

impl PlaylistSummary {
    /// Creates a new PlaylistSummary.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<SharedString>,
        owner: impl Into<SharedString>,
        track_count: usize,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            owner: owner.into(),
            track_count,
        }
    }
}
