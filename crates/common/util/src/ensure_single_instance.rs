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
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use sysinfo::System;

const LOCALHOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(10);
const RECEIVE_TIMEOUT: Duration = Duration::from_millis(35);
const SEND_TIMEOUT: Duration = Duration::from_millis(20);
#[cfg(not(feature = "release-dev"))]
const USER_BLOCK: u16 = 100;
const BASE_PORT: u16 = 43737;

/// Get the base port for the current release channel
#[cfg(feature = "release-dev")]
const fn channel_port() -> u16 { BASE_PORT }

#[cfg(feature = "release-preview")]
const fn channel_port() -> u16 { BASE_PORT + USER_BLOCK }

#[cfg(feature = "release-stable")]
const fn channel_port() -> u16 { BASE_PORT + (2 * USER_BLOCK) }

/// Get the handshake message for the current release channel
#[cfg(feature = "release-dev")]
const fn instance_handshake() -> &'static str { "Yunara Dev Instance Running" }

#[cfg(feature = "release-preview")]
const fn instance_handshake() -> &'static str { "Yunara Preview Instance Running" }

#[cfg(feature = "release-stable")]
const fn instance_handshake() -> &'static str { "Yunara Stable Instance Running" }

fn address() -> SocketAddr {
    // These port numbers are offset by the user ID to avoid conflicts between
    // different users on the same machine. In addition to that the ports for each
    // release channel are spaced out by 100 to avoid conflicts between different
    // users running different release channels on the same machine. This ends up
    // interleaving the ports between different users and different release
    // channels.
    //
    // On macOS user IDs start at 501 and on Linux they start at 1000. The first
    // user on a Mac with ID 501 running a dev channel build will use port
    // 44238, and the second user with ID 502 will use port 44239, and so on.
    // User 501 will use ports 44338, 44438, and 44538 for the preview, stable,
    // and nightly channels, respectively. User 502 will use ports 44339, 44439,
    // and 44539 for the preview, stable, and nightly channels, respectively.
    let port = channel_port();
    let mut user_port = port;

    let mut sys = System::new_all();
    sys.refresh_all();
    if let Ok(current_pid) = sysinfo::get_current_pid()
        && let Some(uid) = sys
            .process(current_pid)
            .and_then(|process| process.user_id())
    {
        let uid_u32 = get_uid_as_u32(uid);
        // Ensure that the user ID is not too large to avoid overflow when
        // calculating the port number. This seems unlikely but it doesn't
        // hurt to be safe.
        let max_port = 65535;
        let max_uid: u32 = max_port - port as u32;
        let wrapped_uid: u16 = (uid_u32 % max_uid) as u16;
        user_port += wrapped_uid;
    }

    SocketAddr::V4(SocketAddrV4::new(LOCALHOST, user_port))
}

#[cfg(unix)]
fn get_uid_as_u32(uid: &sysinfo::Uid) -> u32 { **uid }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsOnlyInstance {
    Yes,
    No,
}

pub fn ensure_only_instance() -> IsOnlyInstance {
    if check_got_handshake() {
        return IsOnlyInstance::No;
    }

    let listener = match TcpListener::bind(address()) {
        Ok(listener) => listener,

        Err(err) => {
            tracing::warn!("Error binding to single instance port: {err}");
            if check_got_handshake() {
                return IsOnlyInstance::No;
            }

            // Avoid failing to start when some other application by chance already has
            // a claim on the port. This is sub-par as any other instance that gets launched
            // will be unable to communicate with this instance and will duplicate
            tracing::warn!("Backup handshake request failed, continuing without handshake");
            return IsOnlyInstance::Yes;
        }
    };

    thread::Builder::new()
        .name("EnsureSingleton".to_string())
        .spawn(move || {
            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(stream) => stream,
                    Err(_) => return,
                };

                _ = stream.set_nodelay(true);
                _ = stream.set_read_timeout(Some(SEND_TIMEOUT));
                _ = stream.write_all(instance_handshake().as_bytes());
            }
        })
        .unwrap();

    IsOnlyInstance::Yes
}

fn check_got_handshake() -> bool {
    match TcpStream::connect_timeout(&address(), CONNECT_TIMEOUT) {
        Ok(mut stream) => {
            let mut buf = vec![0u8; instance_handshake().len()];

            stream.set_read_timeout(Some(RECEIVE_TIMEOUT)).unwrap();
            if let Err(err) = stream.read_exact(&mut buf) {
                tracing::warn!("Connected to single instance port but failed to read: {err}");
                return false;
            }

            if buf == instance_handshake().as_bytes() {
                tracing::info!("Got instance handshake");
                return true;
            }

            tracing::warn!("Got wrong instance handshake value");
            false
        }

        Err(_) => false,
    }
}
