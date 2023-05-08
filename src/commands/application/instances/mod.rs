use crate::{commands::application::ApplicationNamespace, error::CliError};
use clap::{Args, Subcommand};

use self::list::handle_list;

mod list;

#[derive(Debug, Args)]
pub struct Instances {
    #[command(subcommand)]
    pub subcommand: InstancesSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum InstancesSubcommand {
    /// Listing the currently running Elixir instances, normally under a GKE Pod.
    List {
        /// (optional) The namespace to list the running instances.
        #[arg(long, value_enum, default_value_t=ApplicationNamespace::Prod)]
        namespace: ApplicationNamespace,
    },
}

impl Instances {
    pub async fn handle_command(&self) -> Result<bool, CliError> {
        match &self.subcommand {
            InstancesSubcommand::List { namespace } => handle_list().await,
        }
    }
}
