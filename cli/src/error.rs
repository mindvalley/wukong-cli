use owo_colors::OwoColorize;
use thiserror::Error as ThisError;
use wukong_sdk::error::{APIError, ExtractError, WKError};

#[derive(Debug, ThisError)]
pub enum WKCliError {
    #[error(transparent)]
    WKSdkError(#[from] WKError),
    #[error(transparent)]
    WKSecretExtractError(#[from] ExtractError),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    AuthError(#[from] AuthError),
    #[error("You are un-authenticated.")]
    UnAuthenticated,
    #[error("You are un-initialised.")]
    UnInitialised,
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Error parsing \"{value}\"")]
    ChronoParseError {
        value: String,
        #[source]
        source: chrono::ParseError,
    },
    #[error("Invalid input: \"{value}\"")]
    InvalidInput { value: String },
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    ApplicationConfigError(#[from] ApplicationConfigError),
    #[error(transparent)]
    DeploymentError(#[from] DeploymentError),
    #[error(transparent)]
    PipelineError(#[from] PipelineError),
    #[error(transparent)]
    DevConfigError(#[from] DevConfigError),
    #[error(transparent)]
    ApplicationInstanceError(#[from] ApplicationInstanceError),
    #[error("Operation timeout")]
    Timeout,
    #[error("Unable to parse YML file")]
    UnableToParseYmlFile,
    #[error(transparent)]
    InquireError(#[from] inquire::error::InquireError),
}

#[derive(Debug, ThisError)]
pub enum ApplicationInstanceError {
    #[error("There is no k8s configuration associated with your application.")]
    NamespaceNotAvailable,
    #[error("This version has no associated k8s cluster configuration.")]
    VersionNotAvailable { version: String },
    #[error("This application is not available in k8s.")]
    ApplicationNotFound,
}

#[derive(Debug, ThisError)]
pub enum AuthError {
    #[error("Refresh token expired: {message}")]
    OktaRefreshTokenExpired { message: String },
    #[error("OpenID Connect Error: {message}")]
    OpenIDConnectError { message: String },
    #[error("Failed to discover OpenID Provider")]
    OpenIDDiscoveryError,
    #[error("Vault secret not found.")]
    VaultSecretNotFound,
    #[error("Vault permission denied.")]
    VaultPermissionDenied,
    #[error("Invalid credentials. Please try again.")]
    VaultAuthenticationFailed,
    #[error("Vault API Response Error: {message}")]
    VaultResponseError { code: String, message: String },
}

#[derive(Debug, ThisError)]
pub enum DevConfigError {
    #[error("No config files found!")]
    ConfigNotFound,
    #[error("No dev secret config files found!")]
    ConfigSecretNotFound,
    // Invalid secret path in the annotation in config file:
    #[error("Invalid secret path in the config file")]
    InvalidSecretPath {
        config_path: String,
        annotation: String,
    },
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
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
}

#[derive(Debug, ThisError)]
pub enum ApplicationConfigError {
    #[error("Application Config file not found at \"{path}\".")]
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
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
}

#[derive(Debug, ThisError)]
pub enum PipelineError {
    #[error("Could not find the application associated with this Git repo.\n\tEither you're not in the correct working folder for your application, or there's a misconfiguration.")]
    CIStatusApplicationNotFound,
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

impl WKCliError {
    /// Try to second-guess what the user was trying to do, depending on what
    /// went wrong.
    pub fn suggestion(&self) -> Option<String> {
        match self {
            WKCliError::WKSdkError(WKError::APIError(error)) => {
                match error {
                    APIError::UnableToGetPipeline { .. } => Some(
                        String::from("Please check your pipeline's name. It could be invalid."),
                    ),
                        APIError::UnableToGetPipelines { .. } => Some(
                        String::from("Please check your application's name. It could be invalid."),
                    ),
                    APIError::ApplicationNotFound { .. } => Some(
                        String::from("Please check your application config in \".wukong.toml\" file or your repo url. It's unrecognized by wukong."),
                    ),
                    APIError::NamespaceNotFound { .. } => Some(
                        String::from("Please check your namespace value. It could be invalid."),
                    ),
                    APIError::VersionNotFound { .. } => Some(
                        String::from("Please check your version value. It could be invalid."),
                    ),
                    APIError::CIStatusApplicationNotFound => Some(format!(
    r#"You can follow these steps to remedy this error:
1. Confirm that you're in the correct working folder.
2. If you're not, consider moving to the right location and run {} command again.
If none of the above steps work for you, please contact the following people on Slack for assistance: @alex.tuan / @jk-gan / @Fadhil / @Fauzaan"#,
    "wukong pipeline ci-status".yellow()
                    )),
                    APIError::UnAuthenticated => Some(format!(
                        "Run {} to authenticate with your okta account.", "wukong login".yellow())
                    ),
                    APIError::UnAuthorized => Some(format!(
                        "Your token might be invalid/expired. Run {} to authenticate with your okta account.", "wukong login".yellow())
                    ),
                    _ => None,
                }
            },
            WKCliError::UnAuthenticated => Some(
                format!("Your access token is invalid. Run {} to authenticate with your okta account.", "wukong login".yellow()),
            ),
            WKCliError::UnInitialised => Some(format!(
                "Run {} to initialise Wukong's configuration before running other commands.", "wukong init".yellow()
            )),
            WKCliError::ConfigError(error) => match error {
                ConfigError::NotFound { .. } => Some(format!(
                    "Run {} to initialise Wukong's configuration.", "wukong init".yellow()
                )),
                ConfigError::PermissionDenied { path, .. } => Some(format!(
                    "Run \"chmod +rw {path}\" to provide read and write permissions."
                )),
                ConfigError::BadTomlData(_) => Some(
                    format!("Check if your `config.toml` file is in valid TOML format.\nThis usually happens when the config file has accidentally been modified or there is a breaking change to the cli config in the new version.\nYou may want to run {} to re-initialise configuration again.", "wukong init".yellow())
                ),
                _ => None,
            },
            WKCliError::ApplicationConfigError(error) => match error {
                ApplicationConfigError::NotFound { .. } => Some(format!(
        r#"Wukong command only works in the directory contains `.wukong.toml` file. Check if you are in the correct directory.
If so, run {} to initialise the application configuration."#, "wukong application init".yellow())
                ),
                ApplicationConfigError::PermissionDenied { path, .. } => Some(format!(
                    "Run \"chmod +rw {path}\" to provide read and write permissions."
                )),
                ApplicationConfigError::BadTomlData(_) => Some(
                    format!("Check if the `.wukong.toml` file is in valid TOML format.\nThis usually happen when the config file is accidentally modified or there is a breaking change to the application config in the new version.\nYou may want to run {} to re-initialise configuration again.", "wukong application init".yellow())
                ),
                _ => None,
            },
            WKCliError::DevConfigError(error) => match error {
                DevConfigError::ConfigSecretNotFound=> Some(
                    format!("Run {} to pull the latest dev config.\n", "wukong dev config pull".yellow())
               ),
                DevConfigError::InvalidSecretPath { config_path, annotation } => Some(format!(
                    "Please check the {annotation} in the config file: {config_path}"
                )),
                _ => None,
            },
            WKCliError::AuthError(AuthError::OktaRefreshTokenExpired { .. }) => Some(format!("Your refresh token is expired. Run {} to authenticate again.", "wukong login".yellow())),
            _ => None,
        }
    }
}
