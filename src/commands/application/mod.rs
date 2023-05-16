use crate::error::CliError;
use clap::{command, Args, Subcommand, ValueEnum};
use info::handle_info;

use super::{Context, State};

pub mod info;
pub mod instances;

#[derive(Debug, Args)]
pub struct Application {
    #[command(subcommand)]
    pub subcommand: ApplicationSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ApplicationSubcommand {
    /// Show the application’s relevant informations
    Info,
    /// Listing the currently running Elixir instances, normally under a GKE Pod.
    Instances(instances::Instances),
}

#[derive(Debug, ValueEnum, Clone)]
pub enum ApplicationNamespace {
    Prod,
    Staging,
}

impl ToString for ApplicationNamespace {
    fn to_string(&self) -> String {
        match self {
            ApplicationNamespace::Prod => "prod".to_string(),
            ApplicationNamespace::Staging => "staging".to_string(),
        }
    }
}

#[derive(Debug, ValueEnum, Clone)]
pub enum ApplicationVersion {
    Green,
    Blue,
}

impl ToString for ApplicationVersion {
    fn to_string(&self) -> String {
        match self {
            ApplicationVersion::Green => "green".to_string(),
            ApplicationVersion::Blue => "blue".to_string(),
        }
    }
}

impl Application {
    pub async fn handle_command(&self, state: State) -> Result<bool, CliError> {
        let context = Context::from_state(state).await?;

        match &self.subcommand {
            ApplicationSubcommand::Info => handle_info(context).await,
            ApplicationSubcommand::Instances(instances) => instances.handle_command(context).await,
        }
    }
}
