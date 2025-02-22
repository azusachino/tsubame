//!
//! Tsubame Common Local Library
//!

mod structs;
mod utils;

mod stats {}

pub use anyhow::Result;
pub use structs::tsubame::RootConfig;
pub use utils::toml_ext;

pub use structs::tree::TreeNode;

pub mod push_commons {
    tonic::include_proto!("push_commons");
}

pub use utils::read_env_config;
