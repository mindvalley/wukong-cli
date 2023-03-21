pub mod list;
pub mod login;
pub mod update;

use super::{Context, State};
use crate::CliError;
use clap::{Args, Subcommand};
use list::handle_list;
use login::handle_login;
use update::update_secret;

#[derive(Debug, Args)]
pub struct Vault {
    #[command(subcommand)]
    pub subcommand: VaultSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum VaultSubcommand {
    /// List data or secrets
    List,
    /// Authenticate locally
    Login,
    /// Update Secret
    Update,
}

impl Vault {
    pub async fn handle_command(&self, state: State) -> Result<bool, CliError> {
        let context = Context::from_state(state).await?;

        match &self.subcommand {
            VaultSubcommand::List => handle_list(context).await,
            VaultSubcommand::Login => handle_login(context).await,
            VaultSubcommand::Update => update_secret(context).await,
        }
    }
}
