use std::{net::SocketAddr, time::Duration};
use std::path::Path;

use anyhow::Result;
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    Router,
    routing::get,
};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

use tsubame::{Config, CURRENT_VERSION};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Our hope is a little tsubame, current {}", CURRENT_VERSION);

    // init config
    let config_location = Path::new(".").join("config.toml");
    let config = Config::from_disk(config_location)?;
    println!("current config: {:?}", config);

    let pool = MySqlPoolOptions::new()
        .max_connections(8)
        .connect_timeout(Duration::from_secs(3))
        .connect(&config.mysql.to_url())
        .await
        .expect("can connect to database");

    let app = Router::new()
        .route(
            "/",
            get(using_connection_pool_extractor).post(using_connection_extractor),
        )
        .layer(Extension(pool));
    let app_config = config.app;
    let address = SocketAddr::from(([172, 0, 0, 1], app_config.port));
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
    // use select 1 to validate MySQL connection
    sqlx::query_scalar("select 1")
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
