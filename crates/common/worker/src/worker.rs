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

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync::Notify;

use crate::{
    context::WorkerContext,
    err::Result,
    metrics::{WORKER_PAUSED, WORKER_RESUMED},
};

/// Trigger mechanism for worker execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// Execute once immediately on startup, then never again.
    Once,
    /// Execute on every notification via `WorkerHandle::notify()`.
    Notify,
    /// Execute periodically at the specified interval.
    Interval(std::time::Duration),
}

/// Core worker trait for background tasks.
///
/// Implementors only need to define single-shot execution logic in `work()`.
/// The framework handles looping, triggering, and lifecycle management.
#[async_trait::async_trait]
pub trait Worker: Send + 'static {
    /// Worker name for logging and debugging.
    fn name() -> &'static str
    where
        Self: Sized;

    /// Execution trigger strategy.
    fn trigger() -> Trigger
    where
        Self: Sized;

    /// Whether this worker should run on a blocking thread pool.
    /// Default: false (runs on async runtime).
    fn is_blocking() -> bool
    where
        Self: Sized,
    {
        false
    }

    /// Called once when the worker starts, before the first `work()` execution.
    /// Use this for initialization logic.
    async fn on_start(&mut self, _ctx: &WorkerContext) -> Result<()> { Ok(()) }

    /// Single execution unit. This is called each time the trigger fires.
    /// Return `Err` to stop the worker immediately.
    async fn work(&mut self, ctx: &WorkerContext) -> Result<()>;

    /// Called once when the worker is shutting down, after the last `work()`
    /// execution. Use this for cleanup logic. This is called even if the
    /// worker is aborted.
    async fn on_shutdown(&mut self, _ctx: &WorkerContext) -> Result<()> { Ok(()) }
}

/// Handle to control a running worker.
#[derive(Clone)]
pub struct WorkerHandle {
    name:   &'static str,
    notify: Arc<Notify>,
    paused: Arc<AtomicBool>,
}

impl WorkerHandle {
    pub(crate) fn new(name: &'static str, notify: Arc<Notify>, paused: Arc<AtomicBool>) -> Self {
        WorkerHandle {
            name,
            notify,
            paused,
        }
    }

    pub fn name(&self) -> &'static str { self.name }

    /// Notify the worker to wake up (e.g., for new work).
    pub fn notify(&self) { self.notify.notify_one(); }

    /// Pause the worker. The worker will stop executing but remain alive.
    pub fn pause(&self) {
        self.paused.store(true, Ordering::Release);
        WORKER_PAUSED.with_label_values(&[self.name]).inc();
    }

    /// Resume a paused worker.
    pub fn resume(&self) {
        self.paused.store(false, Ordering::Release);
        self.notify.notify_one(); // Wake up the worker
        WORKER_RESUMED.with_label_values(&[self.name]).inc();
    }

    /// Check if the worker is currently paused.
    pub fn is_paused(&self) -> bool { self.paused.load(Ordering::Acquire) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    struct ExampleWorker {
        counter: i64,
    }

    #[async_trait::async_trait]
    impl Worker for ExampleWorker {
        fn name() -> &'static str { "ExampleWorker" }

        fn trigger() -> Trigger { Trigger::Notify }

        async fn work(&mut self, _ctx: &WorkerContext) -> Result<()> {
            self.counter += 1;
            tracing::info!("Worker executed, counter = {}", self.counter);
            Ok(())
        }
    }

    #[test]
    fn test_trait_object_safety() {
        // Ensures Worker trait is properly defined
    }
}
