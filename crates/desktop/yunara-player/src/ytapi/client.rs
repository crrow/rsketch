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

use std::{
    borrow::Borrow,
    convert::TryFrom,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use futures::{StreamExt, TryStreamExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rusty_ytdl::reqwest;
use tokio::sync::{Notify, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument};
use ytmapi_rs::{
    YtMusic, YtMusicBuilder,
    auth::{BrowserToken, OAuthToken, noauth::NoAuthToken},
    common::{PlaylistID, SearchSuggestion, UserChannelID, UserPlaylistsParams},
    continuations::ParseFromContinuable,
    parse::{
        GetUser, LibraryChannel, LibraryPlaylist, ParseFrom, PlaylistItem, SearchResultArtist,
        SearchResultPlaylist, SearchResultProfile, SearchResultSong, UserPlaylist,
    },
    query::{
        GetLibraryChannelsQuery, GetLibraryPlaylistsQuery, GetUserPlaylistsQuery, GetUserQuery,
        PostQuery, Query, SearchQuery,
        search::{FilteredSearch, ProfilesFilter},
    },
};

use crate::ytapi::{
    ApiKey, AuthType,
    err::{InvalidAuthTokenSnafu, OperationCancelledSnafu, Result, TokenRefreshFailedSnafu},
};

#[derive(Clone)]
pub struct ApiClient {
    inner: Arc<ApiClientInner>,
}

impl ApiClient {
    pub async fn open<P: ?Sized + AsRef<Path>>(
        auth_type: AuthType,
        config_path: &P,
        timeout: std::time::Duration,
    ) -> Result<Self> {
        let apikey = auth_type.load(config_path).await?;
        // Cheaply cloneable reqwest client to share amongst services.
        use rusty_ytdl::reqwest;
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .timeout(timeout)
            .build()
            .expect("Expected reqwest client build to succeed");

        let is_oauth = matches!(apikey, ApiKey::OAuthToken(_));
        let inner = GenericalYtmusic::new(apikey, client).await?;

        let generic_client = Arc::new(RwLock::new(inner));
        let token_state = Arc::new(AtomicU8::new(TokenState::NoNeed as u8));
        let notify = Arc::new(Notify::new());
        let cancel_token = CancellationToken::new();

        // Conditionally start worker (OAuth only)
        let worker_notify = if is_oauth {
            let worker_notify = Arc::new(Notify::new());

            tokio::spawn(
                RefreshWorker {
                    generic_client: generic_client.clone(),
                    token_state:    token_state.clone(),
                    worker_notify:  worker_notify.clone(),
                    notify:         notify.clone(),
                    cancel_token:   cancel_token.child_token(),
                    config_dir:     config_path.as_ref().to_path_buf(),
                }
                .run(),
            );

            Some(worker_notify)
        } else {
            None
        };

        Ok(Self {
            inner: Arc::new(ApiClientInner {
                generic_client,
                token_state,
                notify,
                worker_notify,
                cancel_token,
                timeout,
            }),
        })
    }

    #[instrument(skip(self), err(Display))]
    pub async fn get_search_suggestions(&self, text: &str) -> Result<Vec<SearchSuggestion>> {
        self.inner
            .query_api_with_retry(&ytmapi_rs::query::GetSearchSuggestionsQuery::new(text))
            .await
    }

    #[instrument(skip(self), err(Display))]
    pub async fn search_playlists(&self, text: &str) -> Result<Vec<SearchResultPlaylist>> {
        let query = ytmapi_rs::query::SearchQuery::new(text)
            .with_filter(ytmapi_rs::query::search::PlaylistsFilter)
            .with_spelling_mode(ytmapi_rs::query::search::SpellingMode::ExactMatch);
        self.inner.query_api_with_retry(&query).await
    }

    #[instrument(skip(self), err(Display))]
    pub async fn search_artists(&self, text: &str) -> Result<Vec<SearchResultArtist>> {
        let query = ytmapi_rs::query::SearchQuery::new(text)
            .with_filter(ytmapi_rs::query::search::ArtistsFilter)
            .with_spelling_mode(ytmapi_rs::query::search::SpellingMode::ExactMatch);
        self.inner.query_api_with_retry(&query).await
    }

    #[instrument(skip(self), err(Display))]
    pub async fn search_songs(&self, text: &str) -> Result<Vec<SearchResultSong>> {
        let query = ytmapi_rs::query::SearchQuery::new(text)
            .with_filter(ytmapi_rs::query::search::SongsFilter)
            .with_spelling_mode(ytmapi_rs::query::search::SpellingMode::ExactMatch);
        self.inner.query_api_with_retry(&query).await
    }

    #[instrument(skip(self), err(Display))]
    pub async fn get_playlist_songs(
        &self,
        playlist_id: PlaylistID<'static>,
        max_results: usize,
    ) -> Result<Vec<PlaylistItem>> {
        let query = ytmapi_rs::query::GetPlaylistTracksQuery::new((&playlist_id).into());
        self.inner.query_api_with_retry(&query).await
    }

    /// API Search Query for Profiles only.
    pub async fn search_profiles<'a, Q: Into<SearchQuery<'a, FilteredSearch<ProfilesFilter>>>>(
        &self,
        query: Q,
    ) -> Result<Vec<SearchResultProfile>> {
        let query = query.into();
        self.inner.query_api_with_retry(&query).await
    }

    /// Gets information about an user and their videos and playlists.
    pub async fn get_user<'a>(&self, id: impl Into<UserChannelID<'a>>) -> Result<GetUser> {
        self.inner
            .query_api_with_retry(&GetUserQuery::new(id.into()))
            .await
    }

    /// Gets a full list of playlists for a user.
    pub async fn get_user_playlists<
        'a,
        T: Into<UserChannelID<'a>>,
        U: Into<UserPlaylistsParams<'a>>,
    >(
        &self,
        channel_id: T,
        browse_params: U,
    ) -> Result<Vec<UserPlaylist>> {
        let query = GetUserPlaylistsQuery::new(channel_id.into(), browse_params.into());
        self.inner.query_api_with_retry(&query).await
    }

    /// Gets a list of all playlists in your Library.
    #[instrument(skip(self), err(Display))]
    pub async fn get_library_playlists(&self) -> Result<Vec<LibraryPlaylist>> {
        let query = GetLibraryPlaylistsQuery;
        self.inner.query_api_with_retry(&query).await
    }

    /// Gets a list of all channels in your Library.
    #[instrument(skip(self), err(Display))]
    pub async fn get_library_channels(&self) -> Result<Vec<LibraryChannel>> {
        let query = GetLibraryChannelsQuery::default();
        self.inner.query_api_with_retry(&query).await
    }
}

#[derive(IntoPrimitive, TryFromPrimitive, Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
enum TokenState {
    NoNeed = 0,
    NeedRefreshing = 1,
    InRefreshing = 2,
    RefreshFailed = 3,
}

struct RefreshWorker {
    generic_client: Arc<RwLock<GenericalYtmusic>>,
    token_state:    Arc<AtomicU8>,
    worker_notify:  Arc<Notify>,
    notify:         Arc<Notify>,
    cancel_token:   CancellationToken,
    config_dir:     PathBuf,
}

impl RefreshWorker {
    async fn run(self) {
        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    info!("refresh worker exiting");
                    // Set to failed state to wake up waiting requests
                    self.token_state.store(TokenState::RefreshFailed as u8, Ordering::Release);
                    // Wake up all waiting requests
                    self.notify.notify_waiters();
                    break;
                }
                _ = self.worker_notify.notified() => {
                    let state = TokenState::try_from(self.token_state.load(Ordering::Acquire))
                        .expect("TokenState must be valid");
                    if state == TokenState::NeedRefreshing {
                        self.token_state.store(TokenState::InRefreshing as u8, Ordering::Release);

                        let refresh_result = {
                            let mut client = self.generic_client.write().await;
                            client.refresh_token().await
                        };

                        match refresh_result {
                            Ok(Some(token)) => {
                                info!("OAuth token refreshed");
                                if let Err(e) = ApiKey::OAuthToken(token)
                                    .update_file(&self.config_dir)
                                    .await
                                {
                                    error!("Error persisting refreshed oauth token: {e}");
                                }
                                self.token_state.store(TokenState::NoNeed as u8, Ordering::Release);
                            }
                            Ok(None) => {
                                // Browser/NoAuth - should not happen
                                self.token_state.store(TokenState::NoNeed as u8, Ordering::Release);
                            }
                            Err(e) => {
                                error!("OAuth token refresh failed: {e}");
                                self.token_state.store(TokenState::RefreshFailed as u8, Ordering::Release);
                            }
                        }

                        self.notify.notify_waiters();
                    }
                }
            }
        }
    }
}

pub struct ApiClientInner {
    generic_client: Arc<RwLock<GenericalYtmusic>>,
    token_state:    Arc<AtomicU8>,
    notify:         Arc<Notify>,
    worker_notify:  Option<Arc<Notify>>,
    cancel_token:   CancellationToken,
    timeout:        std::time::Duration,
}

impl ApiClientInner {
    /// Run a query. If the oauth token is expired, take the lock and refresh
    /// it (single retry only). If another error occurs, try a single retry too.
    #[instrument(skip(self, query), err(Display))]
    pub async fn query_api_with_retry<Q, O>(&self, query: &Q) -> Result<O>
    where
        Q: ytmapi_rs::query::Query<BrowserToken, Output = O>,
        Q: ytmapi_rs::query::Query<OAuthToken, Output = O>,
    {
        const MAX_RETRIES: u32 = 2;
        let mut retries = 0u32;
        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    return OperationCancelledSnafu.fail();
                }
                result = async {
                    // 1. Register notified BEFORE state check to avoid lost wakeup
                    let notified = self.notify.notified();
                    let state = TokenState::try_from(self.token_state.load(Ordering::Acquire))
                        .expect("TokenState must be valid");
                    match state {
                        TokenState::RefreshFailed => {
                            return Some(TokenRefreshFailedSnafu.fail());
                        }
                        TokenState::InRefreshing | TokenState::NeedRefreshing => {
                            debug!(?state, "token refresh in progress, waiting");
                            tokio::select! {
                                _ = self.cancel_token.cancelled() => {
                                    return Some(OperationCancelledSnafu.fail());
                                }
                                _ = tokio::time::sleep(self.timeout) => {
                                    debug!("timed out waiting for token refresh");
                                    return Some(TokenRefreshFailedSnafu.fail());
                                }
                                _ = notified => {
                                    debug!("token refresh completed, retrying");
                                    return None; // Continue loop
                                }
                            }
                        }
                        TokenState::NoNeed => { /* continue */ }
                    }

                    // 2. Execute request
                    let result = {
                        let client = self.generic_client.read().await;
                        client.query_browser_or_oauth(query).await
                    };

                    // 3. Handle result
                    match result {
                        Ok(output) => Some(Ok(output)),
                        Err(e) => {
                            // Check if this is an OAuth token expired error
                            // Note: ytmapi_rs::Error doesn't implement Clone, so we check via Display
                            let should_refresh = matches!(
                                &e,
                                crate::ytapi::err::Error::YtmapiError { source, .. }
                                    if source.to_string().contains("OAuth") && source.to_string().contains("expired")
                            );

                            if should_refresh {
                                retries += 1;
                                if retries > MAX_RETRIES {
                                    debug!(retries, "max retries exceeded");
                                    return Some(TokenRefreshFailedSnafu.fail());
                                }

                                debug!(retries, "token expired, requesting refresh");

                                // Register notification first (avoid lost wakeup)
                                let notified = self.notify.notified();

                                // Try CAS
                                let cas_result = self.token_state.compare_exchange(
                                    TokenState::NoNeed as u8,
                                    TokenState::NeedRefreshing as u8,
                                    Ordering::AcqRel,
                                    Ordering::Acquire,
                                );

                                if cas_result.is_ok() {
                                    debug!("CAS succeeded, notifying refresh worker");
                                    if let Some(worker_notify) = &self.worker_notify {
                                        worker_notify.notify_one();
                                    }
                                } else {
                                    debug!("refresh already in progress, waiting");
                                }

                                // Wait for refresh to complete
                                tokio::select! {
                                    _ = self.cancel_token.cancelled() => {
                                        return Some(OperationCancelledSnafu.fail());
                                    }
                                    _ = tokio::time::sleep(self.timeout) => {
                                        debug!("timed out waiting for token refresh");
                                        return Some(TokenRefreshFailedSnafu.fail());
                                    }
                                    _ = notified => {
                                        debug!("refresh done, retrying request");
                                        None // Continue loop
                                    }
                                }
                            } else {
                                Some(Err(e))
                            }
                        }
                    }
                } => {
                    if let Some(result) = result {
                        return result;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum GenericalYtmusic {
    Browser(YtMusic<BrowserToken>),
    OAuth(YtMusic<OAuthToken>),
    NoAuth(YtMusic<NoAuthToken>),
}

impl GenericalYtmusic {
    async fn new(key: ApiKey, client: reqwest::Client) -> Result<Self> {
        match key {
            ApiKey::BrowserToken(cookie) => Ok(GenericalYtmusic::Browser(
                YtMusicBuilder::new_with_client(ytmapi_rs::Client::new_from_reqwest_client(client))
                    .with_browser_token_cookie(cookie)
                    .build()
                    .await?,
            )),
            ApiKey::OAuthToken(token) => Ok(GenericalYtmusic::OAuth(
                YtMusicBuilder::new_rustls_tls()
                    .with_auth_token(token)
                    .build()?,
            )),
            ApiKey::None => Ok(GenericalYtmusic::NoAuth(
                YtMusicBuilder::new_rustls_tls().build().await?,
            )),
        }
    }

    // TO DETERMINE HOW TO HANDLE BROWSER/NOAUTH CASE.
    async fn refresh_token(&mut self) -> Result<Option<OAuthToken>> {
        Ok(match self {
            GenericalYtmusic::Browser(_) | GenericalYtmusic::NoAuth(_) => None,
            GenericalYtmusic::OAuth(yt) => Some(yt.refresh_token().await?),
        })
    }

    // TO DETERMINE HOW TO HANDLE BROWSER/NOAUTH CASE.
    fn get_token_hash(&self) -> Result<Option<u64>> {
        Ok(match self {
            GenericalYtmusic::Browser(_) | GenericalYtmusic::NoAuth(_) => None,
            GenericalYtmusic::OAuth(yt) => Some(yt.get_token_hash()),
        })
    }

    async fn query<Q, O>(&self, query: impl Borrow<Q>) -> Result<O>
    where
        Q: Query<BrowserToken, Output = O>,
        Q: Query<OAuthToken, Output = O>,
        Q: Query<NoAuthToken, Output = O>,
    {
        Ok(match self {
            GenericalYtmusic::Browser(yt) => yt.query(query).await?,
            GenericalYtmusic::OAuth(yt) => yt.query(query).await?,
            GenericalYtmusic::NoAuth(yt) => yt.query(query).await?,
        })
    }

    async fn query_browser_or_oauth<'a, Q, O>(&self, query: &'a Q) -> Result<O>
    where
        Q: Query<BrowserToken, Output = O> + 'a,
        Q: Query<OAuthToken, Output = O>,
        &'a Q: Borrow<Q>,
    {
        Ok(match self {
            GenericalYtmusic::Browser(yt) => yt.query(query).await?,
            GenericalYtmusic::OAuth(yt) => yt.query(query).await?,
            GenericalYtmusic::NoAuth(_) => InvalidAuthTokenSnafu {
                current_authtype:   AuthType::Unauthenticated,
                expected_authtypes: vec![AuthType::Browser, AuthType::OAuth],
            }
            .fail()?,
        })
    }

    async fn _stream<Q, O>(&self, query: impl Borrow<Q>, max_pages: usize) -> Result<Vec<O>>
    where
        Q: Query<BrowserToken, Output = O>,
        Q: Query<OAuthToken, Output = O>,
        Q: Query<NoAuthToken, Output = O>,
        O: ParseFromContinuable<Q>,
        Q: PostQuery,
    {
        Ok(match self {
            GenericalYtmusic::Browser(yt) => {
                yt.stream(query.borrow())
                    .take(max_pages)
                    .try_collect()
                    .await?
            }
            GenericalYtmusic::OAuth(yt) => {
                yt.stream(query.borrow())
                    .take(max_pages)
                    .try_collect()
                    .await?
            }
            GenericalYtmusic::NoAuth(yt) => {
                yt.stream(query.borrow())
                    .take(max_pages)
                    .try_collect()
                    .await?
            }
        })
    }

    async fn stream_browser_or_oauth<Q, O>(
        &self,
        query: impl Borrow<Q>,
        max_pages: usize,
    ) -> Result<Vec<O>>
    where
        Q: Query<BrowserToken, Output = O>,
        Q: Query<OAuthToken, Output = O>,
        O: ParseFromContinuable<Q>,
        Q: PostQuery,
    {
        Ok(match self {
            GenericalYtmusic::Browser(yt) => {
                yt.stream(query.borrow())
                    .take(max_pages)
                    .try_collect()
                    .await?
            }
            GenericalYtmusic::OAuth(yt) => {
                yt.stream(query.borrow())
                    .take(max_pages)
                    .try_collect()
                    .await?
            }
            GenericalYtmusic::NoAuth(_) => InvalidAuthTokenSnafu {
                current_authtype:   AuthType::Unauthenticated,
                expected_authtypes: vec![AuthType::Browser, AuthType::OAuth],
            }
            .fail()?,
        })
    }

    async fn query_source<Q, O>(&self, query: impl Borrow<Q>) -> Result<String>
    where
        Q: Query<BrowserToken, Output = O>,
        Q: Query<OAuthToken, Output = O>,
        Q: Query<NoAuthToken, Output = O>,
    {
        Ok(match self {
            GenericalYtmusic::Browser(yt) => yt.raw_json_query(query).await?,
            GenericalYtmusic::OAuth(yt) => yt.raw_json_query(query).await?,
            GenericalYtmusic::NoAuth(yt) => yt.raw_json_query(query).await?,
        })
    }

    async fn query_source_browser_or_oauth<Q, O>(&self, query: impl Borrow<Q>) -> Result<String>
    where
        Q: Query<BrowserToken, Output = O>,
        Q: Query<OAuthToken, Output = O>,
    {
        Ok(match self {
            GenericalYtmusic::Browser(yt) => yt.raw_json_query(query).await?,
            GenericalYtmusic::OAuth(yt) => yt.raw_json_query(query).await?,
            GenericalYtmusic::NoAuth(_) => InvalidAuthTokenSnafu {
                current_authtype:   AuthType::Unauthenticated,
                expected_authtypes: vec![AuthType::Browser, AuthType::OAuth],
            }
            .fail()?,
        })
    }

    async fn _stream_source<Q, O>(&self, query: &Q, max_pages: usize) -> Result<String>
    where
        Q: Query<BrowserToken, Output = O>,
        Q: Query<OAuthToken, Output = O>,
        Q: Query<NoAuthToken, Output = O>,
        Q: PostQuery,
        O: ParseFromContinuable<Q>,
    {
        // If only one page, no need to stream.
        if max_pages == 1 {
            return self.query_source::<Q, O>(query).await;
        }
        Ok(match self {
            GenericalYtmusic::Browser(yt) => {
                yt.raw_json_stream(query.borrow())
                    .take(max_pages)
                    .try_collect()
                    .await?
            }
            GenericalYtmusic::OAuth(yt) => {
                yt.raw_json_stream(query.borrow())
                    .take(max_pages)
                    .try_collect()
                    .await?
            }
            GenericalYtmusic::NoAuth(yt) => {
                yt.raw_json_stream(query.borrow())
                    .take(max_pages)
                    .try_collect()
                    .await?
            }
        })
    }

    async fn stream_source_browser_or_oauth<Q, O>(
        &self,
        query: &Q,
        max_pages: usize,
    ) -> Result<String>
    where
        Q: Query<BrowserToken, Output = O>,
        Q: Query<OAuthToken, Output = O>,
        Q: PostQuery,
        O: ParseFromContinuable<Q>,
        O: ParseFrom<Q>,
    {
        // If only one page, no need to stream.
        if max_pages == 1 {
            return self.query_source_browser_or_oauth::<Q, O>(query).await;
        }
        Ok(match self {
            GenericalYtmusic::Browser(yt) => {
                yt.raw_json_stream(query.borrow())
                    .take(max_pages)
                    .try_collect()
                    .await?
            }
            GenericalYtmusic::OAuth(yt) => {
                yt.raw_json_stream(query.borrow())
                    .take(max_pages)
                    .try_collect()
                    .await?
            }
            GenericalYtmusic::NoAuth(_) => InvalidAuthTokenSnafu {
                current_authtype:   AuthType::Unauthenticated,
                expected_authtypes: vec![AuthType::Browser, AuthType::OAuth],
            }
            .fail()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ytmusic() {
        let _guard = rsketch_common_telemetry::logging::init_tracing_subscriber("test");

        let config_path = yunara_paths::config_dir();
        let client = ApiClient::open(
            AuthType::Browser,
            &config_path,
            std::time::Duration::from_secs(3),
        )
        .await
        .unwrap();

        let channels = client.get_library_channels().await.unwrap();
        println!("{:?}", channels);

        let playlists = client.get_library_playlists().await.unwrap();
        println!("{:?}", playlists);

        let v = client.get_search_suggestions("reol").await.unwrap();
        println!("{:?}", v)
    }
}
