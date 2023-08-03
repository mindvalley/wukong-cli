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

        logger::Builder::new()
            .with_max_level(cli.verbose.log_level_filter())
            .init();

        Ok(Self { cli })
    }
}
