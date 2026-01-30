/// Sidebar component for navigation and playlist listing.
///
/// Contains main navigation items (Home, Explore, Library) and
/// a scrollable list of user playlists.

use gpui::{
    div, prelude::*, px, App, ElementId, IntoElement, ParentElement, Styled, Window,
};

use crate::{
    components::{media::PlaylistItem, theme::ThemeExt},
    models::{Icon, NavItem, PlaylistSummary, Route},
};

/// Left sidebar with navigation and playlists.
///
/// Layout:
/// ```text
/// ┌─────────────────────┐
/// │  🏠 Home            │
/// │  🔍 Explore         │
/// │  📚 Library         │
/// ├─────────────────────┤
/// │  ♥ Liked Music      │
/// │─────────────────────│
/// │  Playlist 1         │
/// │  Playlist 2         │
/// │  ...                │
/// └─────────────────────┘
/// ```
#[derive(IntoElement)]
pub struct Sidebar {
    id: ElementId,
    nav_items: Vec<NavItem>,
    playlists: Vec<PlaylistSummary>,
    current_route: Option<Route>,
    on_navigate: Option<Box<dyn Fn(Route, &mut Window, &mut App) + 'static>>,
}

impl Sidebar {
    /// Creates a new sidebar.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            nav_items: NavItem::default_nav_items(),
            playlists: Vec::new(),
            current_route: None,
            on_navigate: None,
        }
    }

    /// Sets the playlist list.
    pub fn playlists(mut self, playlists: Vec<PlaylistSummary>) -> Self {
        self.playlists = playlists;
        self
    }

    /// Sets the current active route.
    pub fn current_route(mut self, route: Route) -> Self {
        self.current_route = Some(route);
        self
    }

    /// Sets the navigation handler.
    pub fn on_navigate(mut self, handler: impl Fn(Route, &mut Window, &mut App) + 'static) -> Self {
        self.on_navigate = Some(Box::new(handler));
        self
    }

    fn icon_char(icon: &Icon) -> &'static str {
        match icon {
            Icon::Home => "🏠",
            Icon::Explore => "🔍",
            Icon::Library => "📚",
            Icon::Heart => "♥",
            Icon::Plus => "+",
            Icon::Music => "♪",
        }
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let id = self.id;
        let current_route = self.current_route;
        let playlists = self.playlists;

        div()
            .id(id)
            .w(px(240.0))
            .h_full()
            .bg(theme.background_secondary)
            .border_r_1()
            .border_color(theme.border)
            .flex()
            .flex_col()
            .overflow_hidden()
            // Navigation items
            .child(
                div()
                    .p(px(12.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .children(self.nav_items.into_iter().map(|item| {
                        let is_active = current_route
                            .as_ref()
                            .map(|r| *r == item.route)
                            .unwrap_or(false);
                        let icon = Self::icon_char(&item.icon);

                        div()
                            .id(format!("nav-{:?}", item.route))
                            .w_full()
                            .px(px(12.0))
                            .py(px(8.0))
                            .flex()
                            .items_center()
                            .gap(px(12.0))
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_color(if is_active {
                                theme.text_primary
                            } else {
                                theme.text_secondary
                            })
                            .when(is_active, |el| el.bg(theme.active))
                            .hover(|style| style.bg(theme.hover))
                            .child(div().text_size(px(16.0)).child(icon))
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .font_weight(if is_active {
                                        gpui::FontWeight::SEMIBOLD
                                    } else {
                                        gpui::FontWeight::NORMAL
                                    })
                                    .child(item.label),
                            )
                    })),
            )
            // Divider
            .child(div().mx(px(12.0)).h(px(1.0)).bg(theme.border))
            // Playlists section
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .p(px(12.0))
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .children(playlists.into_iter().enumerate().map(|(index, playlist)| {
                        let is_selected = current_route
                            .as_ref()
                            .map(|r| r.playlist_id() == Some(&playlist.id))
                            .unwrap_or(false);

                        PlaylistItem::new(format!("playlist-{}", index), playlist)
                            .selected(is_selected)
                    })),
            )
    }
}
