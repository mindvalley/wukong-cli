mod config;
mod config_lint;

use crate::error::CliError;
use clap::{Args, Subcommand};
use config_lint::handle_config_lint;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct Dev {
    #[command(subcommand)]
    pub subcommand: DevSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DevSubcommand {
    /// Linting the config and show possible warnings, as well as suggestion how to fix the config file.
    ConfigLint {
        /// The path to the project
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// This command group contains the commands to interact with the config secrets with bunker.
    Config(config::Config),
}

impl Dev {
    pub async fn handle_command(&self) -> Result<bool, CliError> {
        match &self.subcommand {
            DevSubcommand::ConfigLint { path } => handle_config_lint(path),
            DevSubcommand::Config(config) => config.handle_command().await,
        }
    }
}
