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

//! Module to allow dynamic use of the generic 'YtMusic' struct at runtime.
pub mod client;
pub mod err;

use std::path::Path;

use err::Result;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, ensure};
use strum::EnumProperty;
use tokio::io::AsyncWriteExt;
use tracing::info;
use ytmapi_rs::auth::OAuthToken;

use crate::ytapi::err::{FileIOSnafu, IOSnafu, InvalidConfigPathSnafu, InvalidOauthFileSnafu};

#[derive(Copy, PartialEq, Clone, Default, Debug, Serialize, Deserialize, EnumProperty)]
pub enum AuthType {
    #[default]
    #[strum(props(filename = "cookie.txt"))]
    Browser,
    #[strum(props(
        filename = "oauth.json",
        input_filename = "oauth.input.json",
        setup_url = "https://github.com/nick42d/youtui?tab=readme-ov-file#oauth-setup-steps-optional"
    ))]
    OAuth,
    Unauthenticated,
}

impl AuthType {
    pub async fn load<P: ?Sized + AsRef<Path>>(&self, p: &P) -> Result<ApiKey> {
        if matches!(self, &AuthType::Unauthenticated) {
            return Ok(ApiKey::None);
        }
        let p = p.as_ref();
        ensure!(
            p.exists(),
            InvalidConfigPathSnafu {
                path: p.to_path_buf(),
            }
        );
        let filename = self
            .get_str("filename")
            .expect("fofilenamerce property should exist")
            .to_owned();
        let filepath = p.join(filename);
        return match self {
            AuthType::Browser => {
                let filecontent = tokio::fs::read_to_string(&filepath)
                    .await
                    .context(FileIOSnafu { path: filepath })?;
                Ok(ApiKey::BrowserToken(filecontent))
            }
            AuthType::OAuth => {
                if !filepath.exists() {
                    let inputfilepath = p.join(
                        self.get_str("input_filename")
                            .expect("fofilenamerce property should exist")
                            .to_owned(),
                    );
                    let filecontent = tokio::fs::read_to_string(&inputfilepath)
                        .await
                        .context(IOSnafu)?;
                    let input = serde_json::from_str::<OAuthInput>(&filecontent).context(
                        InvalidOauthFileSnafu {
                            path:      inputfilepath,
                            help_link: self.get_str("setup_url").unwrap().to_owned(),
                        },
                    )?;
                    let token = input.get_oauth_token().await?;
                    let apikey = ApiKey::OAuthToken(token);
                    apikey.update_file(&p).await?;
                }

                let filecontent = tokio::fs::read_to_string(&filepath)
                    .await
                    .context(IOSnafu)?;
                let v = serde_json::from_str::<OAuthToken>(&filecontent).context(
                    InvalidOauthFileSnafu {
                        path:      filepath,
                        help_link: self.get_str("setup_url").unwrap().to_owned(),
                    },
                )?;
                Ok(ApiKey::OAuthToken(v))
            }
            AuthType::Unauthenticated => unreachable!(),
        };
    }
}

#[derive(Serialize, Deserialize)]
pub enum ApiKey {
    OAuthToken(OAuthToken),
    // BrowserToken takes the cookie, not the BrowserToken itself. This is because to obtain the
    // BrowserToken you must make a web request, and we want to obtain it as lazily as possible.
    BrowserToken(String),
    None,
}

impl ApiKey {
    /// Update the auth file.
    pub async fn update_file<P: AsRef<Path>>(&self, dir: &P) -> Result<()> {
        match self {
            ApiKey::OAuthToken(oauth_token) => {
                let dir = dir.as_ref();
                let filename = AuthType::OAuth
                    .get_str("filename")
                    .expect("fofilenamerce property should exist")
                    .to_owned();
                let filepath = dir.join(filename);

                let tmpfile_path = dir.join("json.tmp");
                let out =
                    serde_json::to_string_pretty(&oauth_token).context(InvalidOauthFileSnafu {
                        path:      filepath.clone(),
                        help_link: AuthType::OAuth.get_str("setup_url").unwrap().to_owned(),
                    })?;
                info!("Updating oauth token at: {:?}", &filepath);
                let mut file = tokio::fs::File::create(&tmpfile_path)
                    .await
                    .context(IOSnafu)?;
                file.write_all(out.as_bytes()).await.context(IOSnafu)?;
                file.flush().await.context(IOSnafu)?;
                file.sync_all().await.context(IOSnafu)?;
                tokio::fs::rename(tmpfile_path, &filepath)
                    .await
                    .context(IOSnafu)?;
                info!("Updated oauth token at: {:?}", filepath);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthInput {
    pub client_id:     String,
    pub client_secret: String,
}

impl OAuthInput {
    async fn get_oauth_token(&self) -> Result<OAuthToken> {
        let client = ytmapi_rs::client::Client::new_rustls_tls()?;
        let (code, url) = ytmapi_rs::generate_oauth_code_and_url(&client, &self.client_id).await?;
        // Hack to wait for input
        println!("Go to {url}, finish the login flow, and press enter when done");
        let mut _buf = String::new();
        let _ = std::io::stdin().read_line(&mut _buf);
        let token =
            ytmapi_rs::generate_oauth_token(&client, code, &self.client_id, &self.client_secret)
                .await?;
        Ok(token)
    }
}
