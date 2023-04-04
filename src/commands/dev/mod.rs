mod config_lint;
mod config_synthesizer;
mod config_secrets_push;

use crate::error::CliError;
use clap::{Args, Subcommand};
use config_lint::handle_config_lint;
use config_synthesizer::handle_config_synthesizer;
use config_secrets_push::handle_config_secrets_push;
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
    /// Synthesize the development config with secrets file from Bunker.
    ConfigSynthesizer {
        /// The path to the project
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Edit the config file from the vault
    ConfigSecretsPush {
        /// The path to the project
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

impl Dev {
    pub async fn handle_command(&self) -> Result<bool, CliError> {
        match &self.subcommand {
            DevSubcommand::ConfigLint { path } => handle_config_lint(path),
            DevSubcommand::ConfigSynthesizer { path } => handle_config_synthesizer(path).await,
            DevSubcommand::ConfigSecretsPush { path } => handle_config_secrets_push(path).await,
        }
    }
}
