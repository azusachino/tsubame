use anyhow::Result;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Router, body::Body};
use tonic::{Response, Status, transport::Server};
use tsubame_application::interceptor::{jwt_auth, rate_limit};
use tsubame_commons::push_commons::push_service_server::{PushService, PushServiceServer};
use tsubame_commons::push_commons::{PushRequest, PushResponse};

// switch malloc
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Debug, Default)]
pub struct GrpcPushService {}

#[tonic::async_trait]
impl PushService for GrpcPushService {
    async fn v1(
        &self,
        request: tonic::Request<PushRequest>,
    ) -> Result<Response<PushResponse>, Status> {
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
    // tsubame_application::load_config()?;

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

    let app_state = rate_limit::AppState::new(10);

    let app = Router::new()
        .route("/status", get(|| async { "OK" }))
        // header check middleware
        .layer(middleware::from_fn(header_interceptor))
        .route(
            "/users",
            post(tsubame_application::service::user::create_user),
        )
        .layer(middleware::from_fn(jwt_auth::jwt_auth))
        // .layer(middleware::from_fn_with_state(
        //     app_state.clone(),
        //     rate_limit::rate_limit,
        // ))
        .with_state(app_state);
    let http_server = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("http server listening on {}", http_server.local_addr()?);
    axum::serve(http_server, app).await?;
    Ok(())
}

async fn header_interceptor(
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    for (name, value) in req.headers() {
        tracing::info!("{}: {}", name.as_str(), value.to_str().unwrap());
    }
    Ok(next.run(req).await)
}
