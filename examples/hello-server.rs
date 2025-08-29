use rsketch_server::grpc::hello::HelloService;
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let addr = "[::1]:50051".parse().unwrap();
        let server = HelloService::default();

        println!("Hello server listening on {}", addr);

        tonic::transport::Server::builder()
            .add_service(rsketch_api::pb::hello::v1::hello_server::HelloServer::new(
                server,
            ))
            .serve(addr)
            .await
            .unwrap();
    });
}
