mod create;

use create::handle_db_create;

use crate::{commands::Context, error::WKCliError};
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct Db {
    #[command(subcommand)]
    pub subcommand: DbSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DbSubcommand {
    /// Create a new database branch
    Create {
        /// The name of the new database branch
        branch_name: String,
    },
}

impl Db {
    pub async fn handle_command(&self, context: Context) -> Result<bool, WKCliError> {
        match &self.subcommand {
            DbSubcommand::Create { branch_name } => handle_db_create(context, branch_name).await,
        }
    }
}
