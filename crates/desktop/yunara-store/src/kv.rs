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

use serde::{Serialize, de::DeserializeOwned};
use snafu::ResultExt;
use sqlx::SqlitePool;

use crate::err::*;

/// Key-value store backed by SQLite
///
/// All values are serialized to JSON before storage
#[derive(Clone)]
pub struct KVStore {
    pool: SqlitePool,
}

impl KVStore {
    /// Create a new KV store from a SQLite pool
    pub(crate) fn new(pool: SqlitePool) -> Self { Self { pool } }

    /// Set a key-value pair
    ///
    /// The value will be serialized to JSON before storage
    ///
    /// # Arguments
    /// * `key` - The key to store
    /// * `value` - The value to store (must implement Serialize)
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let value_json = serde_json::to_string(value).context(CodecSnafu)?;

        sqlx::query("INSERT OR REPLACE INTO kv_table (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value_json)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get a value by key
    ///
    /// Returns `None` if the key does not exist
    /// The value will be deserialized from JSON
    ///
    /// # Arguments
    /// * `key` - The key to retrieve
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM kv_table WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some((value_json,)) => {
                let value = serde_json::from_str(&value_json).context(CodecSnafu)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Remove a key-value pair
    ///
    /// # Arguments
    /// * `key` - The key to remove
    pub async fn remove(&self, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM kv_table WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    use crate::db::DBStore;

    #[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
    struct Person {
        name: String,
        age:  i32,
    }

    #[tokio::test]
    async fn kv_store_test() {
        let tempdir = TempDir::new().unwrap();
        let db_path = tempdir.path().join("test.db");

        let db = DBStore::new(&db_path).await.unwrap();
        let kv = db.kv_store();

        // Run migrations
        sqlx::migrate!("./migrations").run(db.pool()).await.unwrap();

        // Test string
        kv.set("str_key", &"hello".to_string()).await.unwrap();
        assert_eq!(kv.get::<String>("str_key").await.unwrap().unwrap(), "hello");
        assert_eq!(kv.get::<String>("nonexistent").await.unwrap(), None);

        // Test bool
        kv.set("bool_key", &true).await.unwrap();
        assert_eq!(kv.get::<bool>("bool_key").await.unwrap().unwrap(), true);

        // Test i64
        kv.set("i64_key", &42i64).await.unwrap();
        assert_eq!(kv.get::<i64>("i64_key").await.unwrap().unwrap(), 42);

        // Test object
        let person = Person {
            name: "nathan".to_string(),
            age:  30,
        };
        kv.set("person_key", &person).await.unwrap();
        assert_eq!(
            kv.get::<Person>("person_key").await.unwrap().unwrap(),
            person
        );

        // Test remove
        kv.remove("str_key").await.unwrap();
        assert_eq!(kv.get::<String>("str_key").await.unwrap(), None);
    }

    #[tokio::test]
    async fn kv_store_overwrite_test() {
        let tempdir = TempDir::new().unwrap();
        let db_path = tempdir.path().join("test.db");

        let db = DBStore::new(&db_path).await.unwrap();
        let kv = db.kv_store();

        // Run migrations
        sqlx::migrate!("./migrations").run(db.pool()).await.unwrap();

        kv.set("key", &"first").await.unwrap();
        assert_eq!(kv.get::<String>("key").await.unwrap().unwrap(), "first");

        kv.set("key", &"second").await.unwrap();
        assert_eq!(kv.get::<String>("key").await.unwrap().unwrap(), "second");
    }
}
