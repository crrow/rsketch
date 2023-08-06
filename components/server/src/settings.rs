use std::io;

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ServiceConfig {
    #[validate(length(min = 1))]
    pub host: String,
    pub tcp_port: Option<u16>,  // None means that tcp is disabled
    pub udp_port: Option<u16>,  // None means that udp is disabled
    pub http_port: Option<u16>, // None means that http is disabled
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
}
