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

pub mod config;

pub use config::{AppConfig, ApplicationConfig};

pub mod app_state;

pub use app_state::{AppState, IdentifierKey};

pub mod state;

pub use state::{NowPlayingInfo, PlaybackControls, PlayerState, ProgressSlider, VolumeControl};

pub mod services;

pub mod client;
pub mod util;

pub mod consts;

pub mod pane;
pub mod dock;

pub mod yunara_player;

pub use yunara_player::YunaraPlayer;
pub use pane::{Axis, Pane, PaneGroup, PaneItem, PaneItemHandle};
pub use dock::{Dock, DockPanel, DockPanelHandle, DockPosition};
