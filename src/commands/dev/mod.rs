mod config_lint;

use std::path::PathBuf;

use super::{Context, State};
use crate::error::CliError;
use clap::{Args, Subcommand, ValueEnum};
use config_lint::handle_config_lint;

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
}

impl Dev {
    pub fn handle_command(&self) -> Result<bool, CliError> {
        match &self.subcommand {
            DevSubcommand::ConfigLint { path } => handle_config_lint(path),
        }
    }
}
