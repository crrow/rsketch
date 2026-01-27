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

mod helper;
mod util;

use gpui::Application;
use shadow_rs::shadow;
use uuid::Uuid;

shadow!(build);

fn main() {
    helper::startup_time();
    let file_errors = helper::init_paths();
    if !file_errors.is_empty() {
        helper::files_not_created_on_launch(file_errors);
        return;
    }
    // Initialize tracing subscriber
    let _guards = rsketch_common_telemetry::logging::init_global_logging(
        "Yunara",
        &rsketch_common_telemetry::logging::LoggingOptions::builder()
            .dir(yunara_paths::logs_dir().to_string_lossy())
            .append_stdout(helper::stdout_is_a_pty())
            .build(),
        &rsketch_common_telemetry::logging::TracingOptions::default(),
        None,
    );
    tracing::info!(
        "========== starting yunara version {}, sha {} ==========",
        build::VERSION,
        build::COMMIT_HASH,
    );

    let _app = Application::new().with_assets(yunara_assets::Assets);

    // let system_id = app.background_executor().spawn(system_id());
    // let installation_id = app.background_executor().spawn(installation_id());
    let _session_id = Uuid::new_v4().to_string();
    // let session = app
    //     .background_executor()
    //     .spawn(Session::new(session_id.clone()));
}
