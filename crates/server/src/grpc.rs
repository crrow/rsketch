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

use async_trait::async_trait;
use rsketch_common::{
    error::{ParseAddressSnafu, Result},
    readable_size::ReadableSize,
};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tonic::{service::RoutesBuilder, transport::Server};
use tonic_health::server::HealthReporter;
use tonic_reflection::server::v1::{ServerReflection, ServerReflectionServer};
use tracing::info;

use crate::ServiceHandler;

/// Default maximum gRPC receiving message size (512 MB)
pub const DEFAULT_MAX_GRPC_RECV_MESSAGE_SIZE: ReadableSize = ReadableSize::mb(512);
/// Default maximum gRPC sending message size (512 MB)
pub const DEFAULT_MAX_GRPC_SEND_MESSAGE_SIZE: ReadableSize = ReadableSize::mb(512);

/// Configuration options for a gRPC server
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, bon::Builder)]
pub struct GrpcServerConfig {
    /// The address to bind the gRPC server
    pub bind_address:          String,
    /// The address to advertise to clients
    pub server_address:        String,
    /// Maximum gRPC receiving (decoding) message size
    pub max_recv_message_size: ReadableSize,
    /// Maximum gRPC sending (encoding) message size
    pub max_send_message_size: ReadableSize,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            bind_address:          "127.0.0.1:50051".to_string(),
            server_address:        "127.0.0.1:50051".to_string(),
            max_recv_message_size: DEFAULT_MAX_GRPC_RECV_MESSAGE_SIZE,
            max_send_message_size: DEFAULT_MAX_GRPC_SEND_MESSAGE_SIZE,
        }
    }
}

/// Trait for gRPC service implementations that provides a standardized way to
/// register services with the gRPC server.
///
/// This trait abstracts the common patterns needed for gRPC services:
/// - Service registration with the tonic routes builder
/// - Reflection support through file descriptor sets
/// - Service identification for logging and monitoring
/// - Health status management
///
/// By implementing this trait, services can be easily integrated into the
/// GrpcServer framework, which handles server lifecycle, reflection setup,
/// health checking, and graceful shutdown automatically.
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
///
/// use tonic::service::RoutesBuilder;
/// use tonic_health::server::HealthReporter;
///
/// use crate::grpc::GrpcService;
///
/// pub struct MyServiceImpl {}
///
/// impl GrpcService for MyServiceImpl {
///     const FILE_DESCRIPTOR_SET: &'static [u8] = include_bytes!("../proto/my_service.bin");
///     const SERVICE_NAME: &'static str = "MyService";
///
///     fn register_service(self: Arc<Self>, builder: &mut RoutesBuilder) {
///         let service = my_service_server::MyServiceServer::new(self);
///         builder.add_service(service);
///     }
///
///     async fn register_health_status(&self, reporter: &HealthReporter) {
///         // Register this service as healthy
///         reporter
///             .set_serving::<my_service_server::MyServiceServer<Self>>()
///             .await;
///     }
/// }
/// ```
#[async_trait]
pub trait GrpcServiceHandler: Send + Sync + 'static {
    /// The name of the service for logging and identification purposes
    fn service_name(&self) -> &'static str;
    /// The compiled protobuf file descriptor set used for gRPC reflection
    /// This should be generated using prost-build and included at compile time
    fn file_descriptor_set(&self) -> &'static [u8];
    /// Register the service implementation with the tonic routes builder
    /// This method should wrap the service in the appropriate tonic-generated
    /// server and add it to the builder
    fn register_service(self: &Arc<Self>, builder: &mut RoutesBuilder);
    /// readiness_reporting is called after the service is registered and
    /// allows the service to set its initial health status
    async fn readiness_reporting(
        self: &Arc<Self>,
        _cancellation_token: CancellationToken,
        health_reporter: HealthReporter,
    ) {
        // Default implementation does nothing - services can override this
        // to set their specific health status
        health_reporter
            .set_service_status("", tonic_health::ServingStatus::Serving)
            .await;
    }
}

/// Starts the gRPC server and returns a handle for managing its lifecycle.
///
/// This method:
/// 1. Sets up the gRPC reflection service using the service's file descriptor
///    set
/// 2. Sets up the health checking service if enabled
/// 3. Parses and binds to the configured address
/// 4. Spawns the server in a background task
/// 5. Returns a handle for lifecycle management
///
/// The server will automatically register the reflection service, health
/// service (if enabled), and the provided service implementation. It
/// supports graceful shutdown through the returned handle.
///
/// # Errors
/// Returns an error if the bind address cannot be parsed.
pub async fn start_grpc_server(
    config: GrpcServerConfig,
    services: Vec<Arc<impl GrpcServiceHandler>>,
) -> Result<ServiceHandler> {
    // Parse bind address
    let bind_addr = config
        .bind_address
        .parse::<std::net::SocketAddr>()
        .context(ParseAddressSnafu {
            addr: config.bind_address.clone(),
        })?;

    let reflection_service = {
        let mut file_descriptor_sets = Vec::new();
        for service in services.iter() {
            file_descriptor_sets.push(service.file_descriptor_set());
        }
        file_descriptor_sets.push(tonic_reflection::pb::v1::FILE_DESCRIPTOR_SET);
        build_reflection_service(&file_descriptor_sets)
    };

    let (reporter, health_service) = tonic_health::server::health_reporter();
    let mut routes_builder = RoutesBuilder::default();
    routes_builder
        .add_service(health_service)
        .add_service(reflection_service);

    // register services
    for service in services.iter() {
        let service = service.clone();
        service.register_service(&mut routes_builder);
    }

    // Spawn the server task
    let cancellation_token = CancellationToken::new();
    let (join_handle, started_rx) = {
        let (started_tx, started_rx) = oneshot::channel::<()>();
        let cancellation_token_clone = cancellation_token.clone();
        let join_handle = tokio::spawn(async move {
            let result = Server::builder()
                .accept_http1(true)
                .add_routes(routes_builder.routes())
                .serve_with_shutdown(bind_addr, async move {
                    info!("gRPC server (on {}) starting", bind_addr);
                    let _ = started_tx.send(());
                    info!("gRPC server (on {}) started", bind_addr);
                    cancellation_token_clone.cancelled().await;
                    info!("gRPC server (on {}) received shutdown signal", bind_addr);
                })
                .await;

            info!(
                "gRPC server (on {}) task completed: {:?}",
                bind_addr, result
            );
        });
        (join_handle, started_rx)
    };

    let reporter_handlers = {
        let mut handlers = Vec::new();
        for service in services.iter() {
            info!(
                "spawning readiness reporting task for {}",
                service.service_name()
            );
            let service = service.clone();
            let reporter = reporter.clone();
            let cancellation_token_clone = cancellation_token.clone();
            let handle = tokio::spawn(async move {
                service
                    .readiness_reporting(cancellation_token_clone, reporter)
                    .await;
                info!(
                    "readiness reporting task for {} completed",
                    service.service_name()
                );
            });
            handlers.push(handle);
        }
        handlers
    };

    let handle = ServiceHandler {
        join_handle,
        cancellation_token,
        started_rx: Some(started_rx),
        reporter_handles: reporter_handlers,
    };
    Ok(handle)
}

fn build_reflection_service(
    file_descriptor_sets: &[&[u8]],
) -> ServerReflectionServer<impl ServerReflection> {
    let mut builder = tonic_reflection::server::Builder::configure();

    for file_descriptor_set in file_descriptor_sets {
        builder = builder.register_encoded_file_descriptor_set(file_descriptor_set)
    }
    builder
        .build_v1()
        .expect("failed to build reflection service")
}

#[cfg(test)]
mod tests {
    use tracing_subscriber;

    use super::*;

    fn init_test_logging() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_line_number(true)
            .try_init();
    }

    #[derive(Default)]
    struct HelloService;

    #[async_trait::async_trait]
    impl rsketch_api::pb::hello::v1::hello_server::Hello for HelloService {
        async fn hello(
            &self,
            _request: tonic::Request<()>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            Ok(tonic::Response::new(()))
        }
    }

    #[async_trait::async_trait]
    impl GrpcServiceHandler for HelloService {
        fn service_name(&self) -> &'static str { "Hello" }

        fn file_descriptor_set(&self) -> &'static [u8] { rsketch_api::pb::GRPC_DESC }

        fn register_service(self: &Arc<Self>, builder: &mut RoutesBuilder) {
            use tonic::service::LayerExt as _;
            let svc = tower::ServiceBuilder::new()
                .layer(tower_http::cors::CorsLayer::new())
                .layer(tonic_web::GrpcWebLayer::new())
                .into_inner()
                .named_layer(
                    rsketch_api::pb::hello::v1::hello_server::HelloServer::from_arc(self.clone()),
                );
            builder.add_service(svc);
        }

        async fn readiness_reporting(
            self: &Arc<Self>,
            _cancellation_token: CancellationToken,
            reporter: HealthReporter,
        ) {
            // Register the Hello service as healthy
            reporter
                .set_serving::<rsketch_api::pb::hello::v1::hello_server::HelloServer<HelloService>>(
                )
                .await;
        }
    }

    #[tokio::test]
    async fn test_grpc_server_lifecycle() {
        init_test_logging();

        let config = GrpcServerConfig {
            ..Default::default()
        };
        let mut handle = start_grpc_server(config.clone(), vec![Arc::new(HelloService)])
            .await
            .expect("start_grpc_server failed");

        // Wait for server to start
        handle.wait_for_start().await.unwrap();
        info!("Server started successfully");

        // Create a gRPC client and send an RPC request
        let mut client = rsketch_api::pb::hello::v1::hello_client::HelloClient::connect(format!(
            "http://{}",
            config.server_address
        ))
        .await
        .unwrap();

        // Send a hello request
        let request = tonic::Request::new(());
        let response = client.hello(request).await.unwrap();
        info!("Received response: {:?}", response);

        // Signal shutdown
        handle.shutdown();

        // Wait for server to stop
        handle.wait_for_stop().await.unwrap();
        info!("Server stopped successfully");
    }

    #[tokio::test]
    async fn test_readiness_updates() {
        use tonic::transport::Channel;
        use tonic_health::pb::{
            HealthCheckRequest, health_check_response::ServingStatus, health_client::HealthClient,
        };

        // start the server
        let config = GrpcServerConfig::default();
        let mut handle = start_grpc_server(config.clone(), vec![Arc::new(HelloService)])
            .await
            .expect("start_grpc_server failed");
        // Wait for server to start
        handle.wait_for_start().await.unwrap();
        info!("Server started successfully");

        // Create a channel and connect to the gRPC server
        let channel = Channel::from_shared("http://localhost:50051")
            .unwrap()
            .connect()
            .await
            .unwrap();
        // Create the health client using the connected channel
        let mut health_client = HealthClient::new(channel);

        // Check initial health status
        let request = tonic::Request::new(HealthCheckRequest::default());
        let response = health_client.check(request).await.unwrap().into_inner();
        assert_eq!(response.status(), ServingStatus::Serving);

        // Signal shutdown
        handle.shutdown();
    }
}
