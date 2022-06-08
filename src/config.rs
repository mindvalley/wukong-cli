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
    /// > `~/.config/wukong/cli/config.yml`
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
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub core: CoreConfig,
    pub log: LogConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoreConfig {
    /// The current application name
    pub application: String,
    pub collect_telemetry: bool,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub fn load(config_file: &str) -> Result<Self, std::io::Error> {
        let config_file_path = Path::new(config_file);
        let content = std::fs::read_to_string(config_file_path.to_str().unwrap())?;
        let config = toml::from_str(&content)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;
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
    pub fn save(&self, config_file: &str) -> Result<(), std::io::Error> {
        let config_file_path = Path::new(config_file);
        let serialized = toml::to_string(self)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;

        if let Some(outdir) = config_file_path.parent() {
            create_dir_all(outdir)?;
        }
        let mut file = File::create(config_file)?;
        file.write_all(&serialized.into_bytes())?;

        Ok(())
    }
}
