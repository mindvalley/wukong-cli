use crate::{error::CliError, GlobalContext};
use clap::{command, Args, Subcommand};
use info::handle_info;

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
    pub async fn handle_command(&self, context: GlobalContext) -> Result<bool, CliError> {
        match &self.subcommand {
            ApplicationSubcommand::Info => handle_info(context).await,
        }
    }
}
