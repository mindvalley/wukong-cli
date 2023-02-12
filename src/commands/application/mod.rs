use crate::{
    app::APP_CONFIG,
    config::{Config, CONFIG_FILE},
    error::CliError,
    GlobalContext,
};
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
    pub async fn handle_command(&self, mut context: GlobalContext) -> Result<bool, CliError> {
        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let config = Config::load(config_file)?;

        if config.auth.is_none() {
            return Err(CliError::UnAuthenticated);
        }

        if context.application.is_none() {
            context.application = Some(config.core.application.clone());
        }

        context.sub = Some(config.auth.as_ref().unwrap().subject.clone());

        APP_CONFIG.set(config).unwrap();

        match &self.subcommand {
            ApplicationSubcommand::Info => handle_info(context).await,
        }
    }
}
