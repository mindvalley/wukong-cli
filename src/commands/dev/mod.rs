mod config;

use crate::error::CliError;
use clap::{Args, Subcommand};

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
    pub async fn handle_command(&self) -> Result<bool, CliError> {
        match &self.subcommand {
            DevSubcommand::Config(config) => config.handle_command().await,
        }
    }
}
