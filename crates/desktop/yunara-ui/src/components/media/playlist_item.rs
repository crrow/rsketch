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

/// Playlist item component for sidebar display.
///
/// Shows playlist name and owner in a compact format suitable for
/// the sidebar playlist listing.
use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, Window, div, prelude::*, px,
};

use crate::{components::theme::ThemeExt, models::PlaylistSummary};

/// A compact playlist item for sidebar listings.
///
/// Layout:
/// ```text
/// ┌─────────────────────┐
/// │  Playlist Name      │
/// │  owner name         │
/// └─────────────────────┘
/// ```
#[derive(IntoElement)]
pub struct PlaylistItem {
    id:          ElementId,
    playlist:    PlaylistSummary,
    is_selected: bool,
    on_click:    Option<Box<dyn Fn(&PlaylistSummary, &mut Window, &mut App) + 'static>>,
}

impl PlaylistItem {
    /// Creates a new PlaylistItem for the given playlist summary.
    pub fn new(id: impl Into<ElementId>, playlist: PlaylistSummary) -> Self {
        Self {
            id: id.into(),
            playlist,
            is_selected: false,
            on_click: None,
        }
    }

    /// Sets whether this playlist is currently selected.
    pub fn selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    /// Sets the click handler for when the playlist is selected.
    pub fn on_click(
        mut self,
        handler: impl Fn(&PlaylistSummary, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for PlaylistItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let playlist = self.playlist.clone();

        div()
            .id(self.id)
            .w_full()
            .px(px(12.0))
            .py(px(8.0))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .hover(|style| style.bg(theme.hover))
            .when(self.is_selected, |el| el.bg(theme.active))
            .when_some(self.on_click, |el, handler| {
                el.on_click(move |_event, window, cx| {
                    handler(&playlist, window, cx);
                })
            })
            .child(
                div()
                    .text_color(if self.is_selected {
                        theme.text_primary
                    } else {
                        theme.text_primary
                    })
                    .text_size(px(14.0))
                    .line_height(px(18.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(self.playlist.name.clone()),
            )
            .child(
                div()
                    .text_color(theme.text_secondary)
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(self.playlist.owner.clone()),
            )
    }
}
