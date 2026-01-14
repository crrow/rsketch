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

use std::time::Duration;

/// Defines when and how a worker should be executed.
///
/// Triggers control the scheduling strategy for worker execution. Each trigger type
/// returns a different handle type with appropriate control capabilities.
///
/// # Trigger Types
///
/// - **`Once`**: Runs immediately once at startup, then stops
/// - **`Notify`**: Runs only when explicitly triggered via handle
/// - **`Interval`**: Runs periodically at fixed intervals
/// - **`Cron`**: Runs on a cron schedule (e.g., "0 0 * * *" for daily at midnight)
/// - **`IntervalOrNotify`**: Hybrid - runs on interval OR when notified (timer resets)
/// - **`CronOrNotify`**: Hybrid - runs on cron schedule OR when notified
///
/// # Handle Mapping
///
/// Each trigger type yields a specific handle:
///
/// | Trigger | Handle | Capabilities |
/// |---------|--------|--------------|
/// | `Once` | `OnceHandle` | None (runs once) |
/// | `Notify` | `NotifyHandle` | Notifiable |
/// | `Interval` | `IntervalHandle` | Pausable |
/// | `Cron` | `CronHandle` | Pausable |
/// | `IntervalOrNotify` | `IntervalOrNotifyHandle` | Pausable + Notifiable |
/// | `CronOrNotify` | `CronOrNotifyHandle` | Pausable + Notifiable |
///
/// # Examples
///
/// ```rust
/// use rsketch_common_worker::Trigger;
/// use std::time::Duration;
///
/// // Run every 5 seconds
/// let trigger = Trigger::Interval(Duration::from_secs(5));
///
/// // Run every day at midnight (standard cron format)
/// let cron = croner::Cron::new("0 0 * * *").parse().unwrap();
/// let trigger = Trigger::Cron(cron);
///
/// // Run every hour OR when manually notified
/// let trigger = Trigger::IntervalOrNotify(Duration::from_secs(3600));
/// ```
#[derive(Debug, Clone)]
pub enum Trigger {
    /// Execute once immediately on startup, then stop.
    ///
    /// Returns [`OnceHandle`](crate::OnceHandle) with no control methods.
    /// Useful for initialization tasks.
    Once,

    /// Execute only when explicitly notified via handle.
    ///
    /// Returns [`NotifyHandle`](crate::NotifyHandle) with `notify()` method.
    /// Useful for event-driven or on-demand tasks.
    Notify,

    /// Execute at fixed intervals.
    ///
    /// Returns [`IntervalHandle`](crate::IntervalHandle) with pause/resume methods.
    /// The interval starts immediately and repeats continuously.
    Interval(Duration),

    /// Execute on a cron schedule.
    ///
    /// Returns [`CronHandle`](crate::CronHandle) with pause/resume methods.
    /// Uses standard 5-field cron format: `minute hour day month weekday`.
    /// Example: `"0 0 * * *"` = daily at midnight.
    Cron(croner::Cron),

    /// Execute at intervals OR when manually notified.
    ///
    /// Returns [`IntervalOrNotifyHandle`](crate::IntervalOrNotifyHandle) with pause/resume and notify methods.
    /// When notified, the worker runs immediately and the interval timer resets.
    IntervalOrNotify(Duration),

    /// Execute on cron schedule OR when manually notified.
    ///
    /// Returns [`CronOrNotifyHandle`](crate::CronOrNotifyHandle) with pause/resume and notify methods.
    /// Manual notifications trigger immediate execution without affecting the cron schedule.
    CronOrNotify(croner::Cron),
}
