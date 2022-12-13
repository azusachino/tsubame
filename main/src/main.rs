use std::path::Path;

use anyhow::Result;

use tsubame::{Config, CURRENT_VERSION};

#[tokio::main]
async fn main() -> Result<()> {
    println!(
        "Our future is like a tsubame, current version is {}",
        CURRENT_VERSION
    );

    // init config
    let config_location = Path::new(".").join("config.toml");
    let config = Config::from_disk(config_location)?;
    println!("current config: {:?}", config);

    Ok(())
}
