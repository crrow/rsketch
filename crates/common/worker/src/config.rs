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

use std::{sync::Arc, time::Duration};

use rsketch_common_runtime::Runtime;

#[derive(Debug, Clone, bon::Builder)]
pub struct WorkerConfig {
    #[builder(into)]
    runtime: Option<Arc<Runtime>>,

    /// Timeout for graceful shutdown. Workers not responding within this
    /// duration will be forcefully aborted. Default: 30 seconds.
    #[builder(default = Duration::from_secs(30), into)]
    shutdown_timeout: Duration,
}

impl WorkerConfig {
    pub(crate) fn runtime(&self) -> Option<Arc<Runtime>> { self.runtime.clone() }

    pub(crate) fn shutdown_timeout(&self) -> Duration { self.shutdown_timeout }
}
