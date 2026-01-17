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

//! Type-state builder pattern for worker configuration.
//!
//! This module uses the type-state pattern to enforce at compile time that:
//! - A trigger must be set before spawning
//! - The correct handle type is returned for each trigger
//!
//! The builder progresses through type states:
//! `TriggerNotSet` → `TriggerOnce/Notify/Interval/...` → `spawn()` → Handle

use std::{marker::PhantomData, str::FromStr, time::Duration};

use snafu::ResultExt;

use crate::{
    err::CronParseError,
    handle::{
        CronHandle, CronOrNotifyHandle, IntervalHandle, IntervalOrNotifyHandle, NotifyHandle,
        OnceHandle,
    },
    id::WorkerId,
    manager::Manager,
    trigger::Trigger,
    worker::Worker,
};

// Type-state markers for compile-time enforcement of trigger configuration.
// Each marker represents a different builder state.

/// Initial builder state - no trigger set yet.
pub struct TriggerNotSet;
/// Builder configured with `Trigger::Once`.
pub struct TriggerOnce;
/// Builder configured with `Trigger::Notify`.
pub struct TriggerNotify;
/// Builder configured with `Trigger::Interval`.
pub struct TriggerInterval;
/// Builder configured with `Trigger::Cron`.
pub struct TriggerCron;
/// Builder configured with `Trigger::IntervalOrNotify`.
pub struct TriggerIntervalOrNotify;
/// Builder configured with `Trigger::CronOrNotify`.
pub struct TriggerCronOrNotify;

/// Type-safe builder for configuring and spawning workers.
///
/// Uses the type-state pattern to ensure triggers are set before spawning.
/// The generic parameter `T` tracks the current configuration state.
///
/// # Type Parameters
///
/// - `'m`: Lifetime of the mutable reference to Manager
/// - `S`: State type from the Manager
/// - `W`: Worker implementation type
/// - `T`: Type-state marker (TriggerNotSet, TriggerOnce, etc.)
///
/// # Example
///
/// ```rust,no_run
/// # use rsketch_common_worker::{Manager, Worker, WorkerContext};
/// # use std::time::Duration;
/// # struct MyWorker;
/// # #[async_trait::async_trait]
/// # impl Worker for MyWorker {
/// #     async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {}
/// # }
/// # let mut manager = Manager::new();
/// // Type-state progression:
/// let builder = manager.worker(MyWorker); // TriggerNotSet
/// let builder = builder.name("my-worker"); // Still TriggerNotSet  
/// let builder = builder.interval(Duration::from_secs(5)); // Now TriggerInterval
/// let handle = builder.spawn(); // Returns IntervalHandle
/// ```
pub struct WorkerBuilder<'m, S, W, T> {
    manager:  &'m mut Manager<S>,
    worker:   W,
    name:     Option<&'static str>,
    blocking: bool,
    trigger:  Option<Trigger>,
    _phantom: PhantomData<T>,
}

impl<'m, S, W> WorkerBuilder<'m, S, W, TriggerNotSet>
where
    W: Worker,
    S: Send + Sync + 'static,
{
    pub(crate) fn new(manager: &'m mut Manager<S>, worker: W) -> Self {
        WorkerBuilder {
            manager,
            worker,
            name: None,
            blocking: false,
            trigger: None,
            _phantom: PhantomData,
        }
    }

    /// Configures the worker to run once immediately on startup, then stop.
    ///
    /// Returns a builder in `TriggerOnce` state, which can spawn an
    /// [`OnceHandle`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rsketch_common_worker::{Manager, Worker, WorkerContext};
    /// # struct InitWorker;
    /// # #[async_trait::async_trait]
    /// # impl Worker for InitWorker {
    /// #     async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {}
    /// # }
    /// # let mut manager = Manager::new();
    /// let handle = manager.worker(InitWorker).name("init").once().spawn();
    /// ```
    pub fn once(mut self) -> WorkerBuilder<'m, S, W, TriggerOnce> {
        self.trigger = Some(Trigger::Once);
        WorkerBuilder {
            manager:  self.manager,
            worker:   self.worker,
            name:     self.name,
            blocking: self.blocking,
            trigger:  self.trigger,
            _phantom: PhantomData,
        }
    }

    /// Configures the worker to run only when explicitly notified.
    ///
    /// Returns a builder in `TriggerNotify` state, which can spawn a
    /// [`NotifyHandle`]. Use `handle.notify()` to trigger execution.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rsketch_common_worker::{Manager, Worker, WorkerContext, Notifiable};
    /// # struct EventWorker;
    /// # #[async_trait::async_trait]
    /// # impl Worker for EventWorker {
    /// #     async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {}
    /// # }
    /// # let mut manager = Manager::new();
    /// let handle = manager.worker(EventWorker).on_notify().spawn();
    /// handle.notify(); // Trigger execution
    /// ```
    pub fn on_notify(mut self) -> WorkerBuilder<'m, S, W, TriggerNotify> {
        self.trigger = Some(Trigger::Notify);
        WorkerBuilder {
            manager:  self.manager,
            worker:   self.worker,
            name:     self.name,
            blocking: self.blocking,
            trigger:  self.trigger,
            _phantom: PhantomData,
        }
    }

    /// Configures the worker to run at fixed intervals.
    ///
    /// Returns a builder in `TriggerInterval` state, which can spawn an
    /// [`IntervalHandle`]. The worker runs repeatedly with the specified
    /// delay between executions.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rsketch_common_worker::{Manager, Worker, WorkerContext, Pausable};
    /// # use std::time::Duration;
    /// # struct PeriodicWorker;
    /// # #[async_trait::async_trait]
    /// # impl Worker for PeriodicWorker {
    /// #     async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {}
    /// # }
    /// # let mut manager = Manager::new();
    /// let handle = manager.worker(PeriodicWorker)
    ///     .interval(Duration::from_secs(60))  // Every minute
    ///     .spawn();
    /// handle.pause(); // Stop the timer
    /// handle.resume(); // Restart the timer
    /// ```
    pub fn interval(mut self, duration: Duration) -> WorkerBuilder<'m, S, W, TriggerInterval> {
        self.trigger = Some(Trigger::Interval(duration));
        WorkerBuilder {
            manager:  self.manager,
            worker:   self.worker,
            name:     self.name,
            blocking: self.blocking,
            trigger:  self.trigger,
            _phantom: PhantomData,
        }
    }

    /// Configures the worker to run on a cron schedule.
    ///
    /// Returns a builder in `TriggerCron` state, which can spawn a
    /// [`CronHandle`]. Uses standard 5-field cron format: `minute hour day
    /// month weekday`.
    ///
    /// # Cron Format
    ///
    /// - `*` = any value
    /// - `*/N` = every N units
    /// - `1,2,3` = list of values
    /// - `1-5` = range of values
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use rsketch_common_worker::{Manager, Worker, WorkerContext};
    /// # struct CronWorker;
    /// # #[async_trait::async_trait]
    /// # impl Worker for CronWorker {
    /// #     async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {}
    /// # }
    /// # let mut manager = Manager::new();
    /// // Every day at midnight
    /// manager
    ///     .worker(CronWorker)
    ///     .cron("0 0 * * *")
    ///     .unwrap()
    ///     .spawn();
    ///
    /// // Every 15 minutes
    /// manager
    ///     .worker(CronWorker)
    ///     .cron("*/15 * * * *")
    ///     .unwrap()
    ///     .spawn();
    ///
    /// // Every weekday at 9 AM
    /// manager
    ///     .worker(CronWorker)
    ///     .cron("0 9 * * 1-5")
    ///     .unwrap()
    ///     .spawn();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`CronParseError`] if the expression is invalid.
    pub fn cron(
        mut self,
        expr: &str,
    ) -> Result<WorkerBuilder<'m, S, W, TriggerCron>, CronParseError> {
        let cron = croner::Cron::from_str(expr).context(crate::err::InvalidExpressionSnafu)?;
        self.trigger = Some(Trigger::Cron(cron));
        Ok(WorkerBuilder {
            manager:  self.manager,
            worker:   self.worker,
            name:     self.name,
            blocking: self.blocking,
            trigger:  self.trigger,
            _phantom: PhantomData,
        })
    }

    /// Configures the worker to run on an interval OR when manually notified.
    ///
    /// Returns a builder in `TriggerIntervalOrNotify` state, which can spawn an
    /// [`IntervalOrNotifyHandle`]. This hybrid trigger combines periodic
    /// execution with on-demand triggering.
    ///
    /// When `notify()` is called, the worker runs immediately and the interval
    /// timer resets.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rsketch_common_worker::{Manager, Worker, WorkerContext, Notifiable, Pausable};
    /// # use std::time::Duration;
    /// # struct HybridWorker;
    /// # #[async_trait::async_trait]
    /// # impl Worker for HybridWorker {
    /// #     async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {}
    /// # }
    /// # let mut manager = Manager::new();
    /// let handle = manager.worker(HybridWorker)
    ///     .interval_or_notify(Duration::from_secs(300))  // Every 5 minutes
    ///     .spawn();
    ///
    /// handle.notify(); // Run immediately, reset timer
    /// handle.pause(); // Stop the interval
    /// handle.notify(); // Still works when paused
    /// handle.resume(); // Restart the interval
    /// ```
    pub fn interval_or_notify(
        mut self,
        duration: Duration,
    ) -> WorkerBuilder<'m, S, W, TriggerIntervalOrNotify> {
        self.trigger = Some(Trigger::IntervalOrNotify(duration));
        WorkerBuilder {
            manager:  self.manager,
            worker:   self.worker,
            name:     self.name,
            blocking: self.blocking,
            trigger:  self.trigger,
            _phantom: PhantomData,
        }
    }

    /// Configures the worker to run on a cron schedule OR when manually
    /// notified.
    ///
    /// Returns a builder in `TriggerCronOrNotify` state, which can spawn a
    /// [`CronOrNotifyHandle`]. This hybrid trigger combines cron scheduling
    /// with on-demand triggering.
    ///
    /// Unlike `interval_or_notify`, calling `notify()` does NOT reset the cron
    /// schedule. It only triggers an immediate one-time execution.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rsketch_common_worker::{Manager, Worker, WorkerContext, Notifiable};
    /// # struct ReportWorker;
    /// # #[async_trait::async_trait]
    /// # impl Worker for ReportWorker {
    /// #     async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {}
    /// # }
    /// # let mut manager = Manager::new();
    /// // Daily at 2 AM, but can also trigger on demand
    /// let handle = manager
    ///     .worker(ReportWorker)
    ///     .cron_or_notify("0 2 * * *")
    ///     .unwrap()
    ///     .spawn();
    ///
    /// // Generate report now (doesn't affect 2 AM schedule)
    /// handle.notify();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`CronParseError`] if the expression is invalid.
    pub fn cron_or_notify(
        mut self,
        expr: &str,
    ) -> Result<WorkerBuilder<'m, S, W, TriggerCronOrNotify>, CronParseError> {
        let cron = croner::Cron::from_str(expr).context(crate::err::InvalidExpressionSnafu)?;
        self.trigger = Some(Trigger::CronOrNotify(cron));
        Ok(WorkerBuilder {
            manager:  self.manager,
            worker:   self.worker,
            name:     self.name,
            blocking: self.blocking,
            trigger:  self.trigger,
            _phantom: PhantomData,
        })
    }
}

// Common configuration methods available in all builder states
impl<'m, S, W, T> WorkerBuilder<'m, S, W, T>
where
    W: Worker,
    S: Send + Sync + 'static,
{
    /// Sets the worker's name for logging and metrics.
    ///
    /// The name will be accessible via `ctx.name()` in the worker and used for
    /// structured logging and Prometheus metrics labels.
    ///
    /// If not set, defaults to `"unnamed-worker"`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rsketch_common_worker::{Manager, Worker, WorkerContext};
    /// # use std::time::Duration;
    /// # struct MyWorker;
    /// # #[async_trait::async_trait]
    /// # impl Worker for MyWorker {
    /// #     async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {}
    /// # }
    /// # let mut manager = Manager::new();
    /// manager.worker(MyWorker)
    ///     .name("database-cleaner")  // Shows in logs as "database-cleaner"
    ///     .interval(Duration::from_secs(3600))
    ///     .spawn();
    /// ```
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    /// Marks this worker as blocking (runs on dedicated blocking thread pool).
    ///
    /// Use this for CPU-intensive or synchronous blocking operations that would
    /// otherwise block the async runtime. Examples:
    /// - File I/O without async support
    /// - Heavy computation
    /// - Synchronous database calls
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rsketch_common_worker::{Manager, Worker, WorkerContext};
    /// # use std::time::Duration;
    /// # struct HeavyWorker;
    /// # #[async_trait::async_trait]
    /// # impl Worker for HeavyWorker {
    /// #     async fn work<S: Clone + Send + Sync>(&mut self, ctx: WorkerContext<S>) {}
    /// # }
    /// # let mut manager = Manager::new();
    /// manager.worker(HeavyWorker)
    ///     .name("cpu-intensive")
    ///     .blocking()  // Runs on blocking thread pool
    ///     .interval(Duration::from_secs(60))
    ///     .spawn();
    /// ```
    pub fn blocking(mut self) -> Self {
        self.blocking = true;
        self
    }
}

// spawn() implementations for each trigger type
impl<'m, S, W> WorkerBuilder<'m, S, W, TriggerOnce>
where
    W: Worker,
    S: Clone + Send + Sync + 'static,
{
    /// Spawns the worker and returns an [`OnceHandle`].
    ///
    /// The worker will execute immediately once and then stop.
    pub fn spawn(self) -> OnceHandle {
        let name = self.name.unwrap_or("unnamed-worker");
        self.manager
            .spawn_worker(self.worker, name, self.blocking, self.trigger.unwrap())
    }
}

impl<'m, S, W> WorkerBuilder<'m, S, W, TriggerNotify>
where
    W: Worker,
    S: Clone + Send + Sync + 'static,
{
    /// Spawns the worker and returns a [`NotifyHandle`].
    ///
    /// The worker will only execute when `handle.notify()` is called.
    pub fn spawn(self) -> NotifyHandle {
        let name = self.name.unwrap_or("unnamed-worker");
        self.manager
            .spawn_worker(self.worker, name, self.blocking, self.trigger.unwrap())
    }
}

impl<'m, S, W> WorkerBuilder<'m, S, W, TriggerInterval>
where
    W: Worker,
    S: Clone + Send + Sync + 'static,
{
    /// Spawns the worker and returns an [`IntervalHandle`].
    ///
    /// The worker will execute repeatedly at the configured interval.
    /// Use `handle.pause()` and `handle.resume()` to control execution.
    pub fn spawn(self) -> IntervalHandle {
        let name = self.name.unwrap_or("unnamed-worker");
        self.manager
            .spawn_worker(self.worker, name, self.blocking, self.trigger.unwrap())
    }
}

impl<'m, S, W> WorkerBuilder<'m, S, W, TriggerCron>
where
    W: Worker,
    S: Clone + Send + Sync + 'static,
{
    /// Spawns the worker and returns a [`CronHandle`].
    ///
    /// The worker will execute according to the configured cron schedule.
    /// Use `handle.pause()` and `handle.resume()` to control execution.
    pub fn spawn(self) -> CronHandle {
        let name = self.name.unwrap_or("unnamed-worker");
        self.manager
            .spawn_worker(self.worker, name, self.blocking, self.trigger.unwrap())
    }
}

impl<'m, S, W> WorkerBuilder<'m, S, W, TriggerIntervalOrNotify>
where
    W: Worker,
    S: Clone + Send + Sync + 'static,
{
    /// Spawns the worker and returns an [`IntervalOrNotifyHandle`].
    ///
    /// The worker will execute on an interval OR when notified.
    /// Provides both pause/resume and notify() methods.
    pub fn spawn(self) -> IntervalOrNotifyHandle {
        let name = self.name.unwrap_or("unnamed-worker");
        self.manager
            .spawn_worker(self.worker, name, self.blocking, self.trigger.unwrap())
    }
}

impl<'m, S, W> WorkerBuilder<'m, S, W, TriggerCronOrNotify>
where
    W: Worker,
    S: Clone + Send + Sync + 'static,
{
    /// Spawns the worker and returns a [`CronOrNotifyHandle`].
    ///
    /// The worker will execute on a cron schedule OR when notified.
    /// Provides both pause/resume and notify() methods.
    pub fn spawn(self) -> CronOrNotifyHandle {
        let name = self.name.unwrap_or("unnamed-worker");
        self.manager
            .spawn_worker(self.worker, name, self.blocking, self.trigger.unwrap())
    }
}

/// Internal trait for constructing handle types from their components.
///
/// This trait abstracts over the different handle construction patterns,
/// allowing `Manager::spawn_worker` to be generic over the return type.
/// Each handle type implements this to provide its specific construction logic.
///
/// # Implementation Note
///
/// This is an internal trait and should not be implemented by external code.
pub(crate) trait SpawnResult {
    fn from_parts(
        id: WorkerId,
        name: &'static str,
        notify: std::sync::Arc<tokio::sync::Notify>,
        paused: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self;
}

impl SpawnResult for OnceHandle {
    fn from_parts(
        id: WorkerId,
        name: &'static str,
        _notify: std::sync::Arc<tokio::sync::Notify>,
        _paused: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        OnceHandle::new(id, name)
    }
}

impl SpawnResult for NotifyHandle {
    fn from_parts(
        id: WorkerId,
        name: &'static str,
        notify: std::sync::Arc<tokio::sync::Notify>,
        _paused: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        NotifyHandle::new(id, name, notify)
    }
}

impl SpawnResult for IntervalHandle {
    fn from_parts(
        id: WorkerId,
        name: &'static str,
        notify: std::sync::Arc<tokio::sync::Notify>,
        paused: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        IntervalHandle::new(id, name, notify, paused)
    }
}

impl SpawnResult for CronHandle {
    fn from_parts(
        id: WorkerId,
        name: &'static str,
        notify: std::sync::Arc<tokio::sync::Notify>,
        paused: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        CronHandle::new(id, name, notify, paused)
    }
}

impl SpawnResult for IntervalOrNotifyHandle {
    fn from_parts(
        id: WorkerId,
        name: &'static str,
        notify: std::sync::Arc<tokio::sync::Notify>,
        paused: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        IntervalOrNotifyHandle::new(id, name, notify, paused)
    }
}

impl SpawnResult for CronOrNotifyHandle {
    fn from_parts(
        id: WorkerId,
        name: &'static str,
        notify: std::sync::Arc<tokio::sync::Notify>,
        paused: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        CronOrNotifyHandle::new(id, name, notify, paused)
    }
}
