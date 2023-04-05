mod push;

use crate::error::CliError;
use clap::{Args, Subcommand};
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
    Push {
        /// The path to the project
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

impl Config {
    pub async fn handle_command(&self) -> Result<bool, CliError> {
        match &self.subcommand {
            ConfigSubcommand::Push { path } => handle_config_push(path).await,
        }
    }
}
