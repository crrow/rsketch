/// Track data model representing a single music track.
///
/// Contains metadata about a track including its unique identifier,
/// display information (title, artist, album), duration, and cover art URL.

use std::time::Duration;

use gpui::SharedString;

/// Represents a single music track with its metadata.
///
/// Used throughout the UI for displaying track information in lists,
/// the player bar, and queue panels.
#[derive(Debug, Clone)]
pub struct Track {
    /// Unique identifier for the track (from YTMusic API)
    pub id: String,
    /// Track title for display
    pub title: SharedString,
    /// Primary artist name
    pub artist: SharedString,
    /// Album name (optional, not all tracks have album info)
    pub album: Option<SharedString>,
    /// Track duration
    pub duration: Duration,
    /// URL to the cover art image (optional)
    pub cover_url: Option<SharedString>,
}

impl Track {
    /// Creates a new Track with all required fields.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<SharedString>,
        artist: impl Into<SharedString>,
        duration: Duration,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            artist: artist.into(),
            album: None,
            duration,
            cover_url: None,
        }
    }

    /// Sets the album name.
    pub fn with_album(mut self, album: impl Into<SharedString>) -> Self {
        self.album = Some(album.into());
        self
    }

    /// Sets the cover art URL.
    pub fn with_cover_url(mut self, url: impl Into<SharedString>) -> Self {
        self.cover_url = Some(url.into());
        self
    }

    /// Formats the duration as "M:SS" or "H:MM:SS" for display.
    pub fn formatted_duration(&self) -> String {
        let total_secs = self.duration.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{}:{:02}", minutes, seconds)
        }
    }
}
