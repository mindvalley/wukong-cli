use crate::error::CliError;
// use anyhow::{Context, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
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
        dirs_next::home_dir().map(|mut path| {
            path.extend(&[".config", "wukong", "config.toml"]);
            path.to_str().unwrap().to_string()
        })
    };
}

/// The Wukong CLI configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub core: CoreConfig,
    pub log: LogConfig,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CoreConfig {
    /// The current application name
    pub application: String,
    pub collect_telemetry: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LogConfig {
    pub enable: bool,
    pub log_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut home_dir = dirs_next::home_dir().unwrap();
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
        // if !config_file_path.is_file() {
        //     return Err(CliError::ConfigFileNotFound { path })?;
        // }

        // let content =
        //     std::fs::read_to_string(config_file_path.to_str().unwrap()).with_context(|| {
        //         format!(
        //             "Could not read file `{}`",
        //             config_file_path.to_str().unwrap()
        //         )
        //     })?;

        // let config = toml::from_str(&content).with_context(|| {
        //     format!(
        //         "`{:?}` could not be deserialized as Config TOML format",
        //         &content
        //     )
        // })?;

        let content =
            std::fs::read_to_string(config_file_path.to_str().unwrap()).map_err(|err| match err
                .kind()
            {
                io::ErrorKind::NotFound => CliError::ConfigFileNotFound { path, source: err },
                io::ErrorKind::PermissionDenied => {
                    CliError::ConfigFilePermissionDenied { path, source: err }
                }
                io::ErrorKind::ConnectionRefused => todo!(),
                io::ErrorKind::ConnectionReset => todo!(),
                io::ErrorKind::ConnectionAborted => todo!(),
                io::ErrorKind::NotConnected => todo!(),
                io::ErrorKind::AddrInUse => todo!(),
                io::ErrorKind::AddrNotAvailable => todo!(),
                io::ErrorKind::BrokenPipe => todo!(),
                io::ErrorKind::AlreadyExists => todo!(),
                io::ErrorKind::WouldBlock => todo!(),
                io::ErrorKind::InvalidInput => todo!(),
                io::ErrorKind::InvalidData => todo!(),
                io::ErrorKind::TimedOut => todo!(),
                io::ErrorKind::WriteZero => todo!(),
                io::ErrorKind::Interrupted => todo!(),
                io::ErrorKind::Unsupported => todo!(),
                io::ErrorKind::UnexpectedEof => todo!(),
                io::ErrorKind::OutOfMemory => todo!(),
                io::ErrorKind::Other => todo!(),
                _ => todo!(),
            })?;

        let config = toml::from_str(&content).unwrap();

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
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;

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
        assert!(Config::load(path).is_err());
    }
}
