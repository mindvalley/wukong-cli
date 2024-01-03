use crate::{auth::google_cloud::GoogleCloudConfig, error::ConfigError};
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

#[cfg(feature = "prod")]
static WUKONG_API_URL: &str = "https://wukong-api-proxy.mindvalley.dev/api";
#[cfg(feature = "prod")]
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
    #[cfg(feature = "prod")]
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
    pub update_check: Option<UpdateCheck>,
    pub google_cloud: Option<GoogleCloudConfig>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub enum ApiChannel {
    Canary,
    #[default]
    Stable,
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
    pub expiry_time: String,
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

// ReleaseInfo stores information about a release
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct UpdateCheck {
    pub last_update_checked_at: String,
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
            update_check: None,
            google_cloud: None,
        }
    }
}

impl Config {
    /// Load a configuration from default path.
    ///
    /// # Errors
    ///
    /// This function may return typical file I/O errors.
    pub fn load_from_default_path() -> Result<Self, ConfigError> {
        Self::load_from_path(
            CONFIG_FILE
                .as_ref()
                .expect("Unable to identify user's home directory"),
        )
    }

    /// Load a configuration from file.
    ///
    /// # Errors
    ///
    /// This function may return typical file I/O errors.
    pub fn load_from_path(path: &'static str) -> Result<Self, ConfigError> {
        let config_file_path = Path::new(path);

        let content = std::fs::read_to_string(
            config_file_path
                .to_str()
                .expect("The config file path is not valid."),
        )
        .map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => ConfigError::NotFound { path, source: err },
            io::ErrorKind::PermissionDenied => ConfigError::PermissionDenied { path, source: err },
            _ => err.into(),
        })?;

        let config = toml::from_str(&content).map_err(ConfigError::BadTomlData)?;

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
    pub fn save_to_path(&self, path: &str) -> Result<(), ConfigError> {
        let config_file_path = Path::new(path);
        let serialized = toml::to_string(self).map_err(ConfigError::SerializeTomlError)?;

        if let Some(outdir) = config_file_path.parent() {
            create_dir_all(outdir)?;
        }
        let mut file = File::create(path)?;
        file.write_all(&serialized.into_bytes())?;

        Ok(())
    }

    pub fn save_to_default_path(&self) -> Result<(), ConfigError> {
        self.save_to_path(
            CONFIG_FILE
                .as_ref()
                .expect("Unable to identify user's home directory"),
        )
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
        config.save_to_path(path).unwrap();

        // 2. load the config file
        let saved_config = Config::load_from_path(path).unwrap();

        assert_eq!(saved_config.core.application, config.core.application);

        // remove the config file
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn load_non_exist_file() {
        let path = "./non/exist/path";
        let result = Config::load_from_path(path);

        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::NotFound { .. })));
    }
}
