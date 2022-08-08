pub mod config;
pub mod pipeline;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum CommandGroup {
    /// Initialize Wukong's configurations
    Init,
    /// This contains the commands to view & interact with an application’s pipeline
    Pipeline(pipeline::Pipeline),
    /// This contains the commands to view & interact with Wukong's configurations
    Config(config::Config),
    /// Login to start using wukong command
    Login,
}
