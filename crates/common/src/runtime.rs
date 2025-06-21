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

use std::future::Future;

// The following are helpers to build named tasks.
//
// Named tasks require the tokio feature `tracing` to be enabled. If the
// `named_tasks` feature is disabled, this is no-op.
//
// By default, these function will just ignore the name passed and just act like
// a regular call to `tokio::spawn`.
//
// If the user compiles `quickwit-cli` with the `tokio-console` feature, then
// tasks will automatically be named. This is not just "visual sugar".
//
// Without names, tasks will only show their spawn site on tokio-console. This
// is a catastrophy for actors who all share the same spawn site.
//
// The #[track_caller] annotation is used to show the right spawn site in the
// Tokio TRACE spans (only available when the tokio/tracing feature is on).
//
// # Naming
//
// Actors will get named after their type, which is fine. For other tasks,
// please use `snake_case`.

#[cfg(not(all(tokio_unstable, feature = "named_tasks")))]
#[track_caller]
pub fn spawn_named_task<F>(future: F, _name: &'static str) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::task::spawn(future)
}

#[cfg(not(all(tokio_unstable, feature = "named_tasks")))]
#[track_caller]
pub fn spawn_named_task_on<F>(
    future: F,
    _name: &'static str,
    runtime: &tokio::runtime::Handle,
) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    runtime.spawn(future)
}

#[cfg(all(tokio_unstable, feature = "named_tasks"))]
#[track_caller]
pub fn spawn_named_task<F>(future: F, name: &'static str) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::task::Builder::new()
        .name(name)
        .spawn(future)
        .unwrap()
}

#[cfg(all(tokio_unstable, feature = "named_tasks"))]
#[track_caller]
pub fn spawn_named_task_on<F>(
    future: F,
    name: &'static str,
    runtime: &tokio::runtime::Handle,
) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::task::Builder::new()
        .name(name)
        .spawn_on(future, runtime)
        .unwrap()
}
