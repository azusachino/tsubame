#![allow(unused)]
use std::net::SocketAddr;

use axum::{
    extract::Path,
    routing::{delete, get},
    Router,
};

async fn root() {}

async fn list_users() {}

async fn create_user() {}

async fn show_user(Path(id): Path<u64>) {}

async fn do_users_action(Path((version, id)): Path<(String, u64)>) {}

async fn serve_asset(Path(path): Path<String>) {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/users", get(list_users).post(create_user))
        // catch variables
        .route("/users/:id", get(show_user))
        .route("/api/:version/users/:id/action", delete(do_users_action))
        // matches all segments
        .route("/assets/*path", get(serve_asset));

    let socket_addr = SocketAddr::from(([0, 0, 0, 0], 3002));
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
