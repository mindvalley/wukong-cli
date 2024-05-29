pub mod execute;
pub mod list;
pub mod rollback;
pub mod status;

use std::fmt::Display;

use execute::handle_execute;
use list::handle_list;
use rollback::handle_rollback;

use crate::WKCliError;
use clap::{Args, Subcommand, ValueEnum};

use self::status::handle_status;

use super::Context;

#[derive(Debug, Args)]
pub struct Deployment {
    #[command(subcommand)]
    pub subcommand: DeploymentSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DeploymentSubcommand {
    /// List the current available deployment pipelines of an application
    List,
    /// Start the deployment pipeline
    Execute {
        /// The namespace to deploy to.
        #[arg(long, value_enum)]
        namespace: Option<DeploymentNamespace>,
        /// The version that the deployment will perform
        /// against.
        #[arg(long, value_enum)]
        version: Option<DeploymentVersion>,
        /// The build artifact that the deployment will use.
        #[arg(long)]
        artifact: Option<String>,
    },
    /// Rollback the deployment pipeline
    Rollback {
        /// The namespace to deploy to.
        #[arg(long, value_enum)]
        namespace: Option<DeploymentNamespace>,
        /// The version that the deployment will perform
        /// against.
        #[arg(long, value_enum)]
        version: Option<DeploymentVersion>,
    },
    /// Get the status of the latest deployment
    Status {
        /// The version of the deployment
        #[arg(long, value_enum, default_value_t = DeploymentVersion::Green)]
        version: DeploymentVersion,
    },
}

#[derive(Debug, ValueEnum, Clone)]
pub enum DeploymentVersion {
    Blue,
    Green,
}

impl Display for DeploymentVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentVersion::Blue => write!(f, "{}", "Blue".to_string()),
            DeploymentVersion::Green=> write!(f, "{}", "Green".to_string())
        }
    }
}

#[derive(Debug, ValueEnum, Clone)]
pub enum DeploymentNamespace {
    Prod,
    Staging,
}

impl Display for DeploymentNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentNamespace::Prod => write!(f, "{}", "Prod".to_string()),
            DeploymentNamespace::Staging => write!(f, "{}", "Staging".to_string())
        }
    }
}

impl Deployment {
    pub async fn handle_command(&self, context: Context) -> Result<bool, WKCliError> {
        match &self.subcommand {
            DeploymentSubcommand::List => handle_list(context).await,
            DeploymentSubcommand::Execute {
                namespace,
                version,
                artifact,
            } => handle_execute(context, namespace, version, artifact).await,
            DeploymentSubcommand::Rollback { namespace, version } => {
                handle_rollback(context, namespace, version).await
            }
            DeploymentSubcommand::Status { version } => handle_status(context, version).await,
        }
    }
}
