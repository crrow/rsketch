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

use std::{
    collections::HashMap,
    io::{self, IsTerminal},
    path::Path,
    sync::OnceLock,
};

use gpui::{App, AppContext, Application, QuitMode};
use jiff::Timestamp;
use yunara_paths;

use yunara_player::util::ResultExt;

static STARTUP_TIME: OnceLock<Timestamp> = OnceLock::new();

const FORCE_CLI_MODE_ENV_VAR_NAME: &str = "YUNARA_FORCE_CLI_MODE";

#[inline]
pub(crate) fn startup_time() -> Timestamp { *STARTUP_TIME.get_or_init(Timestamp::now) }

pub(crate) fn init_paths() -> HashMap<io::ErrorKind, Vec<&'static Path>> {
    [
        yunara_paths::config_dir(),
        yunara_paths::database_dir(),
        yunara_paths::logs_dir(),
        yunara_paths::temp_dir(),
        yunara_paths::hang_traces_dir(),
    ]
    .into_iter()
    .fold(HashMap::default(), |mut errors, path| {
        if let Err(e) = std::fs::create_dir_all(path) {
            errors.entry(e.kind()).or_insert_with(Vec::new).push(path);
        }
        errors
    })
}

pub(crate) fn files_not_created_on_launch(errors: HashMap<io::ErrorKind, Vec<&Path>>) {
    let message = "Yunara failed to launch";
    let error_details = errors
        .into_iter()
        .flat_map(|(kind, paths)| {
            #[allow(unused_mut)] // for non-unix platforms
            let mut error_kind_details = match paths.len() {
                0 => return None,
                1 => format!(
                    "{kind} when creating directory {:?}",
                    paths.first().expect("match arm checks for a single entry")
                ),
                _many => format!("{kind} when creating directories {paths:?}"),
            };

            #[cfg(unix)]
            {
                if kind == io::ErrorKind::PermissionDenied {
                    error_kind_details.push_str(
                        "\n\nConsider using chown and chmod tools for altering the directories \
                         permissions if your user has corresponding rights.\nFor example, `sudo \
                         chown $(whoami):staff ~/.config` and `chmod +uwrx ~/.config`",
                    );
                }
            }

            Some(error_kind_details)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    eprintln!("{message}: {error_details}");
    Application::new()
        .with_quit_mode(QuitMode::Explicit)
        .run(move |cx| {
            if let Ok(window) = cx.open_window(gpui::WindowOptions::default(), |_, cx| {
                cx.new(|_| gpui::Empty)
            }) {
                window
                    .update(cx, |_, window, cx| {
                        let response = window.prompt(
                            gpui::PromptLevel::Critical,
                            message,
                            Some(&error_details),
                            &["Exit"],
                            cx,
                        );

                        cx.spawn_in(window, async move |_, cx| {
                            response.await?;
                            cx.update(|_, cx| cx.quit())
                        })
                        .detach_and_log_err(cx);
                    })
                    .log_err();
            } else {
                fail_to_open_window(anyhow::anyhow!("{message}: {error_details}"), cx)
            }
        })
}

pub(crate) fn fail_to_open_window(e: anyhow::Error, _cx: &mut App) {
    eprintln!(
        "Yunara failed to open a window: {e:?}. See https://github.com/crrow/rsketch for \
         troubleshooting steps."
    );
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    {
        std::process::exit(1);
    }

    // Maybe unify this with
    // gpui::platform::linux::platform::ResultExt::notify_err(..)?
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        use ashpd::desktop::notification::{Notification, NotificationProxy, Priority};
        _cx.spawn(async move |_cx| {
            let Ok(proxy) = NotificationProxy::new().await else {
                std::process::exit(1);
            };

            let notification_id = "dev.Yunara.Oops";
            proxy
                .add_notification(
                    notification_id,
                    Notification::new("Yunara failed to launch")
                        .body(Some(
                            format!(
                                "{e:?}. See https://Yunara.dev/docs/linux for troubleshooting \
                                 steps."
                            )
                            .as_str(),
                        ))
                        .priority(Priority::High)
                        .icon(ashpd::desktop::Icon::with_names(&[
                            "dialog-question-symbolic",
                        ])),
                )
                .await
                .ok();

            process::exit(1);
        })
        .detach();
    }
}

pub(crate) fn stdout_is_a_pty() -> bool {
    std::env::var(FORCE_CLI_MODE_ENV_VAR_NAME).ok().is_none() && io::stdout().is_terminal()
}
