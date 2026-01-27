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

use std::path::Path;

use sqlx::{
    Sqlite, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::{err::*, kv::KVStore};

/// Database store that manages the SQLite connection pool
#[derive(Clone)]
pub struct DBStore {
    pool: SqlitePool,
}

impl DBStore {
    /// Create a new database store
    ///
    /// # Arguments
    /// * `db_path` - Path to the SQLite database file
    #[tracing::instrument(level = "trace", err)]
    pub async fn new(db_path: impl AsRef<Path> + std::fmt::Debug) -> Result<Self> {
        let db_path = db_path.as_ref();

        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(options)
            .await?;

        tracing::trace!("Initialized DBStore with path: {}", db_path.display());

        Ok(Self { pool })
    }

    /// Get a KV store instance
    pub fn kv_store(&self) -> KVStore { KVStore::new(self.pool.clone()) }

    /// Get the underlying SQLite pool
    pub fn pool(&self) -> &SqlitePool { &self.pool }

    /// Acquire a connection from the pool
    pub async fn acquire(&self) -> Result<sqlx::pool::PoolConnection<Sqlite>> {
        Ok(self.pool.acquire().await?)
    }
}
