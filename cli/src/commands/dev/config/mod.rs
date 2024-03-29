mod diff;
mod lint;
mod pull;
mod push;
mod utils;

use diff::handle_config_diff;
use lint::handle_config_lint;
use pull::handle_config_pull;
use push::handle_config_push;

use crate::{
    commands::{get_context_without_application, ClapApp},
    error::WKCliError,
};
use clap::{Args, Subcommand};
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
    pub async fn handle_command(&self, clap_app: &ClapApp) -> Result<bool, WKCliError> {
        match &self.subcommand {
            ConfigSubcommand::Push => {
                handle_config_push(get_context_without_application(clap_app)?).await
            }
            ConfigSubcommand::Diff => {
                handle_config_diff(get_context_without_application(clap_app)?).await
            }
            ConfigSubcommand::Pull { path } => {
                handle_config_pull(get_context_without_application(clap_app)?, path).await
            }
            ConfigSubcommand::Lint { path } => handle_config_lint(path),
        }
    }
}
