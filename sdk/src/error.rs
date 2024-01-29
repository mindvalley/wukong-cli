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
    // #[error(transparent)]
    // AuthError(#[from] AuthError),
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

// #[derive(Debug, ThisError)]
// pub enum AuthError {
//     #[error("Refresh token expired: {message}")]
//     RefreshTokenExpired { message: String },
//     #[error("OpenID Connect Error: {message}")]
//     OpenIDConnectError { message: String },
//     #[error("Failed to discover OpenID Provider")]
//     OpenIDDiscoveryError,
// }

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
    #[error("API Error: Request to {domain} timed out.")]
    Timeout { domain: String },
    #[error("Failed to get data from GraphQL response.")]
    MissingResponseData,
    #[error("The selected build number is the same as the current deployed version. So there is no changelog.")]
    ChangelogComparingSameBuild,
    #[error("Unable to get pipelines.")]
    UnableToGetPipelines,
    #[error("Unable to get pipeline.")]
    UnableToGetPipeline,
    #[error("Could not find the application associated with this Git repo.\n\tEither you're not in the correct working folder for your application, or there's a misconfiguration.")]
    CIStatusApplicationNotFound,
    #[error("Application not found.")]
    ApplicationNotFound,
    #[error("Unable to determine the changelog for this build.")]
    UnableToDetermineChangelog,
    #[error("Cannot submit this deployment request, since there is another running deployment with the same arguments is running on Spinnaker.\nYou can wait a few minutes and submit the deployment again.")]
    DuplicatedDeployment,
    #[error("Namespace not found.")]
    NamespaceNotFound,
    #[error("Version not found.")]
    VersionNotFound,
    #[error("Build not found.")]
    BuildNotFound,
    #[error("Github Workflow not found.")]
    GithubWorkflowNotFound,
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
    #[error("This application is not available in k8s.")]
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
    #[error(transparent)]
    ResponseError(#[from] tonic::Status),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

// Secret Extractor Error
#[derive(Debug, ThisError)]
pub enum ExtractError {
    #[error("Bad TOML data.")]
    BadTomlData(#[from] toml::de::Error),
}
