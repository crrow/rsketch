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

//! Application actions for navigation and state changes.

use gpui::actions;

/// Navigation actions for switching views in the center pane.
#[derive(Debug, Clone, PartialEq)]
pub enum NavigateAction {
    /// Navigate to the home view
    Home,
    /// Navigate to the explore view
    Explore,
    /// Navigate to the library view
    Library,
    /// Navigate to a specific playlist
    Playlist { id: String, name: String },
}

// Keyboard shortcut actions for playback and navigation
actions!(
    yunara,
    [
        TogglePlayPause,
        NextTrack,
        PreviousTrack,
        VolumeUp,
        VolumeDown,
        ToggleMute,
        NavigateHome,
    ]
);
