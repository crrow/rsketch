/// Player control components for the bottom player bar.
///
/// Contains components for playback control:
/// - `PlaybackControls`: Previous, play/pause, next buttons
/// - `ProgressSlider`: Track progress with seek capability
/// - `VolumeControl`: Volume slider with mute toggle
/// - `PlayerBar`: Complete bottom bar combining all controls

mod playback_controls;
mod player_bar;
mod progress_slider;
mod volume_control;

pub use playback_controls::PlaybackControls;
pub use player_bar::PlayerBar;
pub use progress_slider::ProgressSlider;
pub use volume_control::VolumeControl;
