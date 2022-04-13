use serde::{Deserialize, Deserializer};
use serde_derive::{Deserialize, Serialize};
use toml::{self, Value};

use crate::errors::*;
use crate::utils::{self, toml_ext::TomlExt};
use std::str::FromStr;

/// The overall configuration object for tsubame, essentially an in-memory representation of `config.toml`
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub app: AppConfig,
    pub mysql: MysqlConfig,
}

impl FromStr for Config {
    type Err = Error;
    fn from_str(src: &str) -> Result<Self> {
        toml::from_str(src).with_context(|| "Invalid configuration file")
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            app: AppConfig::default(),
            mysql: MysqlConfig::default(),
        }
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(de: D) -> std::result::Result<Self, D::Error> {
        Ok(Config::default())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    /**
     * server port
     */
    pub port: u16,
}

impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig { port: 8585 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MysqlConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub db: String,
}

impl Default for MysqlConfig {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            port: 3306,
            username: String::from("root"),
            password: String::from("root"),
            db: String::from("mysql"),
        }
    }
}
