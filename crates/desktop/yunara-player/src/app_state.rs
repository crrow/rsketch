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

use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use strum::{EnumProperty, IntoEnumIterator};
use yunara_store::{
    DBStore,
    kv::{IdType, KVStoreExt, KeyRequest},
};

use crate::{config::AppConfig, state::PlayerState, ytapi::client::ApiClient};

/// Application state that holds the lifecycle of the desktop application
///
/// This state is shared across the application and contains:
/// - Database connection and pool
/// - Application configuration
/// - Business services (accessible via extension traits)
///
/// # Extension Trait Pattern
///
/// Business services are accessed through extension traits to keep
/// the code modular and decoupled. Each module can define its own extension
/// trait.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub fn config(&self) -> AppConfig { self.inner.config.clone() }

    pub fn db(&self) -> DBStore { self.inner.db.clone() }

    pub fn player_state(&self) -> &RwLock<PlayerState> { &self.inner.player_state }

    pub fn api_client(&self) -> ApiClient { self.inner.api_client.clone() }
}

struct AppStateInner {
    config:       AppConfig,
    db:           DBStore,
    keys:         HashMap<String, IdType>,
    session_id:   uuid::Uuid,
    player_state: RwLock<PlayerState>,
    api_client:   ApiClient,
}

impl AppState {
    /// Create a new AppState with the given configuration
    ///
    /// # Arguments
    /// * `config` - Application configuration
    ///
    /// # Errors
    /// Returns an error if database initialization fails
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        let db = DBStore::new(config.database.clone()).await?;

        // Batch initialize all identifier keys
        let key_requests: Vec<KeyRequest> = IdentifierKey::iter().map(|k| k.into()).collect();
        let key_map = db.kv_store().batch_get_or_init_keys(key_requests).await?;

        // Initialize API client with Browser auth
        let api_client = ApiClient::open(
            crate::ytapi::AuthType::Browser,
            &yunara_paths::config_dir(),
            std::time::Duration::from_secs(10),
        )
        .await?;

        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                db,
                keys: key_map,
                session_id: uuid::Uuid::new_v4(),
                player_state: RwLock::new(PlayerState::new()),
                api_client,
            }),
        })
    }

    pub fn get_session_id(&self) -> uuid::Uuid { self.inner.session_id }

    pub fn get_key_value(&self, key: &IdentifierKey) -> IdType {
        self.inner
            .keys
            .get(key.as_ref())
            .cloned()
            .expect("Get key value: key should exist as it was initialized on AppState creation")
    }
}

#[derive(
    Debug,
    Clone,
    strum_macros::EnumString,
    strum_macros::AsRefStr,
    strum_macros::EnumIter,
    strum_macros::EnumProperty,
)]
#[strum(serialize_all = "snake_case")]
pub enum IdentifierKey {
    #[strum(props(force = "true"))]
    SessionId,
    #[strum(props(force = "false"))]
    SystemId,
    #[strum(props(force = "false"))]
    InstallationId,
}

impl Into<KeyRequest> for IdentifierKey {
    fn into(self) -> KeyRequest {
        KeyRequest::builder()
            .key(self.as_ref())
            .force(
                self.get_str("force")
                    .expect("force property should exist")
                    .parse::<bool>()
                    .expect("force property should be a valid bool"),
            )
            .build()
    }
}
