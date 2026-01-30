/// Queue panel component for displaying the playback queue.
///
/// Shows the current playback queue with tabs for Up Next, Lyrics, and Related.
/// Only visible when a playlist is actively playing.

use gpui::{
    div, prelude::*, px, App, ElementId, InteractiveElement, IntoElement, ParentElement, Styled,
    Window,
};

use crate::{
    components::{media::TrackItem, theme::ThemeExt},
    models::Track,
};

/// Tab options for the queue panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueueTab {
    /// Shows upcoming tracks in the queue
    #[default]
    UpNext,
    /// Shows lyrics for the current track (not implemented in MVP)
    Lyrics,
    /// Shows related tracks (not implemented in MVP)
    Related,
}

impl QueueTab {
    fn label(&self) -> &'static str {
        match self {
            Self::UpNext => "UP NEXT",
            Self::Lyrics => "LYRICS",
            Self::Related => "RELATED",
        }
    }
}

/// Right-side queue panel showing playback queue.
///
/// Layout:
/// ```text
/// ┌─────────────────────────┐
/// │ [UP NEXT] LYRICS RELATED│
/// ├─────────────────────────┤
/// │  ▶ Track 1 (playing)    │
/// │    Track 2              │
/// │    Track 3              │
/// │    ...                  │
/// └─────────────────────────┘
/// ```
#[derive(IntoElement)]
pub struct QueuePanel {
    id: ElementId,
    active_tab: QueueTab,
    queue: Vec<Track>,
    current_index: Option<usize>,
    on_tab_change: Option<Box<dyn Fn(QueueTab, &mut Window, &mut App) + 'static>>,
    on_track_select: Option<Box<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
}

impl QueuePanel {
    /// Creates a new queue panel.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            active_tab: QueueTab::default(),
            queue: Vec::new(),
            current_index: None,
            on_tab_change: None,
            on_track_select: None,
        }
    }

    /// Sets the queue of tracks.
    pub fn queue(mut self, queue: Vec<Track>) -> Self {
        self.queue = queue;
        self
    }

    /// Sets the currently playing track index.
    pub fn current_index(mut self, index: Option<usize>) -> Self {
        self.current_index = index;
        self
    }

    /// Sets the active tab.
    pub fn active_tab(mut self, tab: QueueTab) -> Self {
        self.active_tab = tab;
        self
    }

    /// Sets the tab change handler.
    pub fn on_tab_change(
        mut self,
        handler: impl Fn(QueueTab, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_tab_change = Some(Box::new(handler));
        self
    }

    /// Sets the track selection handler.
    pub fn on_track_select(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_track_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for QueuePanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let id = self.id;
        let active_tab = self.active_tab;
        let current_index = self.current_index;
        let queue = self.queue;

        div()
            .id(id)
            .w(px(320.0))
            .h_full()
            .bg(theme.background_secondary)
            .border_l_1()
            .border_color(theme.border)
            .flex()
            .flex_col()
            // Tab bar
            .child(
                div()
                    .px(px(16.0))
                    .py(px(12.0))
                    .flex()
                    .gap(px(16.0))
                    .children([QueueTab::UpNext, QueueTab::Lyrics, QueueTab::Related].map(|tab| {
                        let is_active = tab == active_tab;

                        div()
                            .id(format!("tab-{:?}", tab))
                            .text_size(px(11.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .cursor_pointer()
                            .text_color(if is_active {
                                theme.text_primary
                            } else {
                                theme.text_muted
                            })
                            .when(is_active, |el| {
                                el.border_b_2().border_color(theme.accent).pb(px(2.0))
                            })
                            .hover(|style| style.text_color(theme.text_primary))
                            .child(tab.label())
                    })),
            )
            // Divider
            .child(div().w_full().h(px(1.0)).bg(theme.border))
            // Queue content
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .py(px(8.0))
                    .flex()
                    .flex_col()
                    .children(queue.into_iter().enumerate().map(|(index, track)| {
                        let is_playing = current_index == Some(index);

                        TrackItem::new(format!("queue-track-{}", index), track)
                            .playing(is_playing)
                            .with_index(index)
                    })),
            )
    }
}
