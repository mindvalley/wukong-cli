use crate::error::CliError;
use clap::{command, Args, Subcommand};
use info::handle_info;

use super::{Context, State};

pub mod info;

#[derive(Debug, Args)]
pub struct Application {
    #[command(subcommand)]
    pub subcommand: ApplicationSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ApplicationSubcommand {
    /// Show the application’s relevant informations
    Info,
}

impl Application {
    pub async fn handle_command(&self, state: State) -> Result<bool, CliError> {
        let context = Context::from_state(state).await?;

        match &self.subcommand {
            ApplicationSubcommand::Info => handle_info(context).await,
        }
    }
}
