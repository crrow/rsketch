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

use crate::context::WorkerContext;

/// Internal enum for trigger execution strategy.
pub(crate) enum TriggerDriverEnum {
    Once(OnceDriver),
    Notify(NotifyDriver),
    Interval(IntervalDriver),
    Cron(CronDriver),
    IntervalOrNotify(IntervalOrNotifyDriver),
    CronOrNotify(CronOrNotifyDriver),
}

impl TriggerDriverEnum {
    pub async fn wait_next<S: Clone>(&mut self, ctx: &WorkerContext<S>) -> bool {
        match self {
            TriggerDriverEnum::Once(d) => d.wait_next(ctx).await,
            TriggerDriverEnum::Notify(d) => d.wait_next(ctx).await,
            TriggerDriverEnum::Interval(d) => d.wait_next(ctx).await,
            TriggerDriverEnum::Cron(d) => d.wait_next(ctx).await,
            TriggerDriverEnum::IntervalOrNotify(d) => d.wait_next(ctx).await,
            TriggerDriverEnum::CronOrNotify(d) => d.wait_next(ctx).await,
        }
    }
}

/// Internal trait for trigger execution strategy.
trait TriggerDriver: Send {
    /// Wait for next execution. Returns false if should stop.
    async fn wait_next<S: Clone>(&mut self, ctx: &WorkerContext<S>) -> bool;
}

/// Driver for Once trigger.
pub(crate) struct OnceDriver {
    executed: bool,
}

impl OnceDriver {
    pub fn new() -> Self { OnceDriver { executed: false } }
}

impl TriggerDriver for OnceDriver {
    async fn wait_next<S: Clone>(&mut self, ctx: &WorkerContext<S>) -> bool {
        if self.executed {
            // Wait for cancellation
            ctx.cancelled().await;
            false
        } else {
            self.executed = true;
            true
        }
    }
}

/// Driver for Notify trigger.
pub(crate) struct NotifyDriver;

impl NotifyDriver {
    pub fn new() -> Self { NotifyDriver }
}

impl TriggerDriver for NotifyDriver {
    async fn wait_next<S: Clone>(&mut self, ctx: &WorkerContext<S>) -> bool {
        tokio::select! {
            _ = ctx.notified() => true,
            _ = ctx.cancelled() => false,
        }
    }
}

/// Driver for Interval trigger.
pub(crate) struct IntervalDriver {
    interval: tokio::time::Interval,
}

impl IntervalDriver {
    pub fn new(duration: Duration) -> Self {
        let mut interval = tokio::time::interval(duration);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        IntervalDriver { interval }
    }
}

impl TriggerDriver for IntervalDriver {
    async fn wait_next<S: Clone>(&mut self, ctx: &WorkerContext<S>) -> bool {
        tokio::select! {
            _ = self.interval.tick() => true,
            _ = ctx.cancelled() => false,
        }
    }
}

/// Driver for Cron trigger.
pub(crate) struct CronDriver {
    cron: croner::Cron,
}

impl CronDriver {
    pub fn new(cron: croner::Cron) -> Self { CronDriver { cron } }
}

impl TriggerDriver for CronDriver {
    async fn wait_next<S: Clone>(&mut self, ctx: &WorkerContext<S>) -> bool {
        // Get next occurrence
        let next = match self.cron.find_next_occurrence(&chrono::Utc::now(), false) {
            Ok(next) => next,
            Err(_) => {
                // No more occurrences, wait for cancellation
                ctx.cancelled().await;
                return false;
            }
        };

        let now = chrono::Utc::now();
        if next > now {
            let duration = (next - now).to_std().unwrap_or(Duration::from_secs(0));

            tokio::select! {
                _ = tokio::time::sleep(duration) => true,
                _ = ctx.cancelled() => false,
            }
        } else {
            // Next occurrence is in the past or now, execute immediately
            true
        }
    }
}

/// Driver for IntervalOrNotify trigger.
pub(crate) struct IntervalOrNotifyDriver {
    interval: tokio::time::Interval,
}

impl IntervalOrNotifyDriver {
    pub fn new(duration: Duration) -> Self {
        let mut interval = tokio::time::interval(duration);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        IntervalOrNotifyDriver { interval }
    }
}

impl TriggerDriver for IntervalOrNotifyDriver {
    async fn wait_next<S: Clone>(&mut self, ctx: &WorkerContext<S>) -> bool {
        tokio::select! {
            _ = self.interval.tick() => true,
            _ = ctx.notified() => {
                // Reset the interval
                self.interval.reset();
                true
            },
            _ = ctx.cancelled() => false,
        }
    }
}

/// Driver for CronOrNotify trigger.
pub(crate) struct CronOrNotifyDriver {
    cron: croner::Cron,
}

impl CronOrNotifyDriver {
    pub fn new(cron: croner::Cron) -> Self { CronOrNotifyDriver { cron } }
}

impl TriggerDriver for CronOrNotifyDriver {
    async fn wait_next<S: Clone>(&mut self, ctx: &WorkerContext<S>) -> bool {
        // Get next occurrence
        let next = match self.cron.find_next_occurrence(&chrono::Utc::now(), false) {
            Ok(next) => next,
            Err(_) => {
                // No more occurrences, wait for notification or cancellation
                return tokio::select! {
                    _ = ctx.notified() => true,
                    _ = ctx.cancelled() => false,
                };
            }
        };

        let now = chrono::Utc::now();
        if next > now {
            let duration = (next - now).to_std().unwrap_or(Duration::from_secs(0));

            tokio::select! {
                _ = tokio::time::sleep(duration) => true,
                _ = ctx.notified() => true,
                _ = ctx.cancelled() => false,
            }
        } else {
            // Next occurrence is in the past or now, but also check notification
            tokio::select! {
                _ = ctx.notified() => true,
                _ = ctx.cancelled() => false,
                else => true,
            }
        }
    }
}
