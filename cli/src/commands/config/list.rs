use crate::{
    config::Config as CliConfig,
    error::{ConfigError, WKCliError},
};

pub fn handle_list() -> Result<bool, WKCliError> {
    let config = CliConfig::load_default_path()?;

    println!(
        "{}",
        toml::to_string(&config).map_err(ConfigError::SerializeTomlError)?
    );

    Ok(true)
}
