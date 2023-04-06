mod diff;
mod push;
mod utils;

use crate::error::CliError;
use clap::{Args, Subcommand};
use diff::handle_config_diff;
use push::handle_config_push;

#[derive(Debug, Args)]
pub struct Config {
    #[command(subcommand)]
    pub subcommand: ConfigSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    /// Push the current configuration changes to the Bunker.
    Push,
    /// Show changes between the local configuration and the Bunker.
    Diff,
}

impl Config {
    pub async fn handle_command(&self) -> Result<bool, CliError> {
        match &self.subcommand {
            ConfigSubcommand::Push => handle_config_push().await,
            ConfigSubcommand::Diff => handle_config_diff().await,
        }
    }
}
