use anyhow::Result;

use axum::Router;
use axum::routing::{get, post};
use tonic::{Request, Response, Status, transport::Server};
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
    // initializing logger, dotenv and config
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
    tsubame_application::load_config()?;

    let grpc_push_service = GrpcPushService::default();

    tokio::spawn(async move {
        let addr = "[::1]:50002".parse().expect("failed to parse address");
        tracing::info!("grpc server listening on {}", addr);
        Server::builder()
            .add_service(PushServiceServer::new(grpc_push_service))
            .serve(addr)
            .await
            .expect("failed to start server");
    });

    let app = Router::new()
        .route("/status", get(|| async { "OK" }))
        .route(
            "/users",
            post(tsubame_application::service::user::create_user),
        );
    let http_server = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("http server listening on {}", http_server.local_addr()?);
    axum::serve(http_server, app).await?;
    Ok(())
}
