mod config;

use crate::error::WKCliError;
use clap::{Args, Subcommand};

use super::Context;

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
    pub async fn handle_command(&self, context: Context) -> Result<bool, WKCliError> {
        match &self.subcommand {
            DevSubcommand::Config(config) => config.handle_command(context).await,
        }
    }
}
