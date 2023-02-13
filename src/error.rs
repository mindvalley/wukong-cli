use owo_colors::OwoColorize;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum CliError {
    #[error(transparent)]
    APIError(#[from] APIError),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error("Failed to discover OpenID Provider")]
    OpenIDDiscoveryError,
    #[error("You are un-authenticated.")]
    UnAuthenticated,
    #[error("You are un-initialised.")]
    UnInitialised,
    #[error("{message}")]
    AuthError { message: &'static str },
    #[error(transparent)]
    DeploymentError(#[from] DeploymentError),
}

#[derive(Debug, ThisError)]
pub enum APIError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("API Response Error: {message}")]
    ResponseError { code: String, message: String },
    #[error("You are un-authenticated.")]
    UnAuthenticated,
    #[error("The selected build number is the same as the current deployed version. So there is no changelog.")]
    ChangelogComparingSameBuild,
}

#[derive(Debug, ThisError)]
pub enum ConfigError {
    #[error("Config file not found at \"{path}\".")]
    NotFound {
        path: &'static str,
        #[source]
        source: ::std::io::Error,
    },
    #[error("Permission denied: \"{path}\".")]
    PermissionDenied {
        path: &'static str,
        #[source]
        source: ::std::io::Error,
    },
    #[error("Bad TOML data.")]
    BadTomlData(#[source] toml::de::Error),
    #[error("Failed to serialize configuration data into TOML.")]
    SerializeTomlError(#[source] toml::ser::Error),
}

#[derive(Debug, ThisError)]
pub enum DeploymentError {
    #[error("\"{namespace}\" namespace is not available in \"{application}\" application.")]
    NamespaceNotAvailable {
        namespace: String,
        application: String,
    },
    #[error("\"{version}\" version is not available in \"{application}\" application under \"{namespace}\" namespace.")]
    VersionNotAvailable {
        namespace: String,
        version: String,
        application: String,
    },
}

impl CliError {
    /// Try to second-guess what the user was trying to do, depending on what
    /// went wrong.
    pub fn suggestion(&self) -> Option<String> {
        match self {
            CliError::UnAuthenticated => Some(String::from(
                "Your access token is invalid. Run \"wukong login\" to authenticate with your okta account.",
            )),
            CliError::UnInitialised => Some(String::from(
                "Run \"wukong init\" to initialise Wukong's configuration before running other commands.",
            )),
            CliError::ConfigError(error) => match error {
                ConfigError::NotFound { .. } => Some(String::from(
                    "Run \"wukong init\" to initialise configuration.",
                )),
                ConfigError::PermissionDenied { path, .. } => Some(format!(
                    "Run \"chmod +rw {path}\" to provide read and write permissions."
                )),
                ConfigError::BadTomlData(_) => Some(format!(
                    "Check if your config.toml file is in valid TOML format.\nThis usually happen when the config file is accidentally modified or there is a breaking change to the cli config in the new version.\nYou may want to run \"wukong init\" to re-initialise configuration again."
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
                APIError::ResponseError { code, .. } if code == "ci_status_application_not_found" => Some(format!(
        r#"You can follow these steps to remedy this error:  
    1. Confirm that you're in the correct working folder.
    2. If you're not, consider moving to the right location and run {} command again.
If none of the above steps work for you, please contact the following people on Slack for assistance: @alex.tuan / @jk-gan / @Fadhil"#,
        "wukong pipeline ci-status".yellow()
                )),
                APIError::UnAuthenticated => Some(
                    "Run \"wukong login\" to authenticate with your okta account.".to_string()
                ),
                _ => None,
            },
            _ => None,
        }
    }
}
