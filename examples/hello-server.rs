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

use std::sync::Arc;

use rsketch_common::telemetry::logging::{init_global_logging, LoggingOptions, OtlpExportProtocol};
use rsketch_server::grpc::{hello::HelloService, start_grpc_server, GrpcServerConfig};
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let logging_opts = LoggingOptions {
            enable_otlp_tracing: true,
            otlp_export_protocol: Some(OtlpExportProtocol::Grpc),
            log_format: rsketch_common::telemetry::logging::LogFormat::Json,
            ..Default::default()
        };
        let _guards = init_global_logging("hello-server", &logging_opts, &Default::default(), None);

        let config = GrpcServerConfig {
            bind_address: "[::1]:50051".to_string(),
            ..Default::default()
        };
        let service = Arc::new(HelloService);
        let server_handle = start_grpc_server(config, vec![service]).await.unwrap();

        println!("Hello server listening on [::1]:50051");

        tokio::signal::ctrl_c().await.unwrap();
        server_handle.shutdown();
        server_handle.wait_for_stop().await.unwrap();
    });
}
