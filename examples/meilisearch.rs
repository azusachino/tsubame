use meilisearch_sdk::client::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Movie {
    id: usize,
    title: String,
    genres: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("", "");

    let movies = client.index("movies");

    movies
        .add_documents(
            &[
                Movie {
                    id: 1,
                    title: String::from("Carol"),
                    genres: vec!["Romance".to_string(), "Drama".to_string()],
                },
                Movie {
                    id: 2,
                    title: String::from("Wonder Woman"),
                    genres: vec!["Action".to_string(), "Adventure".to_string()],
                },
                Movie {
                    id: 3,
                    title: String::from("Life of Pi"),
                    genres: vec!["Adventure".to_string(), "Drama".to_string()],
                },
                Movie {
                    id: 4,
                    title: String::from("Mad Max"),
                    genres: vec!["Adventure".to_string(), "Science Fiction".to_string()],
                },
                Movie {
                    id: 5,
                    title: String::from("Moana"),
                    genres: vec!["Fantasy".to_string(), "Action".to_string()],
                },
                Movie {
                    id: 6,
                    title: String::from("Philadelphia"),
                    genres: vec!["Drama".to_string()],
                },
            ],
            Some("id"),
        )
        .await?;

    // basic search
    println!(
        "{:?}",
        client
            .index("movies_2")
            .search()
            .with_query("caorl")
            .execute::<Movie>()
            .await?
            .hits
    );
    Ok(())
}
