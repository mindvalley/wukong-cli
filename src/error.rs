use std::error::Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError<'a> {
    // #[error(transparent)]
    // ReqwtError(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error("Config file not found at \"{path}\".")]
    ConfigFileNotFound {
        path: &'a str,
        #[source]
        source: ::std::io::Error,
    },
    #[error("Permission Denied: \"{path}\".")]
    ConfigFilePermissionDenied {
        path: &'a str,
        #[source]
        source: ::std::io::Error,
    },
}

impl<'a> CliError<'a> {
    pub fn suggestion(&self) -> Option<&'static str> {
        match self {
            CliError::ConfigFileNotFound { .. } => {
                Some("Run \"wukong init\" to initialise configuration.")
            }
            CliError::ConfigFilePermissionDenied { path, .. } => {
                Some("Run \"chmod +rw config.toml\" to provide read and write permissions.")
            }
            _ => None,
        }
    }
}

pub fn handle_error(error: CliError) {
    eprintln!("Error:");
    eprintln!("\t{}", error);

    // for verbose
    if let Some(source) = error.source() {
        eprintln!("\nCaused by:");
        eprintln!("\t{}", source);
    }

    if let Some(suggestion) = error.suggestion() {
        eprintln!("\nSuggestion:");
        eprintln!("\t{}", suggestion);
    }
}
