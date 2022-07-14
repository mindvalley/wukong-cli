use crate::command_group::CommandGroup;
use clap::Parser;

/// A Swiss-army Knife CLI For Mindvalley Developers
#[derive(Debug, Parser)]
#[clap(version, author)]
pub struct ClapApp {
    #[clap(subcommand)]
    pub command_group: CommandGroup,

    /// Override the application name that the CLI will perform the command against.
    /// If the flag is not used, then the CLI will use the default application name from the config.
    #[clap(long, short)]
    pub application: Option<String>,
}
