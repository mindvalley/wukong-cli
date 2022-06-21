use std::error::Error;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum CliError<'a> {
    #[error(transparent)]
    ReqwtError(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    ConfigError(ConfigError<'a>),
}

#[derive(Debug, ThisError)]
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
    #[error("Failed to serialize configuration data into TOML.")]
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
    use ansi_term::Colour;

    match error {
        CliError::Io(ref io_error) if io_error.kind() == ::std::io::ErrorKind::BrokenPipe => {
            ::std::process::exit(0);
        }
        _ => {
            // writeln!(output, "{}: {}", Red.paint("[bat error]"), error).ok();
            eprintln!("{}:", Colour::Red.paint("Error"));
            eprintln!("\t{}", error);

            //TODO: for --verbose only
            if let Some(source) = error.source() {
                eprintln!("\n{}:", Colour::Fixed(245).paint("Caused by"));
                eprintln!("\t{}", source);
            }

            if let Some(suggestion) = error.suggestion() {
                eprintln!("\n{}:", Colour::Cyan.paint("Suggestion"));
                eprintln!("\t{}", suggestion);
            }
        }
    };
}
