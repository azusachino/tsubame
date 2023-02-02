use axum::{routing, Router};

pub fn build_router() -> Router {
    Router::new().route("/", routing::get(root))
}

async fn root() {}
