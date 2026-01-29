/// Application state modules for the Yunara music player.
///
/// Contains state structures for player playback, playlists, and themes.
/// These are designed to be used with GPUI's Entity system for reactive
/// updates.
mod player_state;

pub use player_state::{
    NowPlayingInfo, PlaybackControls, PlayerState, ProgressSlider, VolumeControl,
};
