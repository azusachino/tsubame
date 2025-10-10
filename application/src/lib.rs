//! # Tsubame
//!
//! A bless from [YOASOBI](https://en.wikipedia.org/wiki/Yoasobi)

pub mod config;
pub mod interceptor;
mod internal;
pub mod service;

/// The current version of `tsubame`
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

use std::{env, path::Path};

pub use crate::config::Config;

pub use tsubame_commons::toml_ext;

/// The error types used through out this crate.
pub mod errors {
    pub(crate) use anyhow::Context;
    pub use anyhow::{Error, Result};
}

pub fn load_config() -> anyhow::Result<()> {
    tracing::info!(
        "Our future is like a tsubame, current version is {}",
        CURRENT_VERSION
    );

    // init config
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_owned());
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| ".".to_owned());
    let config_file = Path::new(&config_path).join(format!("config.{}.toml", environment));
    let config = Config::from_disk(config_file)?;
    tracing::info!("current config path: {:?}, cfg: {:?}", &config_path, config);
    Ok(())
}
