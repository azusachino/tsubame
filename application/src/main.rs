use anyhow::Result;

use tonic::{transport::Server, Request, Response, Status};
use tsubame_commons::push_commons::push_service_server::{PushService, PushServiceServer};
use tsubame_commons::push_commons::{PushRequest, PushResponse};

// switch malloc
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Debug, Default)]
pub struct GrpcPushService {}

#[tonic::async_trait]
impl PushService for GrpcPushService {
    async fn v1(&self, request: Request<PushRequest>) -> Result<Response<PushResponse>, Status> {
        println!("Got a request: {:?}", request);
        let req = request.into_inner();
        let response = PushResponse {
            id: req.id,
            code: 200,
            message: "OK".into(),
            data: "Hello, World!".into(),
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:50001".parse()?;
    let grpc_push_service = GrpcPushService::default();
    Server::builder()
        .add_service(PushServiceServer::new(grpc_push_service))
        .serve(addr)
        .await?;
    Ok(())
}
