//! # Tsubame
//!
//! A bless from [YOASOBI](https://en.wikipedia.org/wiki/Yoasobi)

pub mod config;
pub mod utils;

/// The current version of `tsubame`
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use crate::config::Config;

/// The error types used through out this crate.
pub mod errors {
    pub(crate) use anyhow::Context;
    pub use anyhow::{Error, Result};
}
