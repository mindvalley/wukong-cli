#![forbid(unsafe_code)]

mod commands;
mod config;
mod error;
mod loader;

// use human_panic::setup_panic;
use log::{error, info};
use std::process;
use wukong_sdk::{output::error::ErrorOutput, run};

#[tokio::main]
async fn main() {
    // setup_panic!();

    // TODO: make sure that the cursor re-appears when interrupting
    // tokio::spawn(async move {
    //     tokio::signal::ctrl_c().await.unwrap();
    //     let term = dialoguer::console::Term::stdout();
    //     let _ = term.show_cursor();
    //     process::exit(1);
    // });

    match run().await {
        Err(error) => {
            error!("{}", ErrorOutput(error));
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
