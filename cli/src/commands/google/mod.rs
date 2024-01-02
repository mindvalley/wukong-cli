use crate::error::WKCliError;
use clap::{Args, Subcommand};
use std::str;

use self::login::handle_login;
pub mod login;

#[derive(Debug, Args)]
pub struct Google {
    #[command(subcommand)]
    pub subcommand: GoogleSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum GoogleSubcommand {
    /// Login to Google
    Login,
}

impl Google {
    pub async fn handle_command(&self) -> Result<bool, WKCliError> {
        match &self.subcommand {
            GoogleSubcommand::Login => handle_login().await,
        }
    }
}
