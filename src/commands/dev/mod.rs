mod config;

use crate::error::CliError;
use clap::{Args, Subcommand};

use super::{Context, State};

#[derive(Debug, Args)]
pub struct Dev {
    #[command(subcommand)]
    pub subcommand: DevSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DevSubcommand {
    /// This command group contains the commands to interact with the config secrets with bunker.
    Config(config::Config),
}

impl Dev {
    pub async fn handle_command(&self, state: State) -> Result<bool, CliError> {
        let context = Context::from_state(state).await?;

        match &self.subcommand {
            DevSubcommand::Config(config) => config.handle_command(context).await,
        }
    }
}
