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

/// Application state modules for the Yunara music player.
///
/// Contains state structures for player playback, playlists, and themes.
/// These are designed to be used with GPUI's Entity system for reactive
/// updates.
mod player_state;

pub use player_state::{
    NowPlayingInfo, PlaybackControls, PlayerState, ProgressSlider, VolumeControl,
};
