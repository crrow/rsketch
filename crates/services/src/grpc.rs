use std::sync::Arc;

use rsketch_common::{
    error::{ParseAddressSnafu, Result},
    readable_size::ReadableSize,
};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tonic::{
    service::{LayerExt as _, RoutesBuilder},
    transport::Server,
};
use tonic_health::server::HealthReporter;
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
    /// Enable reflection service
    pub enable_reflection:     bool,
    /// Enable health checker service
    pub enable_health_checker: bool,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            bind_address:          "127.0.0.1:50051".to_string(),
            server_address:        "127.0.0.1:50051".to_string(),
            max_recv_message_size: DEFAULT_MAX_GRPC_RECV_MESSAGE_SIZE,
            max_send_message_size: DEFAULT_MAX_GRPC_SEND_MESSAGE_SIZE,
            enable_reflection:     true,
            enable_health_checker: true,
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
///
/// By implementing this trait, services can be easily integrated into the
/// GrpcServer framework, which handles server lifecycle, reflection setup,
/// and graceful shutdown automatically.
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
///
/// use tonic::service::RoutesBuilder;
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
/// }
/// ```
pub trait GrpcService: Send + Sync + 'static {
    /// The name of the service for logging and identification purposes
    const SERVICE_NAME: &'static str;
    /// The compiled protobuf file descriptor set used for gRPC reflection
    /// This should be generated using prost-build and included at compile time
    const FILE_DESCRIPTOR_SET: &'static [u8];
    /// Register the service implementation with the tonic routes builder
    /// This method should wrap the service in the appropriate tonic-generated
    /// server and add it to the builder
    fn register_service(self: Arc<Self>, builder: &mut RoutesBuilder);
}

/// A gRPC server that manages the lifecycle of a gRPC service.
///
/// This struct provides a high-level interface for running gRPC services with
/// automatic reflection support, graceful shutdown, and lifecycle management.
/// It wraps a service implementation and handles all the boilerplate needed
/// to run a production-ready gRPC server.
///
/// # Features
/// - Automatic gRPC reflection service registration
/// - Graceful shutdown with cancellation tokens
/// - Configurable message size limits
/// - Lifecycle management through handles
///
/// # Example
/// ```rust
/// let config = GrpcServerConfig::default();
/// let service = MyServiceImpl::new();
/// let mut server = GrpcServer::new(config, service);
/// let handle = server.start().await?;
/// ```
pub struct GrpcServer<S> {
    config:       GrpcServerConfig,
    /// Service implementation wrapped in Arc for shared ownership
    service_impl: Arc<S>,
}

impl<S: GrpcService> GrpcServer<S> {
    /// Creates a new gRPC server with the given configuration and service
    /// implementation.
    ///
    /// The service is wrapped in an Arc to enable shared ownership across async
    /// tasks.
    pub fn new(config: GrpcServerConfig, service: S) -> Self {
        Self {
            config,
            service_impl: Arc::new(service),
        }
    }

    /// Starts the gRPC server and returns a handle for managing its lifecycle.
    ///
    /// This method:
    /// 1. Sets up the gRPC reflection service using the service's file
    ///    descriptor set
    /// 2. Parses and binds to the configured address
    /// 3. Spawns the server in a background task
    /// 4. Returns a handle for lifecycle management
    ///
    /// The server will automatically register the reflection service and the
    /// provided service implementation. It supports graceful shutdown through
    /// the returned handle.
    ///
    /// # Errors
    /// Returns an error if the bind address cannot be parsed.
    pub async fn start(&mut self) -> Result<ServiceHandler> {
        let (started_tx, started_rx) = oneshot::channel::<()>();
        let cancellation_token = CancellationToken::new();

        // Parse bind address
        let bind_addr = self
            .config
            .bind_address
            .parse::<std::net::SocketAddr>()
            .context(ParseAddressSnafu {
                addr: self.config.bind_address.clone(),
            })?;

        info!("Starting {} gRPC server on {}", S::SERVICE_NAME, bind_addr);

        // Clone necessary data for the async task
        let service_impl = self.service_impl.clone();
        let cancellation_token_clone = cancellation_token.clone();
        let config = self.config.clone();

        // Spawn the server task
        let join_handle = tokio::spawn(async move {
            let mut routes_builder = RoutesBuilder::default();

            // register reflection service if enabled
            if config.enable_reflection {
                // Set up reflection service
                let builder = tonic_reflection::server::Builder::configure()
                    .register_encoded_file_descriptor_set(S::FILE_DESCRIPTOR_SET)
                    .with_service_name(S::SERVICE_NAME);
                let reflection_service = builder
                    .build_v1()
                    .expect("Failed to build reflection service");
                routes_builder.add_service(reflection_service);
            }

            // Register health checker service if enabled
            if config.enable_health_checker {
                let (hr, health_service) = tonic_health::server::health_reporter();
                routes_builder.add_service(health_service);
                hr.set_serving::<tonic_health::pb::health_server::HealthServer<
                    tonic_health::server::HealthService,
                >>()
                .await;
            }
            service_impl.register_service(&mut routes_builder);

            let result = Server::builder()
                .accept_http1(true)
                .add_routes(routes_builder.routes())
                .serve_with_shutdown(bind_addr, async move {
                    let _ = started_tx.send(());
                    info!(
                        "{} gRPC server (on {}) waiting for shutdown signal",
                        S::SERVICE_NAME,
                        bind_addr
                    );
                    cancellation_token_clone.cancelled().await;
                    info!(
                        "{} gRPC server (on {}) received shutdown signal",
                        S::SERVICE_NAME,
                        bind_addr
                    );
                })
                .await;

            info!(
                "{} gRPC server (on {}) task completed: {:?}",
                S::SERVICE_NAME,
                bind_addr,
                result
            );
        });

        let handle = ServiceHandler {
            join_handle,
            cancellation_token,
            started_rx: Some(started_rx),
        };
        Ok(handle)
    }
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

    #[tokio::test]
    async fn test_grpc_server_lifecycle() {
        init_test_logging();

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
        impl GrpcService for HelloService {
            const FILE_DESCRIPTOR_SET: &'static [u8] = rsketch_api::pb::GRPC_DESC;
            const SERVICE_NAME: &'static str = "Hello";

            fn register_service(self: Arc<HelloService>, builder: &mut RoutesBuilder) {
                let svc = tower::ServiceBuilder::new()
                    .layer(tower_http::cors::CorsLayer::new())
                    .layer(tonic_web::GrpcWebLayer::new())
                    .into_inner()
                    .named_layer(
                        rsketch_api::pb::hello::v1::hello_server::HelloServer::from_arc(self),
                    );
                builder.add_service(svc);
            }
        }

        let config = GrpcServerConfig::default();
        let mut server = GrpcServer::new(config.clone(), HelloService);

        // Start the server
        let mut handle = server.start().await.unwrap();

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
}
