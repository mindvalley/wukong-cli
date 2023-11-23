use crate::error::WKCliError;
use clap::{Args, Subcommand};
use std::str;

use self::login::handle_login;
mod login;

#[derive(Debug, Args)]
pub struct Gcloud {
    #[command(subcommand)]
    pub subcommand: GcloudSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum GcloudSubcommand {
    /// Login to Google Cloud
    Login,
}

impl Gcloud {
    pub async fn handle_command(&self) -> Result<bool, WKCliError> {
        match &self.subcommand {
            GcloudSubcommand::Login => handle_login().await,
        }
    }
}
