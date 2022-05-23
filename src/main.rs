#![forbid(unsafe_code)]

mod command_group;

use clap::Parser;
use command_group::{pipeline::PipelineSubcommand, CommandGroup};
/// A Swiss-army Knife CLI For Mindvalley Developers
#[derive(Debug, Parser)]
#[clap(version)]
struct Cli {
    #[clap(subcommand)]
    command_group: CommandGroup,
}

fn main() {
    let cli = Cli::parse();

    match cli.command_group {
        CommandGroup::Pipeline(pipeline) => {
            println!("{:?}", pipeline);
            match pipeline.subcommand {
                PipelineSubcommand::List => {
                    println!("List all pipelines");
                    todo!()
                }
                PipelineSubcommand::Describe { name } => todo!(),
                PipelineSubcommand::CiStatus => todo!(),
            }
        }
    }
}
