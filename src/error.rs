use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError<'a> {
    // #[error(transparent)]
    // ReqwtError(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    ConfigFileError(ConfigFileError<'a>),
}

#[derive(Debug, Error)]
pub enum ConfigFileError<'a> {
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
}

impl<'a> CliError<'a> {
    pub fn suggestion(&self) -> Option<String> {
        match self {
            CliError::ConfigFileError(error) => match error {
                ConfigFileError::NotFound { .. } => Some(String::from(
                    "Run \"wukong init\" to initialise configuration.",
                )),
                ConfigFileError::PermissionDenied { path, .. } => Some(format!(
                    "Run \"chmod +rw {path}\" to provide read and write permissions."
                )),
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
