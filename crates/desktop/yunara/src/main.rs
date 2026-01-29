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

use gpui::{AppContext, Application, WindowBounds, WindowOptions, px};
use rsketch_common_util::{
    crashes::{self, CrashConfig, InitCrashHandler},
    ensure_single_instance::ensure_only_instance,
    version::YunaraVersion,
};
use shadow_rs::shadow;
use yunara_player::{
    AppConfig, AppState, YunaraPlayer, config::ApplicationConfig, consts,
};
use yunara_store::DatabaseConfig;
use yunara_ui::components::theme::ThemeProvider;

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
        consts::YUNARA,
        &rsketch_common_telemetry::logging::LoggingOptions::builder()
            .dir(yunara_paths::logs_dir().to_string_lossy())
            .append_stdout(helper::stdout_is_a_pty())
            .build(),
        &rsketch_common_telemetry::logging::TracingOptions::default(),
        None,
    );
    tracing::info!(
        "Starting Yunara desktop application version {}, sha {}",
        build::VERSION,
        build::COMMIT_HASH
    );

    // Create Tokio runtime for async operations (multi-threaded for better
    // concurrency)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    let handle = rt.handle().clone();

    let app = Application::new().with_assets(yunara_assets::Assets);

    // Initialize app state using the tokio runtime
    let app_state = handle
        .block_on(async {
            AppState::new(
                AppConfig::builder()
                    .database(
                        DatabaseConfig::builder()
                            .db_path(yunara_paths::database_dir().join(consts::YUNARA_DB_FILE))
                            .build(),
                    )
                    .app(ApplicationConfig::default())
                    .build(),
            )
            .await
        })
        .expect("unable to initialize application state");

    let app_version = YunaraVersion::load(build::VERSION, build::COMMIT_HASH);
    app.background_executor()
        .spawn(crashes::init(
            CrashConfig::builder()
                .app_name(consts::YUNARA)
                .logs_dir(yunara_paths::logs_dir())
                .temp_dir(yunara_paths::temp_dir())
                .build(),
            InitCrashHandler::builder()
                .session_id(app_state.get_session_id())
                .app_version(app_version.clone())
                .binary(consts::YUNARA)
                .commit_sha(build::COMMIT_HASH)
                .build(),
        ))
        .detach();

    if matches!(
        ensure_only_instance(),
        rsketch_common_util::ensure_single_instance::IsOnlyInstance::No
    ) {
        tracing::info!("Another instance is already running, exiting.");
        return;
    }

    app.run(move |cx| {
        // Initialize gpui_tokio with our runtime handle
        gpui_tokio::init_from_handle(cx, handle);

        // Initialize the theme provider with default YTMusic dark theme
        ThemeProvider::init(cx);

        // Create app state entity
        let app_state_entity = cx.new(|_cx| app_state.clone());

        // Open the main window
        let bounds = WindowBounds::Windowed(gpui::Bounds {
            origin: gpui::Point::default(),
            size:   gpui::Size {
                width:  px(1280.0),
                height: px(800.0),
            },
        });

        let options = WindowOptions {
            window_bounds: Some(bounds),
            ..Default::default()
        };

        cx.open_window(options, move |_window, cx| {
            cx.new(|cx| YunaraPlayer::new(app_state_entity.clone(), cx))
        })
        .expect("Failed to open main window");
    });
}
