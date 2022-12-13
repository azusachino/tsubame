use crate::errors::*;
use crate::toml_ext::TomlExt;
use serde::{Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use toml::value::Table;
use toml::{self, Value};

/// The overall configuration object for tsubame, essentially an in-memory representation of `config.toml`
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub app: AppConfig,
    pub postgres: PostgresqlConfig,
    rest: Value,
}

impl FromStr for Config {
    type Err = Error;
    fn from_str(src: &str) -> Result<Self> {
        toml::from_str(src).with_context(|| "Invalid configuration file")
    }
}

impl Config {
    /**
     * Load Config from disk file
     */
    pub fn from_disk<P: AsRef<Path>>(config_file: P) -> Result<Config> {
        let mut buffer = String::new();
        File::open(config_file)
            .with_context(|| "Unable to open the configuration file")?
            .read_to_string(&mut buffer)
            .with_context(|| "Couldn't read the file")?;

        Config::from_str(&buffer)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            app: AppConfig::default(),
            postgres: PostgresqlConfig::default(),
            rest: Value::Table(Table::default()),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(de: D) -> std::result::Result<Self, D::Error> {
        let raw = Value::deserialize(de)?;

        use serde::de::Error;
        let mut table = match raw {
            Value::Table(t) => t,
            _ => {
                return Err(Error::custom("A config file should always be a toml table"));
            }
        };
        let app: AppConfig = table
            .remove("app")
            .map(|app| app.try_into().map_err(Error::custom))
            .transpose()?
            .unwrap_or_default();

        let postgres: PostgresqlConfig = table
            .remove("postgres")
            .map(|app| app.try_into().map_err(Error::custom))
            .transpose()?
            .unwrap_or_default();
        Ok(Config {
            app,
            postgres: postgres,
            rest: Value::Table(table),
        })
    }
}

impl serde::Serialize for Config {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let mut table = self.rest.clone();
        let app_config = Value::try_from(&self.app).expect("should always be serializable");
        table.insert("app", app_config);
        let postgres_config =
            Value::try_from(&self.postgres).expect("should always be serializable");
        table.insert("postgres", postgres_config);

        table.serialize(s)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    // server port
    pub port: u16,
}

impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig { port: 8585 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostgresqlConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    // user db
    pub db: String,
}

impl Default for PostgresqlConfig {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            port: 5432,
            username: String::from("postgres"),
            password: String::from("postgres"),
            db: String::from("postgres"),
        }
    }
}

impl PostgresqlConfig {
    /// format for sqlx pool connect
    pub fn to_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.db
        )
    }
}
