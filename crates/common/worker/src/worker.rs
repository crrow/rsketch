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

use crate::context::WorkerContext;

/// Core worker trait defining execution logic and lifecycle hooks.
///
/// Workers are stateful tasks that execute according to a trigger schedule.
/// The trait is deliberately simple - configuration (name, trigger, blocking)
/// is handled by the builder API, not by trait methods.
///
/// # Generic State
///
/// All methods are generic over state type `S` to allow the same Worker
/// implementation to work with different state types. The Manager determines
/// the concrete state type.
///
/// # Lifecycle
///
/// 1. `on_start` - Called once before first work execution
/// 2. `work` - Called repeatedly according to trigger schedule
/// 3. `on_shutdown` - Called once during graceful shutdown
///
/// # Example
///
/// ```rust
/// use rsketch_common_worker::{Worker, WorkerContext};
///
/// struct MyWorker {
///     counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
/// }
///
/// #[async_trait::async_trait]
/// impl Worker for MyWorker {
///     async fn on_start<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {
///         println!("Worker {} starting", ctx.name());
///     }
///
///     async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {
///         self.counter
///             .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
///
///         // Check for cancellation
///         if ctx.is_cancelled() {
///             return;
///         }
///     }
///
///     async fn on_shutdown<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {
///         println!("Worker {} shutting down", ctx.name());
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Worker: Send + 'static {
    /// Called once when the worker starts, before the first work execution.
    ///
    /// Use this hook for initialization logic that should happen once.
    /// This method should not panic or fail - any errors should be logged.
    ///
    /// Default implementation does nothing.
    async fn on_start<S: Clone + Send + Sync>(&mut self, _ctx: WorkerContext<S>) {}

    /// Core work unit, called according to the trigger schedule.
    ///
    /// This is where your main worker logic goes. The method will be called:
    /// - Once for `Trigger::Once`
    /// - On demand for `Trigger::Notify`
    /// - Periodically for `Trigger::Interval` and `Trigger::Cron`
    /// - Combination for hybrid triggers
    ///
    /// Use `ctx.is_cancelled()` to check for shutdown requests.
    /// The work should be atomic or idempotent when possible.
    async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>);

    /// Called once during graceful shutdown, after the last work execution.
    ///
    /// Use this hook for cleanup logic like flushing buffers or closing
    /// connections. This method should not panic or fail - any errors
    /// should be logged.
    ///
    /// Default implementation does nothing.
    async fn on_shutdown<S: Clone + Send + Sync>(&mut self, _ctx: WorkerContext<S>) {}
}
