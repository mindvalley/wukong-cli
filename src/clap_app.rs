use crate::commands::CommandGroup;
use clap::Parser;
use clap_complete::{generate, Generator, Shell};

/// A Swiss-army Knife CLI For Mindvalley Developers
#[derive(Debug, Parser)]
#[clap(version, author)]
pub struct ClapApp {
    /// If provided, outputs the completion file for given shell
    #[clap(long = "generate", arg_enum)]
    pub generator: Option<Shell>,

    #[clap(subcommand)]
    pub command_group: CommandGroup,

    /// Override the application name that the CLI will perform the command against.
    /// If the flag is not used, then the CLI will use the default application name from the config.
    #[clap(long, short, global = true)]
    pub application: Option<String>,
}
