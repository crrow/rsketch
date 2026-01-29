/// Player state management for the music player.
///
/// Contains all playback-related state including the current track,
/// playback controls, progress, volume, and queue management.

use std::time::Duration;

use gpui::SharedString;
use yunara_ui::models::{Playlist, Track};

/// Information about the currently playing track.
///
/// Contains display data for the player bar including track metadata
/// and cover art (when loaded).
#[derive(Debug, Clone)]
pub struct NowPlayingInfo {
    /// Track title for display
    pub track_title: SharedString,
    /// Artist name for display
    pub artist_name: SharedString,
    /// URL to fetch cover art from
    pub cover_url: Option<SharedString>,
}

impl NowPlayingInfo {
    /// Creates NowPlayingInfo from a Track.
    pub fn from_track(track: &Track) -> Self {
        Self {
            track_title: track.title.clone(),
            artist_name: track.artist.clone(),
            cover_url: track.cover_url.clone(),
        }
    }
}

/// Playback control state (play/pause, previous/next availability).
#[derive(Debug, Clone, Default)]
pub struct PlaybackControls {
    /// Whether audio is currently playing
    pub is_playing: bool,
    /// Whether there is a previous track to navigate to
    pub has_previous: bool,
    /// Whether there is a next track to navigate to
    pub has_next: bool,
}

/// Progress slider state for tracking playback position.
#[derive(Debug, Clone, Default)]
pub struct ProgressSlider {
    /// Current playback position
    pub current_time: Duration,
    /// Total track duration
    pub total_duration: Duration,
    /// Whether the user is currently dragging the slider
    pub is_dragging: bool,
    /// Visual position while dragging (0.0 to 1.0)
    pub drag_position: Option<f32>,
}

impl ProgressSlider {
    /// Returns the current progress as a fraction (0.0 to 1.0).
    pub fn progress_fraction(&self) -> f32 {
        if self.total_duration.is_zero() {
            return 0.0;
        }

        if let Some(drag_pos) = self.drag_position {
            drag_pos
        } else {
            self.current_time.as_secs_f32() / self.total_duration.as_secs_f32()
        }
    }

    /// Formats the current time as "M:SS" or "H:MM:SS".
    pub fn formatted_current_time(&self) -> String {
        format_duration(self.current_time)
    }

    /// Formats the total duration as "M:SS" or "H:MM:SS".
    pub fn formatted_total_duration(&self) -> String {
        format_duration(self.total_duration)
    }
}

/// Volume control state.
#[derive(Debug, Clone)]
pub struct VolumeControl {
    /// Current volume level (0.0 to 1.0)
    pub volume: f32,
    /// Whether audio is muted
    pub is_muted: bool,
    /// Volume level before muting (to restore on unmute)
    pub previous_volume: f32,
}

impl Default for VolumeControl {
    fn default() -> Self {
        Self {
            volume: 0.5,
            is_muted: false,
            previous_volume: 0.5,
        }
    }
}

impl VolumeControl {
    /// Returns the effective volume (0 if muted, otherwise current volume).
    pub fn effective_volume(&self) -> f32 {
        if self.is_muted {
            0.0
        } else {
            self.volume
        }
    }

    /// Toggles mute state, preserving volume for unmute.
    pub fn toggle_mute(&mut self) {
        if self.is_muted {
            self.is_muted = false;
        } else {
            self.previous_volume = self.volume;
            self.is_muted = true;
        }
    }

    /// Sets volume and clears mute state.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        self.is_muted = false;
    }
}

/// Complete player state including playback, queue, and current playlist.
///
/// This is the central state structure for all player-related functionality.
/// It should be stored in an Entity for GPUI state management.
#[derive(Debug, Clone, Default)]
pub struct PlayerState {
    /// Information about the currently playing track
    pub now_playing: Option<NowPlayingInfo>,
    /// Playback control state
    pub playback: PlaybackControls,
    /// Progress slider state
    pub progress: ProgressSlider,
    /// Volume control state
    pub volume: VolumeControl,
    /// Queue of tracks to play
    pub queue: Vec<Track>,
    /// Index of currently playing track in the queue
    pub current_index: Option<usize>,
    /// The playlist being played from (determines if queue panel shows)
    pub current_playlist: Option<Playlist>,
}

impl PlayerState {
    /// Creates a new PlayerState with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether there is an active playback session.
    ///
    /// Used to determine whether to show the queue panel.
    pub fn has_active_playlist(&self) -> bool {
        self.current_playlist.is_some()
    }

    /// Returns the currently playing track from the queue.
    pub fn current_track(&self) -> Option<&Track> {
        self.current_index.and_then(|idx| self.queue.get(idx))
    }

    /// Starts playing a playlist from the beginning.
    pub fn play_playlist(&mut self, playlist: Playlist, tracks: Vec<Track>) {
        self.current_playlist = Some(playlist);
        self.queue = tracks;
        self.current_index = if self.queue.is_empty() {
            None
        } else {
            Some(0)
        };
        self.update_now_playing();
        self.update_navigation_state();
        self.playback.is_playing = true;
    }

    /// Advances to the next track in the queue.
    ///
    /// Returns true if there was a next track to play.
    pub fn next_track(&mut self) -> bool {
        if let Some(idx) = self.current_index {
            if idx + 1 < self.queue.len() {
                self.current_index = Some(idx + 1);
                self.update_now_playing();
                self.update_navigation_state();
                self.reset_progress();
                return true;
            }
        }
        false
    }

    /// Goes back to the previous track in the queue.
    ///
    /// Returns true if there was a previous track to play.
    pub fn previous_track(&mut self) -> bool {
        if let Some(idx) = self.current_index {
            if idx > 0 {
                self.current_index = Some(idx - 1);
                self.update_now_playing();
                self.update_navigation_state();
                self.reset_progress();
                return true;
            }
        }
        false
    }

    /// Toggles play/pause state.
    pub fn toggle_playback(&mut self) {
        self.playback.is_playing = !self.playback.is_playing;
    }

    /// Updates the now_playing field based on current queue position.
    fn update_now_playing(&mut self) {
        self.now_playing = self.current_track().map(NowPlayingInfo::from_track);
    }

    /// Updates has_previous and has_next based on queue position.
    fn update_navigation_state(&mut self) {
        if let Some(idx) = self.current_index {
            self.playback.has_previous = idx > 0;
            self.playback.has_next = idx + 1 < self.queue.len();
        } else {
            self.playback.has_previous = false;
            self.playback.has_next = false;
        }
    }

    /// Resets progress for a new track.
    fn reset_progress(&mut self) {
        self.progress.current_time = Duration::ZERO;
        self.progress.is_dragging = false;
        self.progress.drag_position = None;

        if let Some(track) = self.current_track() {
            self.progress.total_duration = track.duration;
        }
    }
}

/// Formats a Duration as "M:SS" or "H:MM:SS".
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}
