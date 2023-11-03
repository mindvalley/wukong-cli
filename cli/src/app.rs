use crate::{commands::ClapApp, error::WKCliError, logger};
use clap::Parser;

pub struct App {
    pub cli: ClapApp,
}

#[derive(Debug)]
pub struct AppState {
    pub api_url: String,
    pub okta_client_id: String,
}

impl App {
    pub fn new() -> Result<Self, WKCliError> {
        let cli = ClapApp::parse();

        // Rewrite this to open file on append mode:
        logger::Builder::new()
            .with_max_level(cli.verbose.log_level_filter())
            .with_report(cli.report)
            .init();

        Ok(Self { cli })
    }
}
