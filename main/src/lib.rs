//! # Tsubame
//!
//! A bless from [YOASOBI](https://en.wikipedia.org/wiki/Yoasobi)

pub mod config;
mod internal;

/// The current version of `tsubame`
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use crate::config::Config;

pub use tsubame_lib::toml_ext;

/// The error types used through out this crate.
pub mod errors {
    pub(crate) use anyhow::Context;
    pub use anyhow::{Error, Result};
}

pub fn load_config() -> anyhow::Result<()> {
    println!(
        "Our future is like a tsubame, current version is {}",
        CURRENT_VERSION
    );

    // init config
    let config_location = std::path::Path::new(".").join("config.toml");
    let config = Config::from_disk(config_location)?;
    println!("current config: {:?}", config);
    Ok(())
}
