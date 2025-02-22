use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    HttpServer::new(|| App::new())
        .bind(("127.0.0.1", 8081))?
        .run()
        .await?;

    Ok(())
}
