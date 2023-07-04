#![forbid(unsafe_code)]

mod app;
mod commands;
mod config;
mod error;
mod loader;
mod logger;
mod output;
mod utils;

use app::App;
use error::WKCliError;
use human_panic::setup_panic;
use log::{error, info};
use std::process;
use wukong_sdk::output::error::ErrorOutput;

#[tokio::main]
async fn main() {
    setup_panic!();

    // TODO: make sure that the cursor re-appears when interrupting
    // tokio::spawn(async move {
    //     tokio::signal::ctrl_c().await.unwrap();
    //     let term = dialoguer::console::Term::stdout();
    //     let _ = term.show_cursor();
    //     process::exit(1);
    // });

    match run().await {
        Err(error) => {
            // error!("{}", ErrorOutput(error));
            println!("Error: {}", error);
            process::exit(1);
        }
        Ok(false) => {
            info!("wukong cli session ended.");
            process::exit(1);
        }
        Ok(true) => {
            info!("wukong cli session ended.");
            process::exit(0);
        }
    }
}

async fn run() -> Result<bool, WKCliError> {
    let app = App::new()?;

    app.cli.execute().await
}
