# Playlist Integration Design

**Date**: 2026-02-02
**Status**: Design
**Author**: Claude Code

## Overview

This design document outlines the integration of YouTube Music playlist functionality into the Yunara desktop application. The implementation enables users to view their playlist library in the sidebar and access detailed playlist information including song lists.

### Goals

- Display user's playlists in the sidebar
- Load playlists asynchronously on application startup
- Open playlist details view showing songs when clicked
- Cache playlist details for performance
- Support refresh functionality (via keyboard shortcuts)

### Non-Goals

- Playlist editing/creation (future work)
- Playlist playback controls (future work)
- UI refresh buttons (using keyboard shortcuts instead)
- Loading spinners/progress indicators (silent loading)

## Architecture

### Service Layer Pattern

We introduce a `PlaylistService` layer to separate business logic from UI components:

```
UI Components (Sidebar, PlaylistView)
    ↓
AppState (holds service reference)
    ↓
PlaylistService (business logic + caching)
    ↓
ApiClient (API calls)
```

**Benefits**:
- Separation of concerns: AppState doesn't contain playlist logic
- Testability: Can mock PlaylistService
- Extensibility: Can add SearchService, PlayerService, etc.
- Thread-safe: Uses Arc + RwLock

## Data Structures

### PlaylistService

```rust
use moka::future::Cache;
use std::sync::{Arc, atomic::AtomicBool};
use parking_lot::RwLock;

pub struct PlaylistService {
    api_client: ApiClient,

    // Playlist list state (loaded once on startup)
    playlists: Arc<RwLock<Vec<LibraryPlaylist>>>,
    playlists_loaded: Arc<AtomicBool>,

    // Playlist details cache (on-demand loading)
    details_cache: Cache<String, Vec<PlaylistItem>>,
}
```

**Design Decisions**:
- **Playlist list**: Simple `RwLock` storage, loaded once (user playlists don't change frequently)
- **Playlist details**: moka Cache with TTL and LRU (song lists are large and need eviction)
- **Why not cache the list?**: Playlist lists are small (~dozens), details are large (~hundreds of songs)

### Cache Configuration

```rust
Cache::builder()
    .max_capacity(100)                      // Max 100 playlists cached
    .time_to_live(Duration::from_secs(3600))   // 1 hour expiration
    .time_to_idle(Duration::from_secs(600))    // 10 min idle expiration
    .build()
```

### AppState Integration

```rust
struct AppStateInner {
    // ... existing fields ...
    playlist_service: Arc<PlaylistService>,
}

impl AppState {
    pub fn playlist_service(&self) -> Arc<PlaylistService> {
        self.inner.playlist_service.clone()
    }
}
```

## Data Flow

### Application Startup

```
Application Start
    ↓
Create ApiClient
    ↓
Create PlaylistService(api_client)
    ↓
Create AppState(playlist_service)
    ↓
Spawn background task → playlist_service.load_playlists()
    ↓                              ↓
UI renders                    API call get_library_playlists()
    ↓                              ↓
Sidebar shows empty           Success → Update playlists
    ↓                              ↓
                           Notify UI → Sidebar re-renders with list
```

**Key Points**:
- Non-blocking: Background task doesn't block UI
- Silent failure: Errors only logged, no user interruption
- Reactive update: UI automatically updates when data loads

### Playlist Click Flow

```
User clicks "Japanese" playlist in Sidebar
    ↓
Trigger on_click → handle_playlist_click()
    ↓
Call player.open_playlist(id, name)
    ↓
Create PlaylistView(app_state, id, name)
    ↓
Add to main_pane
    ↓
PlaylistView renders and triggers load_tracks()
    ↓
Call service.get_playlist_details(id)
    ↓
Check moka cache
    ↓
├─ Cache hit → Return immediately
└─ Cache miss → API call get_playlist_songs()
    ↓
Update view.tracks
    ↓
cx.notify() triggers re-render
    ↓
Display song list
```

### Refresh Flow (Future)

```
User presses refresh shortcut
    ↓
Trigger refresh action
    ↓
Call service.refresh_all()
    ↓
Invalidate all cache
    ↓
Reload playlists from API
    ↓
Notify UI to update
```

## Component Design

### PlaylistService API

```rust
impl PlaylistService {
    pub fn new(api_client: ApiClient) -> Self;

    // Load/refresh playlist list
    pub async fn load_playlists(&self) -> Result<()>;

    // Get list (sync, for UI rendering)
    pub fn get_playlists(&self) -> Vec<LibraryPlaylist>;

    // Get details (async, with caching)
    pub async fn get_playlist_details(&self, playlist_id: &str)
        -> Result<Vec<PlaylistItem>>;

    // Refresh all (for keyboard shortcuts)
    pub async fn refresh_all(&self) -> Result<()>;

    // Refresh single playlist
    pub async fn refresh_playlist(&self, playlist_id: &str) -> Result<()>;
}
```

### Sidebar Integration

**Changes**:
1. Read playlists from `app_state.playlist_service().get_playlists()`
2. Replace placeholder text with actual playlist items
3. Add click handler to open PlaylistView

```rust
impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let playlists = self.app_state.playlist_service().get_playlists();

        // ... existing code ...

        // Replace lines 305-312 placeholder
        .child(
            div()
                .flex_1()
                .overflow_y_scroll()
                .px(px(12.0))
                .children(playlists.into_iter().map(|playlist| {
                    // Render playlist item with click handler
                }))
        )
    }
}
```

### PlaylistView Integration

**Changes**:
1. Add state fields: `tracks`, `loading`, `load_error`
2. Load data on creation via `load_tracks()`
3. Render song list from loaded data

```rust
pub struct PlaylistView {
    weak_self: WeakEntity<Self>,
    app_state: AppState,
    playlist_id: String,
    playlist_name: String,

    // New fields
    tracks: Vec<PlaylistItem>,
    loading: bool,
    load_error: Option<String>,
}

impl PlaylistView {
    pub fn new(...) -> Self {
        let mut view = Self { ... };
        view.load_tracks(cx);
        view
    }

    fn load_tracks(&mut self, cx: &mut Context<Self>) {
        let service = self.app_state.playlist_service();
        // Spawn async task to load and update
    }
}
```

## Error Handling

### Strategy: Silent Loading

Per requirements, we use silent loading with minimal user disruption:

1. **Startup load failure**: Log error, sidebar shows empty
2. **Playlist detail load failure**: Log error, view shows empty
3. **No error messages or toasts**: Keep UI clean

### Implementation

```rust
// Startup
tokio::spawn(async move {
    if let Err(e) = service.load_playlists().await {
        tracing::error!("Failed to load playlists on startup: {}", e);
    }
});

// Detail loading
match service.get_playlist_details(&id).await {
    Ok(tracks) => {
        view.tracks = tracks;
        view.load_error = None;
    }
    Err(e) => {
        view.tracks = Vec::new();
        view.load_error = Some(e.to_string());
        tracing::error!("Failed to load playlist {}: {}", id, e);
    }
}
```

### Edge Cases

1. **Empty playlist list**: Show "No playlists yet"
2. **Empty playlist details**: Show "This playlist is empty"
3. **Prevent duplicate loading**: Check `loading` flag before loading
4. **Concurrent access**: RwLock and moka Cache are thread-safe

## Implementation Plan

### Phase 1: Service Layer
1. Create `PlaylistService` in `crates/desktop/yunara-player/src/services/playlist_service.rs`
2. Add moka dependency to `Cargo.toml`
3. Implement service methods with caching
4. Add tracing for debugging

### Phase 2: AppState Integration
1. Add `playlist_service` field to `AppStateInner`
2. Initialize service in `AppState::new()`
3. Spawn background task for initial load
4. Add accessor method `playlist_service()`

### Phase 3: Sidebar Integration
1. Update `Sidebar::render()` to read playlists
2. Replace placeholder with real playlist items
3. Add click handler to open PlaylistView
4. Handle empty state

### Phase 4: PlaylistView Integration
1. Add state fields to `PlaylistView`
2. Implement `load_tracks()` method
3. Update render to display songs
4. Add error handling and edge cases

### Phase 5: Testing & Polish
1. Test startup flow
2. Test click flow
3. Test error cases (network failure, etc.)
4. Add logging statements
5. Verify cache behavior

## Future Enhancements

- Refresh keyboard shortcut
- Playlist creation/editing
- Drag-and-drop song reordering
- Playlist search/filtering
- Cache persistence across restarts
- Optimistic UI updates

## Dependencies

**New**:
- `moka = "0.12"` - High-performance caching

**Existing**:
- `ytmapi_rs` - API client
- `parking_lot` - RwLock
- `tracing` - Logging
- `gpui` - UI framework

## Testing Strategy

### Unit Tests
- `PlaylistService::load_playlists()` - Success and failure cases
- `PlaylistService::get_playlist_details()` - Cache hit/miss scenarios
- `PlaylistService::refresh_all()` - Cache invalidation

### Integration Tests
- Startup flow with mock API
- Click flow with mock service
- Error scenarios with failing API

### Manual Testing
- Load with real YouTube Music account
- Click various playlists
- Test with empty playlist list
- Test with empty playlists
- Verify cache behavior (fast second load)

## References

- [YouTube Music UI Design](docs/youtube-music-ui-improvements.md)
- [Sidebar Compact Layout](docs/sidebar-compact-layout-design.md)
- ytmapi_rs documentation
- moka documentation
