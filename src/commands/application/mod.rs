use crate::{
    config::{Config, CONFIG_FILE},
    error::CliError,
};
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
    pub async fn handle_command(&self, mut state: State) -> Result<bool, CliError> {
        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let config = Config::load(config_file)?;

        if state.application.is_none() {
            state.application = Some(config.core.application.clone());
        }
        state.sub = Some(
            config
                .auth
                .as_ref()
                .ok_or(CliError::UnAuthenticated)?
                .subject
                .clone(),
        );

        let context = Context { state, config };

        match &self.subcommand {
            ApplicationSubcommand::Info => handle_info(context).await,
        }
    }
}
