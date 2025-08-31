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

/// Generated build information from built.rs
mod built {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// Package author information from Cargo.toml
pub const AUTHOR: &str = built::PKG_AUTHORS;

/// Returns true if this is an official release build (KISEKI_RELEASE env var is
/// set)
const fn is_official_release() -> bool { option_env!("KISEKI_RELEASE").is_some() }

/// Extract commit hash as a const string, defaulting to empty string if
/// unavailable
const COMMIT_HASH: &str = match built::GIT_COMMIT_HASH_SHORT {
    Some(hash) => hash,
    None => "",
};

/// Extract dirty status as a const string suffix
const DIRTY_SUFFIX: &str = match built::GIT_DIRTY {
    Some(true) => "-dirty",
    _ => "",
};

/// Full version string with optional development suffix
///
/// For official releases: uses PKG_VERSION as-is
/// For development builds: appends "-unofficial" or
/// "-unofficial+{hash}{-dirty}"
#[allow(clippy::const_is_empty)]
pub const FULL_VERSION: &str = {
    if is_official_release() {
        built::PKG_VERSION
    } else if COMMIT_HASH.is_empty() {
        // No git info available
        const_format::concatcp!(built::PKG_VERSION, "-unofficial")
    } else {
        // Git info available - include hash and dirty status
        const_format::concatcp!(
            built::PKG_VERSION,
            "-unofficial+",
            COMMIT_HASH,
            DIRTY_SUFFIX
        )
    }
};
