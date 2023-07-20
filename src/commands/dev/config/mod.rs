mod diff;
mod lint;
mod pull;
mod push;
mod utils;

use crate::{
    commands::{Context, State},
    error::CliError,
};
use clap::{Args, Subcommand};
use diff::handle_config_diff;
use lint::handle_config_lint;
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
    /// Pull the development config file from Bunker.
    Pull {
        /// The path to the project
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Linting the config and show possible warnings, as well as suggestion how to fix the config file.
    Lint {
        /// The path to the project
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

impl Config {
    pub async fn handle_command(&self, state: State) -> Result<bool, CliError> {
        match &self.subcommand {
            ConfigSubcommand::Push => {
                let context = Context::from_state(state).await?;
                handle_config_push(context).await
            }
            ConfigSubcommand::Diff => {
                let context = Context::from_state(state).await?;
                handle_config_diff(context).await
            }
            ConfigSubcommand::Pull { path } => {
                let context = Context::from_state(state).await?;
                handle_config_pull(context, path).await
            }
            ConfigSubcommand::Lint { path } => handle_config_lint(path),
        }
    }
}