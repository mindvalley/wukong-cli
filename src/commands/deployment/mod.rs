pub mod execute;
pub mod list;

use execute::handle_execute;
use list::handle_list;

use crate::{CliError, GlobalContext};
use clap::{Args, Subcommand, ValueEnum};

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
        artifact: Option<i64>,
    },
}

#[derive(Debug, ValueEnum, Clone)]
pub enum DeploymentVersion {
    Blue,
    Green,
}

impl ToString for DeploymentVersion {
    fn to_string(&self) -> String {
        match self {
            DeploymentVersion::Blue => "Blue".to_string(),
            DeploymentVersion::Green => "Green".to_string(),
        }
    }
}

#[derive(Debug, ValueEnum, Clone)]
pub enum DeploymentNamespace {
    Prod,
    Staging,
}

impl ToString for DeploymentNamespace {
    fn to_string(&self) -> String {
        match self {
            DeploymentNamespace::Prod => "Prod".to_string(),
            DeploymentNamespace::Staging => "Staging".to_string(),
        }
    }
}

impl Deployment {
    pub async fn handle_command(&self, context: GlobalContext) -> Result<bool, CliError> {
        match &self.subcommand {
            DeploymentSubcommand::List => handle_list(context).await,
            DeploymentSubcommand::Execute {
                namespace,
                version,
                artifact,
            } => handle_execute(context, namespace, version, artifact).await,
        }
    }
}
