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

#[allow(dead_code)]
mod app_state;
#[allow(dead_code)]
mod config;
mod consts;
mod helper;
mod services;
mod util;

use app_state::AppState;
use gpui::Application;
use rsketch_common_util::crashes::{self, CrashConfig, InitCrashHandler};
use shadow_rs::shadow;
use yunara_store::DatabaseConfig;

use crate::config::ApplicationConfig;

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

    let app = Application::new().with_assets(yunara_assets::Assets);
    let app_state = app
        .foreground_executor()
        .block_on(async {
            AppState::new(
                config::AppConfig::builder()
                    .database(
                        DatabaseConfig::builder()
                            .db_path(yunara_paths::database_dir())
                            .build(),
                    )
                    .app(ApplicationConfig::default())
                    .build(),
            )
            .await
        })
        .expect("unable to initialize application state");

    app.background_executor()
        .spawn(crashes::init(
            CrashConfig::builder()
                .app_name(consts::APP_NAME)
                .logs_dir(yunara_paths::logs_dir())
                .temp_dir(yunara_paths::temp_dir())
                .build(),
            InitCrashHandler::builder()
                .session_id(app_state.get_session_id())
                .app_version(rsketch_common_util::version::YunaraVersion::load(
                    build::VERSION,
                    build::COMMIT_HASH,
                ))
                .binary(consts::APP_NAME)
                .commit_sha(build::COMMIT_HASH)
                .build(),
        ))
        .detach();

    // TODO: set crash handler
}
