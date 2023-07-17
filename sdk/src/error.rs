use owo_colors::OwoColorize;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum WKError {
    #[error(transparent)]
    APIError(#[from] APIError),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
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
    #[error("Failed to discover OpenID Provider")]
    OpenIDDiscoveryError,
    #[error("You are un-authenticated.")]
    UnAuthenticated,
    #[error("You are un-initialised.")]
    UnInitialised,
    #[error(transparent)]
    AuthError(#[from] AuthError),
    #[error(transparent)]
    DeploymentError(#[from] DeploymentError),
    #[error(transparent)]
    ApplicationInstanceError(#[from] ApplicationInstanceError),
    #[error(transparent)]
    VaultError(#[from] VaultError),
    #[error(transparent)]
    DevConfigError(#[from] DevConfigError),
    #[error(transparent)]
    GCloudError(#[from] GCloudError),
    #[error(transparent)]
    ExtractError(#[from] ExtractError),
    #[error("Operation timeout")]
    Timeout,
}

#[derive(Debug, ThisError)]
pub enum AuthError {
    #[error("Refresh token expired: {message}")]
    RefreshTokenExpired { message: String },
    #[error("OpenID Connect Error: {message}")]
    OpenIDConnectError { message: String },
    #[error("Failed to discover OpenID Provider")]
    OpenIDDiscoveryError,
}

#[derive(Debug, ThisError)]
pub enum APIError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("API Response Error: {message}")]
    ResponseError { code: String, message: String },
    #[error("API Error: You are un-authenticated.")]
    UnAuthenticated,
    #[error("API Error: You are un-authorized.")]
    UnAuthorized,
    #[error("The selected build number is the same as the current deployed version. So there is no changelog.")]
    ChangelogComparingSameBuild,
    #[error("API Error: Request to {domain} timed out.")]
    Timeout { domain: String },
    // Error during refreshing tokens
    #[error(transparent)]
    AuthError(#[from] AuthError),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error("Failed to get data from GraphQL response.")]
    MissingResponseData,
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

#[derive(Debug, ThisError)]
pub enum ApplicationInstanceError {
    #[error("There is no k8s configuration associated with your application.")]
    NamespaceNotAvailable,
    #[error("This version has no associated k8s cluster configuration.")]
    VersionNotAvailable { version: String },
    #[error("This application is not available  in k8s.")]
    ApplicationNotFound,
}

// Vault Service Error
#[derive(Debug, ThisError)]
pub enum VaultError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("API Response Error: {message}")]
    ResponseError { code: String, message: String },
    #[error("Invalid credentials. Please try again.")]
    AuthenticationFailed,
    #[error("You are un-initialised.")]
    UnInitialised,
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error("Secret not found.")]
    SecretNotFound,
    #[error("API token not found.")]
    ApiTokenNotFound,
    #[error("Invalid API token.")]
    ApiTokenInvalid,
    #[error("Permission denied.")]
    PermissionDenied,
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
}

// GCloud Service Error
#[derive(Debug, ThisError)]
pub enum GCloudError {
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    GoogleLogging2Error(#[from] google_logging2::Error),
}

// Secret Extractor Error
#[derive(Debug, ThisError)]
pub enum ExtractError {
    #[error("Bad TOML data.")]
    BadTomlData(#[from] toml::de::Error),
}

impl WKError {
    /// Try to second-guess what the user was trying to do, depending on what
    /// went wrong.
    pub fn suggestion(&self) -> Option<String> {
        match self {
            WKError::UnAuthenticated => Some(String::from(
                "Your access token is invalid. Run \"wukong login\" to authenticate with your okta account.",
            )),
            WKError::UnInitialised => Some(String::from(
                "Run \"wukong init\" to initialise Wukong's configuration before running other commands.",
            )),
            WKError::ConfigError(error) => match error {
                ConfigError::NotFound { .. } => Some(String::from(
                    "Run \"wukong init\" to initialise Wukong's configuration.",
                )),
                ConfigError::PermissionDenied { path, .. } => Some(format!(
                    "Run \"chmod +rw {path}\" to provide read and write permissions."
                )),
                ConfigError::BadTomlData(_) => Some(
                    "Check if your config.toml file is in valid TOML format.\nThis usually happen when the config file is accidentally modified or there is a breaking change to the cli config in the new version.\nYou may want to run \"wukong init\" to re-initialise configuration again.".to_string()
                ),
                _ => None,
            },
            WKError::APIError(error) => match error {
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
                APIError::UnAuthorized => Some(
                    "Your token might be invalid/expired. Run \"wukong login\" to authenticate with your okta account.".to_string()
                ),
                _ => None,
            },
             WKError::ApplicationInstanceError(error) => match error {
                ApplicationInstanceError::VersionNotAvailable { version, .. } => Some(
                    format!(
                    "You can try to check the following:
    * Whether your application is supporting this {version} version. 
    * Contact Wukong dev team to check if there is any k8s configuration enabled for this version. "
                )),
                ApplicationInstanceError::NamespaceNotAvailable { .. } => Some(
                    "You may want to contact Wukong dev team to check if there is any k8s configuration for your application.".to_string()
                ),
                _ => None,
            },
            WKError::DevConfigError(error) => match error {
                DevConfigError::ConfigSecretNotFound=> Some(
                    "Run \"wukong config dev pull\" to pull the latest dev config.\n".to_string()
               ),
                DevConfigError::InvalidSecretPath { config_path, annotation } => Some(format!(
                    "Please check the {annotation} in the config file: {config_path}"
                )),
                _ => None,
            },
            WKError::AuthError(AuthError::RefreshTokenExpired { .. }) => Some("Your refresh token is expired. Run \"wukong login\" to authenticate again.".to_string()),
            WKError::VaultError(VaultError::ConfigError(error)) => match error {
                    ConfigError::NotFound { .. } => Some(String::from(
                        "Run \"wukong init\" to initialise Wukong's configuration.",
                    )),
                    ConfigError::PermissionDenied { path, .. } => Some(format!(
                        "Run \"chmod +rw {path}\" to provide read and write permissions."
                    )),
                    ConfigError::BadTomlData(_) => Some(
                        "Check if your config.toml file is in valid TOML format.\nThis usually happen when the config file is accidentally modified or there is a breaking change to the cli config in the new version.\nYou may want to run \"wukong init\" to re-initialise configuration again.".to_string()
                    ),
                    _ => None,
                },
            _ => None,
        }
    }
}
