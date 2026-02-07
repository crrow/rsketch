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

//! Playlist service for managing playlist data with caching.
//!
//! Provides async loading of user playlists and playlist details,
//! with moka-based caching for playlist song lists.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use moka::future::Cache;
use parking_lot::RwLock;
use tracing::info;
use ytmapi_rs::{
    common::YoutubeID,
    parse::{LibraryPlaylist, PlaylistItem},
};

use crate::ytapi::client::ApiClient;

/// Service for managing playlist data with caching.
///
/// Holds the user's playlist library list (loaded once on startup)
/// and caches playlist details (songs) on demand.
pub struct PlaylistService {
    api_client: ApiClient,

    /// Playlist library list (loaded once on startup)
    playlists: Arc<RwLock<Vec<LibraryPlaylist>>>,

    /// Whether playlists have been loaded at least once
    playlists_loaded: Arc<AtomicBool>,

    /// Cache for playlist detail (song lists), keyed by playlist ID
    details_cache: Cache<String, Vec<PlaylistItem>>,
}

impl PlaylistService {
    /// Creates a new PlaylistService.
    pub fn new(api_client: ApiClient) -> Self {
        let details_cache = Cache::builder()
            .max_capacity(100)
            .time_to_live(Duration::from_secs(3600))
            .time_to_idle(Duration::from_secs(600))
            .build();

        Self {
            api_client,
            playlists: Arc::new(RwLock::new(Vec::new())),
            playlists_loaded: Arc::new(AtomicBool::new(false)),
            details_cache,
        }
    }

    /// Loads the user's playlist library from the API.
    ///
    /// Updates the internal playlist list on success.
    pub async fn load_playlists(&self) -> crate::ytapi::err::Result<()> {
        info!("Loading library playlists");
        let result = self.api_client.get_library_playlists().await?;
        info!("Loaded {} playlists", result.len());

        {
            let mut playlists = self.playlists.write();
            *playlists = result;
        }
        self.playlists_loaded.store(true, Ordering::Release);
        Ok(())
    }

    /// Returns the current playlist list (sync, for UI rendering).
    pub fn get_playlists(&self) -> Vec<LibraryPlaylist> {
        self.playlists.read().clone()
    }

    /// Returns whether playlists have been loaded.
    pub fn is_loaded(&self) -> bool {
        self.playlists_loaded.load(Ordering::Acquire)
    }

    /// Gets playlist details (songs) with caching.
    ///
    /// Returns cached data if available, otherwise fetches from the API.
    pub async fn get_playlist_details(
        &self,
        playlist_id: &str,
    ) -> crate::ytapi::err::Result<Vec<PlaylistItem>> {
        // Check cache first
        if let Some(cached) = self.details_cache.get(playlist_id).await {
            return Ok(cached);
        }

        // Cache miss - fetch from API
        info!(playlist_id, "Loading playlist details");
        let pid = ytmapi_rs::common::PlaylistID::from_raw(playlist_id.to_owned());
        let items = self.api_client.get_playlist_songs(pid, 500).await?;
        info!(playlist_id, count = items.len(), "Loaded playlist details");

        // Store in cache
        self.details_cache
            .insert(playlist_id.to_owned(), items.clone())
            .await;

        Ok(items)
    }

    /// Gets first page of playlist details (50 items) with continuation token
    pub async fn get_playlist_first_page(
        &self,
        playlist_id: &str,
    ) -> crate::ytapi::err::Result<crate::ytapi::client::PlaylistPage> {
        info!(playlist_id, "Loading playlist first page");
        let pid = ytmapi_rs::common::PlaylistID::from_raw(playlist_id.to_owned());
        let page = self.api_client.get_playlist_first_page(pid).await?;
        info!(playlist_id, count = page.items.len(), has_more = page.continuation.is_some(), "Loaded playlist first page");
        Ok(page)
    }

    /// Gets next page of playlist details using continuation token
    pub async fn get_playlist_next_page(
        &self,
        continuation: String,
    ) -> crate::ytapi::err::Result<crate::ytapi::client::PlaylistPage> {
        info!("Loading playlist next page");
        let page = self.api_client.get_playlist_next_page(continuation).await?;
        info!(count = page.items.len(), has_more = page.continuation.is_some(), "Loaded playlist next page");
        Ok(page)
    }

    /// Refreshes all data: invalidates cache and reloads playlists.
    pub async fn refresh_all(&self) -> crate::ytapi::err::Result<()> {
        info!("Refreshing all playlist data");
        self.details_cache.invalidate_all();
        self.load_playlists().await
    }

    /// Refreshes a single playlist's details by invalidating its cache entry.
    pub async fn refresh_playlist(&self, playlist_id: &str) {
        self.details_cache.invalidate(playlist_id).await;
    }
}
