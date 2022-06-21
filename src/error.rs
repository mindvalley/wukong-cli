use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError<'a> {
    // #[error(transparent)]
    // ReqwtError(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    ConfigError(ConfigError<'a>),
}

#[derive(Debug, Error)]
pub enum ConfigError<'a> {
    #[error("Config file not found at \"{path}\".")]
    NotFound {
        path: &'a str,
        #[source]
        source: ::std::io::Error,
    },
    #[error("Permission denied: \"{path}\".")]
    PermissionDenied {
        path: &'a str,
        #[source]
        source: ::std::io::Error,
    },
    #[error("Bad TOML data.")]
    BadTomlData(#[source] toml::de::Error),
    #[error("Failed to serialize configuration data into TOML")]
    SerializeTomlError(#[source] toml::ser::Error),
}

impl<'a> CliError<'a> {
    pub fn suggestion(&self) -> Option<String> {
        match self {
            CliError::ConfigError(error) => match error {
                ConfigError::NotFound { .. } => Some(String::from(
                    "Run \"wukong init\" to initialise configuration.",
                )),
                ConfigError::PermissionDenied { path, .. } => Some(format!(
                    "Run \"chmod +rw {path}\" to provide read and write permissions."
                )),
                ConfigError::BadTomlData(_) => Some(String::from(
                    "Check if your config.toml file is in valid TOML format.",
                )),
                _ => None,
            },
            _ => None,
        }
    }
}

pub fn handle_error(error: CliError) {
    eprintln!("Error:");
    eprintln!("\t{}", error);

    //TODO: for --verbose only
    if let Some(source) = error.source() {
        eprintln!("\nCaused by:");
        eprintln!("\t{}", source);
    }

    if let Some(suggestion) = error.suggestion() {
        eprintln!("\nSuggestion:");
        eprintln!("\t{}", suggestion);
    }
}
