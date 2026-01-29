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

//! User service module
//!
//! This is an example of how to implement business services using the extension trait pattern.

use std::sync::Arc;

use crate::app_state::AppState;

/// User service handler
///
/// This is a placeholder for actual user service implementation
pub struct UserServiceHandler {
    // Service dependencies would go here
    // For example: database, cache, etc.
}

impl UserServiceHandler {
    /// Create a new user service handler
    pub fn new() -> Self { Self {} }

    /// Example method: list users
    pub async fn list_users(&self) -> anyhow::Result<Vec<String>> {
        // Placeholder implementation
        Ok(vec!["user1".to_string(), "user2".to_string()])
    }

    /// Example method: get user by id
    pub async fn get_user(&self, _id: &str) -> anyhow::Result<Option<String>> {
        // Placeholder implementation
        Ok(None)
    }
}

// ============================================================================
// Extension Trait for AppState
// ============================================================================

/// Extension trait for accessing the user service
pub trait UserServiceExt {
    /// Get a reference to the user service
    fn user_service(&self) -> &Arc<UserServiceHandler>;
}

impl UserServiceExt for AppState {
    fn user_service(&self) -> &Arc<UserServiceHandler> {
        // This would access self.inner.user once the field is added
        // For now, this is just a placeholder to show the pattern
        todo!("Add user field to AppStateInner")
    }
}
