use std::{env, io};

use config::{ConfigError, Source};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ServiceConfig {
    #[validate(length(min = 1))]
    pub host: String,
    pub tcp_port: Option<u16>,
    // None means that tcp is disabled
    pub udp_port: Option<u16>,
    // None means that udp is disabled
    pub http_port: Option<u16>,
    // None means that http is disabled
    pub grpc_port: Option<u16>, // None means that gRPC is disabled
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct TlsConfig {
    pub cert: String,
    pub key: String,
    pub ca_cert: String,
    #[serde(default = "default_tls_cert_ttl")]
    #[validate(range(min = 1))]
    pub cert_ttl: Option<u64>,
}

const fn default_tls_cert_ttl() -> Option<u64> {
    Some(3600)
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct Settings {
    #[serde(default = "default_debug")]
    pub debug: bool,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[validate]
    pub service: ServiceConfig,
    #[validate]
    pub tls: Option<TlsConfig>,
}

const fn default_debug() -> bool {
    false
}

fn default_log_level() -> String {
    "INFO".to_string()
}

impl Settings {
    pub fn tls(&self) -> io::Result<&TlsConfig> {
        self.tls
            .as_ref()
            .ok_or_else(Self::err_tls_config_is_undefined)
    }
    pub fn err_tls_config_is_undefined() -> io::Error {
        io::Error::new(
            io::ErrorKind::Other,
            "TLS config is not defined in the config file",
        )
    }
    pub fn new(path: Option<String>) -> Result<Self, ConfigError> {
        let config_exists = |path| config::File::with_name(path).collect().is_ok();
        // Check if custom config file exists, report error if not
        if let Some(ref path) = path {
            if !config_exists(path) {
                return Err(ConfigError::Message(format!(
                    "Config file {} does not exist",
                    path
                )));
            }
        }

        let env = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());
        let config_path_env = format!("config/{env}");

        // Configuration builder: define different levels of configuration files
        let mut config = config::Config::builder()
            // Start with compile-time base config
            .add_source(config::File::from_str(BASE_CONFIG, config::FileFormat::Toml))
            // Merge main config: config/config
            .add_source(config::File::with_name("config/config").required(false))
            // Merge env config: config/{env}
            // Uses RUN_MODE, defaults to 'development'
            .add_source(config::File::with_name(&config_path_env).required(false))
            // Merge local config, not tracked in git: config/local
            .add_source(config::File::with_name("config/local").required(false));

        // Merge user provided config with --config-path
        if let Some(path) = path {
            config = config.add_source(config::File::with_name(&path).required(false));
        }

        // Merge environment settings
        // E.g.: `QDRANT_DEBUG=1 ./target/app` would set `debug=true`
        config = config.add_source(config::Environment::with_prefix("OURO").separator("__"));
        // Build and merge config and deserialize into Settings, attach any load errors
        // we had
        let mut settings: Settings = config.build()?.try_deserialize()?;
        Ok(settings)
    }
}

const BASE_CONFIG: &str = "default.toml";
