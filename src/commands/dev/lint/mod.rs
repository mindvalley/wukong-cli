mod lint;

use crate::error::CliError;
use clap::{Args, Subcommand};
use lint::handle_config_lint;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct Lint {
    #[command(subcommand)]
    pub subcommand: LintSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum LintSubcommand {
    /// Linting the config and show possible warnings, as well as suggestion how to fix the config file.
    Lint {
        /// The path to the project
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

impl Lint {
    pub async fn handle_command(&self) -> Result<bool, CliError> {
        match &self.subcommand {
            LintSubcommand::Lint { path } => handle_config_lint(path),
        }
    }
}
