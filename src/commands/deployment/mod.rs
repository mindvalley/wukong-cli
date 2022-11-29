pub mod execute;
pub mod list;

use execute::handle_execute;
use list::handle_list;

use crate::{
    telemetry::{self, Command, TelemetryData},
    CliError, GlobalContext,
};
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
        // SAFETY: the application can't be None since it is checked in the caller
        let current_application = context.application.as_ref().unwrap().clone();
        // SAFETY: the sub can't be None since it is checked in the caller
        let current_sub = context.sub.as_ref().unwrap().clone();

        match &self.subcommand {
            DeploymentSubcommand::List => {
                TelemetryData::new(
                    Some(Command {
                        name: "deployment_list".to_string(),
                        run_mode: telemetry::CommandRunMode::NonInteractive,
                    }),
                    Some(current_application),
                    current_sub,
                )
                .record_event()
                .await;

                handle_list(context).await
            }
            DeploymentSubcommand::Execute {
                namespace,
                version,
                artifact,
            } => {
                TelemetryData::new(
                    Some(Command {
                        name: "deployment_execute".to_string(),
                        run_mode: telemetry::CommandRunMode::Interactive,
                    }),
                    Some(current_application),
                    current_sub,
                )
                .record_event()
                .await;

                handle_execute(context, namespace, version, artifact).await
            }
        }
    }
}
