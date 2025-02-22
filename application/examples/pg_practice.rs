use dotenv::dotenv;
use sqlx::prelude::*;
use sqlx::PgPool;

// easy postgres usage sample

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // load params from local `.env`
    dotenv().ok();
    let pg_host = std::env::var("PG_HOST").unwrap_or(String::from("localhost"));
    let pg_password = std::env::var("PG_PASSWORD").unwrap_or(String::from("postgres"));
    // construct the connection info url
    let url = format!(
        "postgres://postgres:{}@{}:5432/postgres",
        pg_password, pg_host
    );

    let pool = PgPool::connect(&url).await?;

    // apply the query
    let result = sqlx::query("SELECT id, name FROM users WHERE id = $1")
        .bind(1)
        .try_map(|row: sqlx::postgres::PgRow| {
            let id: i32 = row.get(0);
            let name: String = row.get(1);
            Ok((id, name))
        })
        .fetch_one(&pool)
        .await?;

    println!("User {} has ID {}", result.1, result.0);
    Ok(())
}
