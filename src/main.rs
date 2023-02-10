use std::sync::Arc;

use anyhow::Result;
use axum::{routing::get, Extension, Router};
use tsubame::GlobalState;

// async fn list_friends(_: Extension<GlobalState>) {}

#[tokio::main]
async fn main() -> Result<()> {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let gs = Arc::new(GlobalState::new().await);

    let app = Router::new()
        .route("/", get(|| async { "Hello World" }))
        // .route("/friends", get(list_friends))
        // Add middleware that inserts the state into all incoming request's
        // extensions.
        .layer(Extension(gs));

    axum::Server::bind(&"0.0.0.0:3001".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
