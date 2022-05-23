use clap::Parser;

/// A Swiss-army Knife CLI For Mindvalley Developers
#[derive(Parser)]
#[clap(version)]
struct Cli;

fn main() {
    Cli::parse();
}
