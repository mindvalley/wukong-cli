mod diff;
mod pull;
mod push;
mod utils;

use crate::error::CliError;
use clap::{Args, Subcommand};
use diff::handle_config_diff;
use pull::handle_config_pull;
use push::handle_config_push;
use std::path::PathBuf;

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
    /// Pull the development config with secrets file from Bunker.
    Pull {
        /// The path to the project
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

impl Config {
    pub async fn handle_command(&self) -> Result<bool, CliError> {
        match &self.subcommand {
            ConfigSubcommand::Push => handle_config_push().await,
            ConfigSubcommand::Diff => handle_config_diff().await,
            ConfigSubcommand::Pull { path } => handle_config_pull(path).await,
        }
    }
}
