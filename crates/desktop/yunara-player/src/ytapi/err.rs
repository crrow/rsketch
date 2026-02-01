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

use std::path::PathBuf;

use snafu::Snafu;

use crate::ytapi::AuthType;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum Error {
    IO {
        source: std::io::Error,
        #[snafu(implicit)]
        loc:    snafu::Location,
    },
    #[snafu(display("Failed to do file operation on {}", path.display()))]
    FileIO {
        source: std::io::Error,
        path:   PathBuf,
        #[snafu(implicit)]
        loc:    snafu::Location,
    },
    #[snafu(transparent)]
    YtmapiError {
        source: ytmapi_rs::Error,
        #[snafu(implicit)]
        loc:    snafu::Location,
    },
    #[snafu(display("Invalid Config path, config dir not exists"))]
    InvalidConfigPath {
        path: PathBuf,
        #[snafu(implicit)]
        loc:  snafu::Location,
    },
    #[snafu(display(
        "Not supported on auth type {:?}. Expected auth type: {:?}",
        current_authtype,
        expected_authtypes
    ))]
    InvalidAuthToken {
        current_authtype:   AuthType,
        expected_authtypes: Vec<AuthType>,
        #[snafu(implicit)]
        loc:                snafu::Location,
    },
    #[snafu(display(
         "Error parsing AuthType::OAuth auth token from {}. See README.md for more \
                     information on auth tokens: {}",
                    path.display(),
                    help_link
    ))]
    InvalidOauthFile {
        source:    serde_json::Error,
        path:      PathBuf,
        help_link: String,
    },
    #[snafu(display(
         "Error parsing OAuth client credential from {}. See README.md for more \
                     information on auth tokens: {}",
                    path.display(),
                    help_link
    ))]
    InvalidOauthInputFile {
        source:    serde_json::Error,
        path:      PathBuf,
        help_link: String,
    },
    #[snafu(display(
        "Token refresh failed, client is no longer usable. Please recreate the client and \
         re-authenticate."
    ))]
    TokenRefreshFailed {
        #[snafu(implicit)]
        loc: snafu::Location,
    },
    #[snafu(display("Operation cancelled"))]
    OperationCancelled {
        #[snafu(implicit)]
        loc: snafu::Location,
    },
}
