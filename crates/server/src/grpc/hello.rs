use std::sync::Arc;

use async_trait::async_trait;
use rsketch_api::pb::hello::v1::hello_server;
use tonic::service::RoutesBuilder;
use tonic_health::server::HealthReporter;
use tokio_util::sync::CancellationToken;

use crate::grpc::GrpcServiceHandler;

#[derive(Default)]
pub struct HelloService;

#[async_trait]
impl hello_server::Hello for HelloService {
    async fn hello(
        &self,
        _request: tonic::Request<()>,    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        Ok(tonic::Response::new(()))
    }
}

#[async_trait]
impl GrpcServiceHandler for HelloService {
    fn service_name(&self) -> &'static str {
        "Hello"
    }

    fn file_descriptor_set(&self) -> &'static [u8] {
        rsketch_api::pb::GRPC_DESC
    }

    fn register_service(self: &Arc<Self>, builder: &mut RoutesBuilder) {
        builder.add_service(hello_server::HelloServer::from_arc(self.clone()));
    }

    async fn readiness_reporting(
        self: &Arc<Self>,
        _cancellation_token: CancellationToken,
        reporter: HealthReporter,
    ) {
        reporter
            .set_serving::<hello_server::HelloServer<HelloService>>()
            .await;
    }
}
