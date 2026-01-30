/// Navigation models for routing within the application.
///
/// Defines the routes available in the app and navigation item structures
/// for the sidebar.

use gpui::SharedString;

/// Application routes for navigation.
///
/// Represents the different views/pages in the application that can be
/// navigated to via the sidebar or other navigation elements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// Home page with recommendations and recent activity
    Home,
    /// Explore page for discovering new music
    Explore,
    /// User's library with saved content
    Library,
    /// Specific playlist detail view
    Playlist(String),
}

impl Route {
    /// Returns whether this route represents a playlist detail view.
    pub fn is_playlist(&self) -> bool {
        matches!(self, Self::Playlist(_))
    }

    /// Returns the playlist ID if this is a playlist route.
    pub fn playlist_id(&self) -> Option<&str> {
        match self {
            Self::Playlist(id) => Some(id),
            _ => None,
        }
    }
}

/// Icon identifier for navigation items.
///
/// Represents the available icons that can be displayed alongside
/// navigation labels in the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// Home icon
    Home,
    /// Search/explore icon
    Explore,
    /// Library/bookshelf icon
    Library,
    /// Heart/favorite icon
    Heart,
    /// Plus/add icon
    Plus,
    /// Music note icon
    Music,
}

/// Navigation item for sidebar display.
///
/// Combines an icon, display label, and target route for rendering
/// clickable navigation links.
#[derive(Debug, Clone)]
pub struct NavItem {
    /// Icon to display alongside the label
    pub icon: Icon,
    /// Display text for the navigation item
    pub label: SharedString,
    /// Target route when clicked
    pub route: Route,
}

impl NavItem {
    /// Creates a new navigation item.
    pub fn new(icon: Icon, label: impl Into<SharedString>, route: Route) -> Self {
        Self {
            icon,
            label: label.into(),
            route,
        }
    }

    /// Creates the default set of main navigation items.
    pub fn default_nav_items() -> Vec<Self> {
        vec![
            Self::new(Icon::Home, "Home", Route::Home),
            Self::new(Icon::Explore, "Explore", Route::Explore),
            Self::new(Icon::Library, "Library", Route::Library),
        ]
    }
}
