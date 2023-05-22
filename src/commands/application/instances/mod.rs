use self::list::handle_list;
use crate::{
    commands::{application::ApplicationNamespace, Context},
    error::CliError,
};
use clap::{Args, Subcommand};

use super::ApplicationVersion;

mod connect;
mod list;

#[derive(Debug, Args)]
pub struct Instances {
    #[command(subcommand)]
    pub subcommand: InstancesSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum InstancesSubcommand {
    /// Listing the currently running Elixir instances, normally under a GKE Pod.
    ///
    /// List all the available running Pods for an application. It will show the Pod’s name and IP address.
    List {
        /// (optional) The namespace to list the running instances.
        #[arg(long, value_enum, default_value_t=ApplicationNamespace::Prod)]
        namespace: ApplicationNamespace,

        /// (optional) The version of the application to filter the returning running instances.
        #[arg(long, value_enum, default_value_t=ApplicationVersion::Green)]
        version: ApplicationVersion,
    },
    /// Start the interactive session to connect to the remote Elixir instance.
    Connect {
        /// The instance name to connect to.
        name: String,
        /// (optional) The port for livebook instance.
        #[arg(long, short, default_value_t = 8080)]
        port: u16,
    },
}

impl Instances {
    pub async fn handle_command(&self, context: Context) -> Result<bool, CliError> {
        match &self.subcommand {
            InstancesSubcommand::List { namespace, version } => {
                handle_list(context, &namespace.to_string(), &version.to_string()).await
            }
            InstancesSubcommand::Connect { name, port } => todo!(),
        }
    }
}
