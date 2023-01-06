//! # Tsubame
//!
//! A bless from [YOASOBI](https://en.wikipedia.org/wiki/Yoasobi)

mod internal;
pub mod config;

/// The current version of `tsubame`
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use crate::config::Config;

pub use tsubame_lib::toml_ext;

/// The error types used through out this crate.
#[allow(unused_imports)]
pub mod errors {
    pub(crate) use anyhow::Context;
    pub use anyhow::{Error, Result};
}
