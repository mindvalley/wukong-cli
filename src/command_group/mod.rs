pub mod pipeline;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum CommandGroup {
    /// This contains the commands to view & interact with an application’s pipeline
    Pipeline(pipeline::Pipeline),
}
