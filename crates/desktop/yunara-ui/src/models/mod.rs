/// Data models shared across UI components.
///
/// This module contains the core data structures used throughout the Yunara
/// music player UI, including track metadata, playlist information, and
/// navigation types.

mod navigation;
mod playlist;
mod track;

pub use navigation::{Icon, NavItem, Route};
pub use playlist::{Playlist, PlaylistSummary, SortOrder, Visibility};
pub use track::Track;
