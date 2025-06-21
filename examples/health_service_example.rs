use std::sync::Arc;

use rsketch_services::grpc::{GrpcServer, GrpcServerConfig, GrpcService};
use tonic::service::RoutesBuilder;
use tonic_health::server::HealthReporter;
use tracing::info;

/// Example service implementation that demonstrates health service integration
#[derive(Default)]
struct ExampleService {
    /// Simulate some internal state that affects health
    healthy: bool,
}

impl ExampleService {
    fn new() -> Self { Self { healthy: true } }

    /// Simulate a method that could change health status
    fn mark_unhealthy(&mut self) {
        self.healthy = false;
        info!("Service marked as unhealthy");
    }

    /// Simulate a method that could restore health status
    fn mark_healthy(&mut self) {
        self.healthy = true;
        info!("Service marked as healthy");
    }
}

/// Implementation of GrpcService for our example service
#[async_trait::async_trait]
impl GrpcService for ExampleService {
    const FILE_DESCRIPTOR_SET: &'static [u8] = &[];
    const SERVICE_NAME: &'static str = "ExampleService";

    // In real usage, this would be your proto descriptor

    fn register_service(self: Arc<Self>, builder: &mut RoutesBuilder) {
        // In a real implementation, you would register your actual gRPC service here
        // For this example, we'll just log that the service is being registered
        info!("Registering ExampleService with gRPC server");

        // Example of how you would register a real service:
        // let service = your_service_server::YourServiceServer::new(self);
        // builder.add_service(service);
    }

    async fn register_health_status(&self, reporter: &HealthReporter) {
        if self.healthy {
            info!("Registering ExampleService as healthy");
            // In a real implementation, you would register your actual service
            // type: reporter
            //     .set_serving::<your_service_server::YourServiceServer<Self>>()
            //     .await;
        } else {
            info!("Registering ExampleService as unhealthy");
            // reporter
            //     .set_not_serving::<your_service_server::YourServiceServer<Self>>()
            //     .await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create service configuration with health checking enabled
    let config = GrpcServerConfig {
        bind_address: "127.0.0.1:50051".to_string(),
        server_address: "127.0.0.1:50051".to_string(),
        enable_health_checker: true,
        enable_reflection: true,
        ..Default::default()
    };

    // Create the service
    let service = ExampleService::new();

    // Create and start the gRPC server
    let mut server = GrpcServer::new(config, service);
    let mut handle = server.start().await?;

    info!("Waiting for server to start...");
    handle.wait_for_start().await?;
    info!("Server started successfully!");

    // The server is now running with health checking enabled
    // You can test the health endpoint using a gRPC client or tools like grpcurl:
    // grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check

    info!("Press Ctrl+C to stop the server...");

    // Wait for shutdown signal (in a real application, you might wait for a signal)
    tokio::signal::ctrl_c().await?;

    info!("Shutting down server...");
    handle.shutdown();
    handle.wait_for_stop().await?;

    info!("Server stopped successfully!");
    Ok(())
}
