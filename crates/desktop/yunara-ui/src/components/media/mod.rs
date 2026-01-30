/// Media display components for tracks and playlists.
///
/// Contains reusable components for displaying music content:
/// - `TrackItem`: Single track in a list
/// - `CoverMosaic`: Playlist cover art grid
/// - `PlaylistItem`: Playlist entry in sidebar

mod cover_mosaic;
mod playlist_item;
mod track_item;

pub use cover_mosaic::CoverMosaic;
pub use playlist_item::PlaylistItem;
pub use track_item::TrackItem;
