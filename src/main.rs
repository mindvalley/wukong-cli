#![forbid(unsafe_code)]

use human_panic::setup_panic;
use log::{error, info};
use std::process;
use wukong::{output::error::ErrorOutput, run};

#[tokio::main]
async fn main() {
    setup_panic!();

    // make sure that the cursor re-appears when interrupting
    ctrlc::set_handler(move || {
        let term = dialoguer::console::Term::stdout();
        let _ = term.show_cursor();
        std::process::exit(1);
    })
    .expect("Error setting Ctrl-C handler");

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
