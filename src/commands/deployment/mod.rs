pub mod execute;
pub mod list;
pub mod rollback;

use execute::handle_execute;
use list::handle_list;
use rollback::handle_rollback;

use crate::{
    config::{Config, CONFIG_FILE},
    CliError,
};
use clap::{Args, Subcommand, ValueEnum};

use super::{Context, State};

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
    pub async fn handle_command(&self, mut state: State) -> Result<bool, CliError> {
        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let config = Config::load(config_file)?;

        if state.application.is_none() {
            state.application = Some(config.core.application.clone());
        }
        state.sub = Some(
            config
                .auth
                .as_ref()
                .ok_or(CliError::UnAuthenticated)?
                .subject
                .clone(),
        );

        let context = Context { state, config };

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
        }
    }
}
