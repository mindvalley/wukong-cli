pub mod execute;
pub mod list;

use execute::handle_execute;
use list::handle_list;

use crate::{CliError, GlobalContext};
use clap::{ArgEnum, Args, Subcommand};

#[derive(Debug, Args)]
pub struct Deployment {
    #[clap(subcommand)]
    pub subcommand: DeploymentSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DeploymentSubcommand {
    /// List the current available deployment pipelines of an application
    List,
    /// Start the deployment pipeline
    Execute {
        /// The namespace to deploy to
        #[clap(long)]
        namespace: Option<String>,
        /// The version that the deployment will perform
        /// against.
        #[clap(long, arg_enum, default_value_t = DeploymentVersion::Green)]
        version: DeploymentVersion,
        /// The build artifact that the deployment will use.
        #[clap(long)]
        artifact: Option<String>,
    },
}

#[derive(Debug, ArgEnum, Clone)]
pub enum DeploymentVersion {
    Blue,
    Green,
}

impl Deployment {
    pub async fn handle_command<'a>(&self, context: GlobalContext) -> Result<bool, CliError<'a>> {
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
