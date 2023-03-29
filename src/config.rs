use crate::error::{CliError, ConfigError};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::{self, Write},
    path::Path,
};

#[cfg(not(feature = "prod"))]
static WUKONG_API_URL: &str = "http://localhost:4000/api";
#[cfg(not(feature = "prod"))]
static OKTA_CLIENT_ID: &str = "0oakfxaegyAV5JDD5357";

#[cfg(all(feature = "prod"))]
static WUKONG_API_URL: &str = "https://wukong-api-proxy.mindvalley.dev/api";
#[cfg(all(feature = "prod"))]
static OKTA_CLIENT_ID: &str = "0oakfxaegyAV5JDD5357";

/// The default path to the CLI configuration file.
///
/// This is a [Lazy] of `Option<String>`, the value of which is
///
/// > `~/.config/wukong/config.yml`
///
/// It will only be `None` if it is unable to identify the user's home
/// directory, which should not happen under typical OS environments.
///
/// [Lazy]: https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html
pub static CONFIG_FILE: Lazy<Option<String>> = Lazy::new(|| {
    #[cfg(all(feature = "prod"))]
    return dirs::home_dir().map(|mut path| {
        path.extend([".config", "wukong", "config.toml"]);
        path.to_str().unwrap().to_string()
    });

    #[cfg(not(feature = "prod"))]
    {
        match std::env::var("WUKONG_DEV_CONFIG_FILE") {
            Ok(config) => {
                // TODO: we should check whether the config file valid
                Some(config)
            }
            Err(_) => dirs::home_dir().map(|mut path| {
                path.extend([".config", "wukong", "dev", "config.toml"]);
                path.to_str().unwrap().to_string()
            }),
        }
    }
});

/// The Wukong CLI configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Config {
    pub core: CoreConfig,
    pub auth: Option<AuthConfig>,
    pub vault: Option<VaultConfig>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CoreConfig {
    /// The current application name
    pub application: String,
    pub wukong_api_url: String,
    pub okta_client_id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct VaultConfig {
    pub api_token: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ConfigWithPath {
    pub config: Config,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct AuthConfig {
    pub account: String,
    pub subject: String,
    pub id_token: String,
    pub access_token: String,
    pub expiry_time: String,
    pub refresh_token: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut home_dir = dirs::home_dir().unwrap();
        home_dir.extend([".config", "wukong", "log*"]);

        Self {
            core: CoreConfig {
                application: "".to_string(),
                wukong_api_url: WUKONG_API_URL.to_string(),
                okta_client_id: OKTA_CLIENT_ID.to_string(),
            },
            auth: None,
            vault: None,
        }
    }
}

impl Config {
    /// Load a configuration from file.
    ///
    /// # Errors
    ///
    /// This function may return typical file I/O errors.
    pub fn load(path: &'static str) -> Result<Self, CliError> {
        let config_file_path = Path::new(path);

        let content = std::fs::read_to_string(
            config_file_path
                .to_str()
                .expect("The config file path is not valid."),
        )
        .map_err(|err| match err.kind() {
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
        let serialized = toml::to_string(self).map_err(ConfigError::SerializeTomlError)?;

        if let Some(outdir) = config_file_path.parent() {
            create_dir_all(outdir)?;
        }
        let mut file = File::create(path)?;
        file.write_all(&serialized.into_bytes())?;

        Ok(())
    }

    pub fn get_config_with_path() -> Result<ConfigWithPath, CliError> {
        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let config = Config::load(config_file)?;

        let config_with_path = ConfigWithPath {
            config,
            path: config_file.to_string(),
        };

        Ok(config_with_path)
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
