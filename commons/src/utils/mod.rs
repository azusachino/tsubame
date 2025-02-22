pub mod toml_ext;

use anyhow::Result;

use crate::RootConfig;

// read the configuration with environment setting
pub fn read_env_config() -> Result<crate::RootConfig> {
    let env = if let Ok(env) = std::env::var("ENV") {
        env
    } else {
        "dev".to_owned()
    };
    let cwd = std::env::current_dir()?;
    let env_cfg_path = format!("config.{}.toml", env);
    let config_location = cwd.join(env_cfg_path);

    let cfg = if config_location.exists() {
        RootConfig::from_disk(config_location)?
    } else {
        RootConfig::default()
    };

    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_env_config() {
        let cfg = read_env_config().unwrap();
        assert_eq!(cfg.server.port, 8080);
    }
}
