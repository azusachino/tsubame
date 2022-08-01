use std::path::Path;

use tsubame::Config;

fn main() {
    let root = Path::new(".");
    let config_location = root.join("config.toml");

    let config = if config_location.exists() {
        Config::from_disk(&config_location).unwrap()
    } else {
        Config::default()
    };

    println!("{:?}", config);
}
