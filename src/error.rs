use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum CliError<'a> {
    #[error(transparent)]
    APIError(#[from] APIError),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    ConfigError(ConfigError<'a>),
    #[error("Failed to discover OpenID Provider")]
    OpenIDDiscoveryError,
    #[error("You are un-authenticated.")]
    UnAuthenticated,
    #[error("You are un-initialised.")]
    UnInitialised,
}

#[derive(Debug, ThisError)]
pub enum APIError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Response Error: {message}")]
    ResponseError { code: String, message: String },
    #[error("You are un-authenticated.")]
    UnAuthenticated,
    #[error("Could not find the application associated with this Git repo.\n\tEither you're not in the correct working folder for your application, or there's a misconfiguration.")]
    InvalidRepoUrl,
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
    /// Try to second-guess what the user was trying to do, depending on what
    /// went wrong.
    pub fn suggestion(&self) -> Option<String> {
        match self {
            CliError::UnAuthenticated => Some(String::from(
                "Your access token is invalid. Run \"wukong login\" to authenticate with your okta account.",
            )),
            CliError::UnInitialised => Some(String::from(
                "Run \"wukong init\" to initialise Wukong's configuration.",
            )),
            CliError::ConfigError(error) => match error {
                ConfigError::NotFound { .. } => Some(String::from(
                    "Run \"wukong init\" to initialise configuration.",
                )),
                ConfigError::PermissionDenied { path, .. } => Some(format!(
                    "Run \"chmod +rw {path}\" to provide read and write permissions."
                )),
                ConfigError::BadTomlData(_) => Some(String::from(
                    "Check if your config.toml file is in valid TOML format. You may want to remove the config.toml file and run \"wukong init\" to re-initialise configuration again.",
                )),
                _ => None,
            },
            CliError::APIError(error) => match error {
                APIError::ResponseError { code, .. } if code == "unable_to_get_pipeline" => Some(
                    String::from("Please check your pipeline's name. It could be invalid."),
                ),
                APIError::ResponseError { code, .. } if code == "unable_to_get_pipelines" => Some(
                    String::from("Please check your application's name. It could be invalid."),
                ),
                APIError::ResponseError { code, .. } if code == "application_not_found" => Some(
                    String::from("Please check your repo url. It's unrecognized by wukong."),
                ),
                APIError::UnAuthenticated => Some(
                    "Run \"wukong login\" to authenticate with your okta account.".to_string()
                ),
                APIError::InvalidRepoUrl => Some(
                    r#"You can follow these steps to remedy this error:  
            1. Confirm that you're in the correct working folder.
            2. If you're not, consider moving to the right location and run 'wukong pipeline ci-status' command again.
        If none of the above steps work for you, please contact the following people on Slack for assistance :slack: @alex.tuan / :slack: @jk-gan / :slack: @Fadhil"#.to_string()
                ),
                _ => None,
            },
            _ => None,
        }
    }
}
