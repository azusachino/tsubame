use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RootConfig {
    pub server: ServerConfig,
    pub postgres: PostgresConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub db: String,
}

impl RootConfig {
    pub fn from_disk<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: RootConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

impl Default for RootConfig {
    fn default() -> Self {
        RootConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_owned(),
                port: 8080,
            },
            postgres: PostgresConfig {
                host: "127.0.0.1".to_owned(),
                port: 5432,
                username: "postgres".to_owned(),
                password: "postgres".to_owned(),
                db: "postgres".to_owned(),
            },
        }
    }
}
