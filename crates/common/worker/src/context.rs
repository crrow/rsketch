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

use std::sync::Arc;

use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

/// Context passed to each worker instance.
#[derive(Clone)]
pub struct WorkerContext {
    cancel_token: CancellationToken,
    notify:       Arc<Notify>,
}

impl WorkerContext {
    pub(crate) fn new(cancel_token: CancellationToken, notify: Arc<Notify>) -> Self {
        WorkerContext {
            cancel_token,
            notify,
        }
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool { self.cancel_token.is_cancelled() }

    /// Wait for cancellation signal.
    pub async fn cancelled(&self) { self.cancel_token.cancelled().await }

    /// Wait for a notification (e.g., new work available).
    pub async fn notified(&self) { self.notify.notified().await }

    /// Get a child cancellation token for sub-tasks.
    pub fn child_token(&self) -> CancellationToken { self.cancel_token.child_token() }
}
