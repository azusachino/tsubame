use anyhow::Result;
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    routing::get,
    Router,
};

use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::{net::SocketAddr, time::Duration};
use tsubame::CURRENT_VERSION;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Our hope is a little tsubame, current {}", CURRENT_VERSION);

    let db_connection_str = std::env::var("MYSQL_URL")
        .unwrap_or_else(|_| "mysql://root:abcd1234@localhost".to_string());

    let pool = MySqlPoolOptions::new()
        .max_connections(8)
        .connect_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can connect to database");

    let app = Router::new()
        .route(
            "/",
            get(using_connection_pool_extractor).post(using_connection_extractor),
        )
        .layer(Extension(pool));
    let address = SocketAddr::from(([172, 0, 0, 1], 3000));
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

// Extract ConnectionPoll by using `Extension`
async fn using_connection_pool_extractor(
    Extension(pool): Extension<MySqlPool>,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("select 1")
        .fetch_one(&pool)
        .await
        .map_err(internal_error)
}

// Extract PoolConnection from request
struct DatabaseConnection(sqlx::pool::PoolConnection<sqlx::MySql>);

#[async_trait]
impl<B> FromRequest<B> for DatabaseConnection
where
    B: Send,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<MySqlPool>::from_request(req)
            .await
            .map_err(internal_error)?;
        let conn = pool.acquire().await.map_err(internal_error)?;
        Ok(Self(conn))
    }
}

async fn using_connection_extractor(
    DatabaseConnection(conn): DatabaseConnection,
) -> Result<String, (StatusCode, String)> {
    let mut conn = conn;
    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&mut conn)
        .await
        .map_err(internal_error)
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
