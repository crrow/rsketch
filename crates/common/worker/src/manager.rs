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

use rsketch_common_runtime::Runtime;
use tokio::{sync::Notify, task::JoinSet};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::{
    config::WorkerConfig,
    context::WorkerContext,
    err::Result,
    metrics::{
        WORKER_ACTIVE, WORKER_ERRORS, WORKER_EXECUTION_DURATION_SECONDS, WORKER_EXECUTION_ERRORS,
        WORKER_EXECUTIONS, WORKER_SHUTDOWN_ERRORS, WORKER_START_ERRORS, WORKER_STARTED,
        WORKER_STOPPED,
    },
    worker::{Trigger, Worker, WorkerHandle},
};

/// Manages lifecycle of multiple background workers.
pub struct Manager {
    cancel_token:     CancellationToken,
    runtime:          Option<Arc<Runtime>>,
    shutdown_timeout: std::time::Duration,
    joins:            JoinSet<Result<()>>,
}

impl Manager {
    /// Create a new worker manager.
    pub fn start(config: WorkerConfig) -> Result<Self> {
        Ok(Manager {
            cancel_token:     CancellationToken::new(),
            runtime:          config.runtime(),
            shutdown_timeout: config.shutdown_timeout(),
            joins:            JoinSet::new(),
        })
    }

    /// Register a new worker and return its handle.
    ///
    /// The worker starts immediately in a background task.
    pub fn register<W>(&mut self, mut worker: W) -> WorkerHandle
    where
        W: Worker,
    {
        let name = W::name();
        let trigger = W::trigger();
        let notify = Arc::new(Notify::new());
        let paused = Arc::new(AtomicBool::new(false));
        let ctx = WorkerContext::new(self.cancel_token.child_token(), notify.clone());

        let paused_clone = paused.clone();
        let task = async move {
            info!(worker = name, trigger = ?trigger, "Worker starting");
            WORKER_STARTED.with_label_values(&[name]).inc();
            WORKER_ACTIVE.with_label_values(&[name]).set(1);

            // Call on_start hook
            if let Err(e) = worker.on_start(&ctx).await {
                error!(worker = name, error = ?e, "Worker failed during on_start");
                WORKER_START_ERRORS.with_label_values(&[name]).inc();
                WORKER_ACTIVE.with_label_values(&[name]).set(0);
                return Err(e);
            }

            let result = Self::run_loop(&mut worker, &ctx, &paused_clone, trigger, name).await;

            // Always call on_shutdown, even if work failed
            if let Err(e) = worker.on_shutdown(&ctx).await {
                error!(worker = name, error = ?e, "Worker failed during on_shutdown");
                WORKER_SHUTDOWN_ERRORS.with_label_values(&[name]).inc();
            }

            match &result {
                Ok(_) => {
                    info!(worker = name, "Worker stopped gracefully");
                    WORKER_STOPPED.with_label_values(&[name]).inc();
                }
                Err(e) => {
                    error!(worker = name, error = ?e, "Worker failed");
                    WORKER_ERRORS.with_label_values(&[name]).inc();
                }
            }
            WORKER_ACTIVE.with_label_values(&[name]).set(0);
            result
        };

        let runtime = self
            .runtime
            .clone()
            .unwrap_or_else(rsketch_common_runtime::background_runtime);

        if W::is_blocking() {
            // For blocking workers, spawn on blocking thread pool and block_on the async
            // task
            let handle = runtime.handle().clone();
            self.joins
                .spawn_blocking_on(move || handle.block_on(task), runtime.handle());
        } else {
            self.joins.spawn_on(task, runtime.handle());
        }

        WorkerHandle::new(name, notify, paused)
    }

    async fn run_loop<W>(
        worker: &mut W,
        ctx: &WorkerContext,
        paused: &Arc<AtomicBool>,
        trigger: crate::worker::Trigger,
        name: &'static str,
    ) -> Result<()>
    where
        W: Worker,
    {
        match trigger {
            Trigger::Once => {
                worker.work(ctx).await?;
                WORKER_EXECUTIONS.with_label_values(&[name]).inc();
            }
            Trigger::Notify => loop {
                tokio::select! {
                    _ = ctx.notified() => {
                        // Check if paused
                        if paused.load(Ordering::Acquire) {
                            continue; // Skip execution if paused
                        }

                        let start = std::time::Instant::now();
                        match worker.work(ctx).await {
                            Ok(_) => {
                                WORKER_EXECUTIONS.with_label_values(&[name]).inc();
                                WORKER_EXECUTION_DURATION_SECONDS
                                    .with_label_values(&[name])
                                    .observe(start.elapsed().as_secs_f64());
                            }
                            Err(e) => {
                                error!(worker = name, error = ?e, "Worker execution failed");
                                WORKER_EXECUTION_ERRORS.with_label_values(&[name]).inc();
                                return Err(e);
                            }
                        }
                    }
                    _ = ctx.cancelled() => {
                        break;
                    }
                }
            },
            Trigger::Interval(duration) => {
                let mut interval = tokio::time::interval(duration);
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            // Check if paused
                            if paused.load(Ordering::Acquire) {
                                continue; // Skip execution if paused
                            }

                            let start = std::time::Instant::now();
                            match worker.work(ctx).await {
                                Ok(_) => {
                                    WORKER_EXECUTIONS.with_label_values(&[name]).inc();
                                    WORKER_EXECUTION_DURATION_SECONDS
                                        .with_label_values(&[name])
                                        .observe(start.elapsed().as_secs_f64());
                                }
                                Err(e) => {
                                    error!(worker = name, error = ?e, "Worker execution failed");
                                    WORKER_EXECUTION_ERRORS.with_label_values(&[name]).inc();
                                    return Err(e);
                                }
                            }
                        }
                        _ = ctx.cancelled() => {
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Gracefully shutdown all workers.
    ///
    /// Cancels all workers and waits for them to finish within the configured
    /// timeout. Workers not responding in time will be aborted.
    pub async fn shutdown(mut self) -> Result<()> {
        info!("Shutting down worker manager");
        self.cancel_token.cancel();

        let deadline = tokio::time::Instant::now() + self.shutdown_timeout;
        let mut aborted_count = 0;
        let mut total_count = 0;

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());

            tokio::select! {
                result = self.joins.join_next() => {
                    match result {
                        Some(Ok(Ok(()))) => {
                            total_count += 1;
                        }
                        Some(Ok(Err(e))) => {
                            total_count += 1;
                            error!(error = ?e, "Worker error during shutdown");
                        }
                        Some(Err(e)) => {
                            total_count += 1;
                            if e.is_cancelled() {
                                aborted_count += 1;
                            } else {
                                error!(error = ?e, "Join error during shutdown");
                            }
                        }
                        None => {
                            // All workers finished
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(remaining) => {
                    // Timeout reached, abort remaining workers
                    error!(
                        timeout = ?self.shutdown_timeout,
                        "Shutdown timeout reached, aborting remaining workers"
                    );
                    self.joins.abort_all();

                    // Drain remaining join handles
                    while let Some(result) = self.joins.join_next().await {
                        total_count += 1;
                        if let Err(e) = result && e.is_cancelled() {
                            aborted_count += 1;
                        }
                    }
                    break;
                }
            }
        }

        if aborted_count > 0 {
            error!(
                stopped = total_count - aborted_count,
                aborted = aborted_count,
                "Worker manager shutdown complete"
            );
        } else {
            info!(stopped = total_count, "Worker manager shutdown complete");
        }

        Ok(())
    }
}
