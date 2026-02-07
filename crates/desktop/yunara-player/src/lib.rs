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

//! Yunara Player - Core application logic for Yunara music player
//!
//! This crate contains all application state, business logic, and UI components
//! for the Yunara desktop music player. The main binary crate handles only
//! platform-specific startup and initialization.

pub mod actions;
pub mod app_state;
pub mod client;
pub mod config;
pub mod consts;
pub mod dock;
pub mod pane;
pub mod player_bar;
pub mod services;
pub mod sidebar;
pub mod state;
pub mod util;
pub mod ytapi;
pub mod yunara_player;

pub use actions::{
    CycleRepeatMode, NavigateAction, NavigateHome, NextTrack, PreviousTrack, ToggleMute,
    TogglePlayPause, ToggleShuffle, VolumeDown, VolumeUp,
};
pub use app_state::{AppState, IdentifierKey};
pub use config::{AppConfig, ApplicationConfig};
pub use dock::{Dock, DockPanel, DockPanelHandle, DockPosition};
pub use pane::{Axis, Pane, PaneGroup, PaneItem, PaneItemHandle};
pub use player_bar::PlayerBar;
pub use sidebar::Sidebar;
pub use state::{
    NowPlayingInfo, PlaybackControls, PlayerState, ProgressSlider, RepeatMode, VolumeControl,
};
pub use yunara_player::YunaraPlayer;
