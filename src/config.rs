use crate::error::{CliError, ConfigError};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::{self, Write},
    path::Path,
};

lazy_static! {
    /// The default path to the CLI configuration file.
    ///
    /// This is a [lazy_static] of `Option<String>`, the value of which is
    ///
    /// > `~/.config/wukong/config.yml`
    ///
    /// It will only be `None` if it is unable to identify the user's home
    /// directory, which should not happen under typical OS environments.
    ///
    /// [lazy_static]: https://docs.rs/lazy_static
    pub static ref CONFIG_FILE: Option<String> = {
        dirs::home_dir().map(|mut path| {
            path.extend(&[".config", "wukong", "config.toml"]);
            path.to_str().unwrap().to_string()
        })
    };
}

/// The Wukong CLI configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Config {
    pub core: CoreConfig,
    pub log: LogConfig,
    pub auth: Option<AuthConfig>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CoreConfig {
    /// The current application name
    pub application: String,
    pub collect_telemetry: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct LogConfig {
    pub enable: bool,
    pub log_dir: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct AuthConfig {
    pub account: String,
    pub id_token: String,
    pub access_token: String,
    pub expiry_time: String,
    pub refresh_token: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut home_dir = dirs::home_dir().unwrap();
        home_dir.extend(&[".config", "wukong", "log*"]);

        Self {
            core: CoreConfig {
                application: "".to_string(),
                collect_telemetry: false,
            },
            log: LogConfig {
                enable: true,
                log_dir: home_dir.to_str().unwrap().to_string(),
            },
            auth: None,
        }
    }
}

impl Config {
    /// Load a configuration from file.
    ///
    /// # Errors
    ///
    /// This function may return typical file I/O errors.
    pub fn load(path: &str) -> Result<Self, CliError> {
        let config_file_path = Path::new(path);

        let content =
            std::fs::read_to_string(config_file_path.to_str().unwrap()).map_err(|err| match err
                .kind()
            {
                io::ErrorKind::NotFound => {
                    CliError::ConfigError(ConfigError::NotFound { path, source: err })
                }
                io::ErrorKind::PermissionDenied => {
                    CliError::ConfigError(ConfigError::PermissionDenied { path, source: err })
                }
                _ => CliError::Io(err),
            })?;

        let config = toml::from_str(&content)
            .map_err(|err| CliError::ConfigError(ConfigError::BadTomlData(err)))?;

        Ok(config)
    }

    /// Save a configuration to file.
    ///
    /// If the file's directory does not exist, it will be created. If the file
    /// already exists, it will be overwritten.
    ///
    /// # Errors
    ///
    /// This function may return typical file I/O errors.
    pub fn save(&self, path: &str) -> Result<(), CliError> {
        let config_file_path = Path::new(path);
        let serialized = toml::to_string(self)
            .map_err(|err| CliError::ConfigError(ConfigError::SerializeTomlError(err)))?;

        if let Some(outdir) = config_file_path.parent() {
            create_dir_all(outdir)?;
        }
        let mut file = File::create(path)?;
        file.write_all(&serialized.into_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn save_and_load_config_file() {
        let path = "./config.toml";
        let config = Config::default();

        // 1. save the config file
        config.save(path).unwrap();

        // 2. load the config file
        let saved_config = Config::load(path).unwrap();

        assert_eq!(saved_config.core.application, config.core.application);
        assert_eq!(
            saved_config.core.collect_telemetry,
            config.core.collect_telemetry
        );
        assert_eq!(saved_config.log.enable, config.log.enable);
        assert_eq!(saved_config.log.log_dir, config.log.log_dir);

        // remove the config file
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn load_non_exist_file() {
        let path = "./non/exist/path";
        let result = Config::load(path);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(CliError::ConfigError(ConfigError::NotFound { .. }))
        ));
    }
}
