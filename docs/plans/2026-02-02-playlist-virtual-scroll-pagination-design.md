# Playlist Virtual Scroll with Pagination Design

**Date**: 2026-02-02
**Status**: Approved
**Author**: Claude (Brainstorming Session)

## Overview

Implement virtual scrolling with paginated data loading for playlist detail view to improve performance and user experience when browsing large playlists.

## Requirements

- **Virtual Scrolling**: Only render visible items (fixed height: 60px per item)
- **Pagination**: Load playlist data in pages of 50 items
- **Auto-loading**: Trigger next page load when user scrolls near bottom (20 items threshold)
- **API Support**: Utilize ytmapi-rs continuation tokens for true pagination

## Architecture

### Three-Layer Changes

```
┌─────────────────────────────────────────────────┐
│  UI Layer (PlaylistView)                        │
│  - Virtual scroll state & rendering              │
│  - Pagination state & triggers                   │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│  Service Layer (PlaylistService)                │
│  - Pass-through for paginated API calls         │
│  - No caching (data lives in UI layer)          │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│  API Client Layer (ApiClient)                   │
│  - Continuation token management                │
│  - First page vs. next page methods             │
└─────────────────────────────────────────────────┘
```

## Component Designs

### 1. API Client Layer

**Location**: `crates/desktop/yunara-player/src/ytapi/client.rs`

#### New Type

```rust
/// Represents a page of playlist items with optional continuation
pub struct PlaylistPage {
    pub items: Vec<PlaylistItem>,
    pub continuation: Option<String>,  // Continuation token for next page
}
```

#### New Methods

```rust
impl ApiClient {
    /// Get the first page of a playlist
    pub async fn get_playlist_first_page(
        &self,
        playlist_id: PlaylistID<'static>,
    ) -> Result<PlaylistPage> {
        // Use ytmapi_rs::query::GetPlaylistTracksQuery
        // Call parse_from_continuable() to get (items, continuation)
    }

    /// Get next page using continuation token
    pub async fn get_playlist_next_page(
        &self,
        continuation: String,
    ) -> Result<PlaylistPage> {
        // Use ytmapi_rs::query::GetContinuationsQuery
        // Call parse_continuation() to get next page
    }
}
```

#### Existing Method

Keep `get_playlist_songs()` for backward compatibility, but it will only be used by non-paginated scenarios.

### 2. Service Layer

**Location**: `crates/desktop/yunara-player/src/services/playlist_service.rs`

#### Caching Strategy

**Decision**: Remove `details_cache` for paginated playlists.

**Rationale**:
- Data already lives in `PlaylistView::tracks`
- Avoids duplicate memory usage
- Simplifies implementation
- Trade-off: Need to reload when switching playlists (acceptable UX cost)

#### New Methods

```rust
impl PlaylistService {
    /// Get playlist first page (50 items)
    pub async fn get_playlist_first_page(
        &self,
        playlist_id: &str,
    ) -> crate::ytapi::err::Result<PlaylistPage> {
        let pid = ytmapi_rs::common::PlaylistID::from_raw(playlist_id.to_owned());
        self.api_client.get_playlist_first_page(pid).await
    }

    /// Get next page using continuation token
    pub async fn get_playlist_next_page(
        &self,
        continuation: String,
    ) -> crate::ytapi::err::Result<PlaylistPage> {
        self.api_client.get_playlist_next_page(continuation).await
    }
}
```

### 3. UI Layer - Virtual Scroll

**Location**: `crates/desktop/yunara-player/src/pane/items/playlist_view.rs`

#### State Extensions

```rust
pub struct PlaylistView {
    // Existing fields
    weak_self: WeakEntity<Self>,
    app_state: AppState,
    playlist_id: String,
    playlist_name: String,
    thumbnail_url: Option<String>,
    gradient_top_color: Option<Rgba>,
    blurred_background_path: Option<PathBuf>,
    blur_target_size: Option<(u32, u32)>,
    blur_in_flight: bool,
    tracks: Vec<PlaylistItem>,
    loading: bool,

    // NEW: Pagination state
    continuation_token: Option<String>,  // Token for next page
    has_more: bool,                      // Whether more data exists
    loading_more: bool,                  // Whether loading next page

    // NEW: Virtual scroll state
    scroll_offset: f32,                  // Current scroll position (pixels)
}
```

#### Constants

```rust
const ITEM_HEIGHT: f32 = 60.0;  // Fixed height per playlist item
const LOAD_THRESHOLD: usize = 20;  // Trigger load when 20 items remaining
```

#### Key Methods

**Initial Load**:
```rust
fn load_first_page(&mut self, cx: &mut Context<Self>) {
    self.loading = true;
    let service = self.app_state.playlist_service().clone();
    let playlist_id = self.playlist_id.clone();

    let tokio_task = gpui_tokio::Tokio::spawn(cx, async move {
        service.get_playlist_first_page(&playlist_id).await
    });

    cx.spawn(async move |this, cx| {
        let result = tokio_task.await.unwrap();
        let _ = cx.update(|cx| {
            this.update(cx, |view, cx| {
                match result {
                    Ok(page) => {
                        view.tracks = page.items;
                        view.continuation_token = page.continuation;
                        view.has_more = page.continuation.is_some();
                    }
                    Err(e) => {
                        tracing::error!("Failed to load first page: {}", e);
                        view.tracks = Vec::new();
                        view.has_more = false;
                    }
                }
                view.loading = false;
                cx.notify();
            })
        });
    }).detach();
}
```

**Load More**:
```rust
fn load_next_page(&mut self, cx: &mut Context<Self>) {
    // Guard: only load if not already loading and more data exists
    if self.loading_more || !self.has_more {
        return;
    }

    let Some(token) = self.continuation_token.clone() else {
        return;
    };

    self.loading_more = true;
    let service = self.app_state.playlist_service().clone();

    let tokio_task = gpui_tokio::Tokio::spawn(cx, async move {
        service.get_playlist_next_page(token).await
    });

    cx.spawn(async move |this, cx| {
        let result = tokio_task.await.unwrap();
        let _ = cx.update(|cx| {
            this.update(cx, |view, cx| {
                match result {
                    Ok(page) => {
                        view.tracks.extend(page.items);
                        view.continuation_token = page.continuation;
                        view.has_more = page.continuation.is_some();
                    }
                    Err(e) => {
                        tracing::error!("Failed to load next page: {}", e);
                        view.has_more = false;
                    }
                }
                view.loading_more = false;
                cx.notify();
            })
        });
    }).detach();
}
```

**Check Load Trigger**:
```rust
fn check_load_more(&mut self, cx: &mut Context<Self>) {
    if self.tracks.is_empty() {
        return;
    }

    // Calculate visible range
    let viewport_height = cx.window().map(|w| f32::from(w.viewport_size().height))
        .unwrap_or(600.0);
    let first_visible = (self.scroll_offset / ITEM_HEIGHT).floor() as usize;
    let visible_count = (viewport_height / ITEM_HEIGHT).ceil() as usize + 2;
    let last_visible = (first_visible + visible_count).min(self.tracks.len());

    // Check if near bottom
    let remaining = self.tracks.len().saturating_sub(last_visible);
    if remaining < LOAD_THRESHOLD && self.has_more && !self.loading_more {
        self.load_next_page(cx);
    }
}
```

#### Render Method

**Virtual Scroll Structure**:
```rust
fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
    // ... existing header code ...

    // Calculate visible range
    let viewport_height = f32::from(window.viewport_size().height);
    let first_visible = (self.scroll_offset / ITEM_HEIGHT).floor() as usize;
    let visible_count = (viewport_height / ITEM_HEIGHT).ceil() as usize + 2; // +2 buffer
    let last_visible = (first_visible + visible_count).min(self.tracks.len());

    let visible_tracks = if self.tracks.is_empty() {
        &[]
    } else {
        &self.tracks[first_visible..last_visible]
    };

    // Total content height for scroll container
    let total_height = self.tracks.len() as f32 * ITEM_HEIGHT;

    gpui::div()
        .id("playlist-view")
        // ... existing background/header rendering ...
        .child(
            gpui::div()
                .id("playlist-tracks-scroll")
                .flex()
                .flex_col()
                .flex_1()
                .overflow_y_scroll()
                .on_scroll(cx.listener(|this, event, cx| {
                    this.scroll_offset = event.position.y;
                    this.check_load_more(cx);
                    cx.notify();
                }))
                // Top spacer (for items before visible range)
                .child(gpui::div().h(px(first_visible as f32 * ITEM_HEIGHT)))
                // Visible items
                .children(visible_tracks.iter().enumerate().map(|(idx, item)| {
                    let actual_index = first_visible + idx;
                    // ... render track item (existing code) ...
                }))
                // Bottom spacer (for items after visible range)
                .child(
                    gpui::div()
                        .h(px((self.tracks.len() - last_visible) as f32 * ITEM_HEIGHT))
                        .when(self.loading_more, |el| {
                            el.child(
                                gpui::div()
                                    .text_sm()
                                    .text_color(theme.text_muted)
                                    .child("Loading more...")
                            )
                        })
                )
        )
}
```

## Data Flow

```
User Opens Playlist
    ↓
PlaylistView::new()
    ↓
load_first_page()
    ↓
Service::get_playlist_first_page()
    ↓
ApiClient::get_playlist_first_page()
    ↓
Returns: PlaylistPage { items: [50 tracks], continuation: Some(token) }
    ↓
Update: tracks, continuation_token, has_more = true
    ↓
render() - Show first 50 items (only ~10-15 visible)
    ↓
─────────────────────────────────────────────
User Scrolls Down
    ↓
on_scroll() updates scroll_offset
    ↓
check_load_more()
    ↓
Condition: remaining < 20 && has_more && !loading_more
    ↓
load_next_page()
    ↓
Service::get_playlist_next_page(token)
    ↓
ApiClient::get_playlist_next_page(token)
    ↓
Returns: PlaylistPage { items: [next 50], continuation: Some(token2) }
    ↓
tracks.extend(new items) - Now have 100 items
    ↓
render() - Update visible range, continue scrolling
```

## Performance Characteristics

### Memory Usage

- **Before**: All playlist items in memory at once (500+ items × ~1KB = 500KB+)
- **After**: Incrementally loaded (50 items at a time), only loaded pages in memory
- **Rendering**: Only 10-15 items in DOM at any time (vs 500+ before)

### User Experience

- **Initial Load**: ~100-200ms for first 50 items (vs 1-2s for 500 items)
- **Scrolling**: Smooth 60fps (virtual scroll reduces DOM size)
- **Subsequent Pages**: Load in background while user scrolls
- **Network**: Distributed API calls (less burst load)

## Error Handling

- **First Page Failure**: Show error state, allow retry
- **Next Page Failure**: Log error, set `has_more = false`, stop pagination
- **Token Expiry**: Treat as end of data, stop loading

## Testing Strategy

1. **Unit Tests**: Test visible range calculation logic
2. **Integration Tests**: Test pagination state transitions
3. **Manual Testing**:
   - Small playlists (<50 items) - single page, no pagination
   - Medium playlists (50-200 items) - multiple pages
   - Large playlists (500+ items) - extensive scrolling
   - Slow network - verify loading states
   - Error scenarios - verify graceful degradation

## Future Enhancements

- Add pull-to-refresh for reloading playlist
- Implement search/filter within loaded tracks
- Add jump-to-position for large playlists
- Consider caching strategy for frequently accessed playlists
- Implement bi-directional virtual scroll if needed

## Implementation Order

1. **Phase 1**: API Client changes (add `PlaylistPage` type and methods)
2. **Phase 2**: Service layer updates (add pagination methods, remove cache)
3. **Phase 3**: UI state changes (add pagination and scroll state fields)
4. **Phase 4**: Virtual scroll rendering (update `render()` method)
5. **Phase 5**: Pagination triggers (implement `check_load_more()`)
6. **Phase 6**: Testing and refinement

## References

- ytmapi-rs documentation: `ParseFromContinuable` trait
- GPUI scroll events: `on_scroll()` handler
- Current implementation: `playlist_view.rs:502-577`
