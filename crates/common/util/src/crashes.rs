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

//! Crash handling and minidump generation.
//!
//! This module provides crash reporting functionality using minidumps.
//! Ported from Zed editor with modifications.

#[cfg(not(feature = "release-dev"))]
use std::panic::PanicHookInfo;
use std::{
    env, panic,
    path::{Path, PathBuf},
};

use bon::Builder;

/// Configuration for crash handler initialization.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Builder)]
#[builder(on(String, into))]
#[builder(on(PathBuf, into))]
pub struct CrashConfig {
    pub app_name: String,
    pub temp_dir: PathBuf,
    pub logs_dir: PathBuf,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Builder)]
#[builder(on(String, into))]
pub struct InitCrashHandler {
    pub session_id:      String,
    pub app_version:     String,
    pub binary:          String,
    #[builder(skip)]
    pub release_channel: String,
    pub commit_sha:      String,
}

#[cfg(not(feature = "release-dev"))]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct CrashPanic {
    pub message: String,
    pub span:    String,
}

// Dev build: simple panic handler with backtraces
#[cfg(feature = "release-dev")]
#[allow(unsafe_code)]
pub async fn init(_config: CrashConfig, _crash_init: InitCrashHandler) {
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        unsafe { env::set_var("RUST_BACKTRACE", "1") };
        old_hook(info);
        // Prevent the macOS crash dialog from popping up.
        if cfg!(target_os = "macos") {
            std::process::exit(1);
        }
    }));
}

#[cfg(feature = "release-dev")]
pub fn crash_server(_socket: &Path, _logs_dir: PathBuf) {
    // No-op in dev builds
}

// Production builds: full crash handler with minidumps
#[cfg(not(feature = "release-dev"))]
#[allow(unsafe_code)]
pub async fn init(config: CrashConfig, crash_init: InitCrashHandler) {
    #[cfg(target_os = "macos")]
    use std::sync::atomic::AtomicU32;
    use std::{
        process,
        sync::{
            Arc, OnceLock,
            atomic::{AtomicBool, Ordering},
        },
        time::Duration,
    };

    use crash_handler::{CrashEventResult, CrashHandler};
    use minidumper::{Client, SocketName};
    use tokio::process::Command;
    use tracing::info;

    // Set once the crash handler has initialized and the client has connected to
    // it.
    static CRASH_HANDLER: OnceLock<Arc<Client>> = OnceLock::new();
    // Set when the first minidump request is made to avoid generating duplicate
    // crash reports.
    static REQUESTED_MINIDUMP: AtomicBool = AtomicBool::new(false);

    #[cfg(target_os = "macos")]
    static PANIC_THREAD_ID: AtomicU32 = AtomicU32::new(0);

    static LOGS_DIR: OnceLock<PathBuf> = OnceLock::new();

    LOGS_DIR.set(config.logs_dir.clone()).ok();

    panic::set_hook(Box::new(panic_hook));

    let exe = env::current_exe().expect("unable to find ourselves");
    let pid = process::id();
    let socket_name = config
        .temp_dir
        .join(format!("{}-crash-handler-{}", config.app_name, pid));
    let _crash_handler = Command::new(exe)
        .arg("--crash-handler")
        .arg(&socket_name)
        .spawn()
        .expect("unable to spawn server process");

    #[cfg(target_os = "linux")]
    let server_pid = _crash_handler.id();
    info!("spawning crash handler process");

    let mut elapsed = Duration::ZERO;
    let retry_frequency = Duration::from_millis(100);
    let mut maybe_client = None;
    while maybe_client.is_none() {
        if let Ok(client) = Client::with_name(SocketName::Path(&socket_name)) {
            maybe_client = Some(client);
            info!("connected to crash handler process after {elapsed:?}");
            break;
        }
        elapsed += retry_frequency;
        tokio::time::sleep(retry_frequency).await;
    }
    let client = maybe_client.unwrap();
    client
        .send_message(1, serde_json::to_vec(&crash_init).unwrap())
        .unwrap();

    let client = Arc::new(client);
    let handler = CrashHandler::attach(unsafe {
        let client = client.clone();
        crash_handler::make_crash_event(move |crash_context: &crash_handler::CrashContext| {
            // Only request a minidump once.
            let res = if REQUESTED_MINIDUMP
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                #[cfg(target_os = "macos")]
                suspend_all_other_threads();

                // On macos this "ping" is needed to ensure that all our
                // `client.send_message` calls have been processed before we trigger the
                // minidump request.
                client.ping().ok();
                client.request_dump(crash_context).is_ok()
            } else {
                true
            };
            CrashEventResult::Handled(res)
        })
    })
    .expect("failed to attach signal handler");

    #[cfg(target_os = "linux")]
    {
        handler.set_ptracer(Some(server_pid));
    }
    CRASH_HANDLER.set(client.clone()).ok();
    std::mem::forget(handler);
    info!("crash handler registered");

    loop {
        client.ping().ok();
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    #[cfg(target_os = "macos")]
    unsafe fn suspend_all_other_threads() {
        let task = unsafe { mach2::traps::current_task() };
        let mut threads: mach2::mach_types::thread_act_array_t = std::ptr::null_mut();
        let mut count = 0;
        unsafe {
            mach2::task::task_threads(task, &raw mut threads, &raw mut count);
        }
        let current = unsafe { mach2::mach_init::mach_thread_self() };
        let panic_thread = PANIC_THREAD_ID.load(Ordering::SeqCst);
        for i in 0..count {
            let t = unsafe { *threads.add(i as usize) };
            if t != current && t != panic_thread {
                unsafe { mach2::thread_act::thread_suspend(t) };
            }
        }
    }

    fn panic_hook(info: &PanicHookInfo) {
        use std::{thread, time::Duration};

        let message = info.payload_as_str().unwrap_or("Box<Any>").to_owned();

        let span = info
            .location()
            .map(|loc| format!("{}:{}", loc.file(), loc.line()))
            .unwrap_or_default();

        let current_thread = std::thread::current();
        let thread_name = current_thread.name().unwrap_or("<unnamed>");

        // Wait 500ms for the crash handler process to start up.
        // If it's still not there just write panic info and no minidump.
        let retry_frequency = Duration::from_millis(100);
        for _ in 0..5 {
            if let Some(client) = CRASH_HANDLER.get() {
                let location = info
                    .location()
                    .map_or_else(|| "<unknown>".to_owned(), |location| location.to_string());
                tracing::error!("thread '{thread_name}' panicked at {location}:\n{message}...");
                client
                    .send_message(
                        2,
                        serde_json::to_vec(&CrashPanic {
                            message: message.clone(),
                            span:    span.clone(),
                        })
                        .unwrap(),
                    )
                    .ok();
                tracing::error!("triggering a crash to generate a minidump...");

                #[cfg(target_os = "macos")]
                PANIC_THREAD_ID.store(
                    unsafe { mach2::mach_init::mach_thread_self() },
                    Ordering::SeqCst,
                );

                std::process::abort();
            }
            thread::sleep(retry_frequency);
        }
    }
}

#[cfg(not(feature = "release-dev"))]
pub fn crash_server(socket: &Path, logs_dir: PathBuf) {
    use std::{
        fs::{self, File},
        io,
        sync::{
            Arc, OnceLock,
            atomic::{AtomicBool, Ordering},
        },
        thread,
        time::Duration,
    };

    use minidumper::{LoopAction, MinidumpBinary, SocketName};

    const CRASH_HANDLER_PING_TIMEOUT: Duration = Duration::from_secs(60);
    const CRASH_HANDLER_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

    struct CrashServer {
        initialization_params: OnceLock<InitCrashHandler>,
        panic_info:            OnceLock<CrashPanic>,
        has_connection:        Arc<AtomicBool>,
        logs_dir:              PathBuf,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    struct CrashInfo {
        pub init:           InitCrashHandler,
        pub panic:          Option<CrashPanic>,
        pub minidump_error: Option<String>,
    }

    impl minidumper::ServerHandler for CrashServer {
        fn create_minidump_file(&self) -> Result<(File, PathBuf), io::Error> {
            let err_message = "Missing initialization data";
            let dump_path = self
                .logs_dir
                .join(
                    &self
                        .initialization_params
                        .get()
                        .expect(err_message)
                        .session_id,
                )
                .with_extension("dmp");
            let file = File::create(&dump_path)?;
            Ok((file, dump_path))
        }

        fn on_minidump_created(
            &self,
            result: Result<MinidumpBinary, minidumper::Error>,
        ) -> LoopAction {
            let minidump_error = match result {
                Ok(MinidumpBinary { mut file, path, .. }) => {
                    use io::Write;
                    file.flush().ok();
                    drop(file);
                    let original_file = File::open(&path).unwrap();
                    let compressed_path = path.with_extension("zstd");
                    let compressed_file = File::create(&compressed_path).unwrap();
                    zstd::stream::copy_encode(original_file, compressed_file, 0).ok();
                    fs::rename(&compressed_path, path).unwrap();
                    None
                }
                Err(e) => Some(format!("{e:?}")),
            };

            let crash_info = CrashInfo {
                init: self
                    .initialization_params
                    .get()
                    .expect("not initialized")
                    .clone(),
                panic: self.panic_info.get().cloned(),
                minidump_error,
            };

            let crash_data_path = self
                .logs_dir
                .join(&crash_info.init.session_id)
                .with_extension("json");

            fs::write(crash_data_path, serde_json::to_vec(&crash_info).unwrap()).ok();

            LoopAction::Exit
        }

        fn on_message(&self, kind: u32, buffer: Vec<u8>) {
            match kind {
                1 => {
                    let init_data = serde_json::from_slice::<InitCrashHandler>(&buffer)
                        .expect("invalid init data");
                    self.initialization_params
                        .set(init_data)
                        .expect("already initialized");
                }
                2 => {
                    let panic_data =
                        serde_json::from_slice::<CrashPanic>(&buffer).expect("invalid panic data");
                    self.panic_info.set(panic_data).expect("already panicked");
                }
                _ => {
                    panic!("invalid message kind");
                }
            }
        }

        fn on_client_disconnected(&self, _clients: usize) -> LoopAction { LoopAction::Exit }

        fn on_client_connected(&self, _clients: usize) -> LoopAction {
            self.has_connection.store(true, Ordering::SeqCst);
            LoopAction::Continue
        }
    }

    let Ok(mut server) = minidumper::Server::with_name(SocketName::Path(socket)) else {
        tracing::info!("Couldn't create socket, there may already be a running crash server");
        return;
    };

    let shutdown = Arc::new(AtomicBool::new(false));
    let has_connection = Arc::new(AtomicBool::new(false));

    thread::Builder::new()
        .name("CrashServerTimeout".to_owned())
        .spawn({
            let shutdown = shutdown.clone();
            let has_connection = has_connection.clone();
            move || {
                std::thread::sleep(CRASH_HANDLER_CONNECT_TIMEOUT);
                if !has_connection.load(Ordering::SeqCst) {
                    shutdown.store(true, Ordering::SeqCst);
                }
            }
        })
        .unwrap();

    server
        .run(
            Box::new(CrashServer {
                initialization_params: OnceLock::new(),
                panic_info: OnceLock::new(),
                has_connection,
                logs_dir,
            }),
            &shutdown,
            Some(CRASH_HANDLER_PING_TIMEOUT),
        )
        .expect("failed to run server");
}
