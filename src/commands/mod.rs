pub mod completions;
pub mod config;
pub mod deployment;
pub mod init;
pub mod login;
pub mod pipeline;

use clap::Subcommand;
use clap_complete::Shell;

#[derive(Debug, Subcommand)]
pub enum CommandGroup {
    /// Initialize Wukong's configurations
    Init,
    /// This contains the commands to view & interact with an application’s pipeline
    Pipeline(pipeline::Pipeline),
    /// This command group contains the commands to view and interact with the
    /// Continuous Delivery pipeline of an application.
    Deployment(deployment::Deployment),
    /// This contains the commands to view & interact with Wukong's configurations
    Config(config::Config),
    /// Login to start using wukong command
    Login,
    /// Generate wukong cli completions for your shell to stdout
    Completions {
        #[clap(value_enum)]
        shell: Shell,
    },
}
