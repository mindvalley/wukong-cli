mod config;
mod db;

use crate::error::WKCliError;
use clap::{Args, Subcommand};

use super::{get_context, ClapApp};

#[derive(Debug, Args)]
pub struct Dev {
    #[command(subcommand)]
    pub subcommand: DevSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DevSubcommand {
    /// This command group contains the commands to interact with the config secrets with bunker.
    Config(config::Config),
    Db(db::Db),
}

impl Dev {
    pub async fn handle_command(&self, clap_app: &ClapApp) -> Result<bool, WKCliError> {
        match &self.subcommand {
            DevSubcommand::Config(config) => config.handle_command(clap_app).await,
            DevSubcommand::Db(db) => db.handle_command(get_context(clap_app)?).await,
        }
    }
}
