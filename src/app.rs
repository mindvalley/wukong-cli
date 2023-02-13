use crate::{commands::ClapApp, config::Config, error::CliError, logger};
use clap::Parser;
use once_cell::sync::OnceCell;

pub struct App {
    // pub config: ConfigState,
    pub cli: ClapApp,
}

#[derive(Debug)]
pub struct AppState {
    pub api_url: String,
    pub okta_client_id: String,
}

// pub static APP_STATE: OnceCell<AppState> = OnceCell::new();
pub static APP_CONFIG: OnceCell<Config> = OnceCell::new();

impl App {
    pub fn new() -> Result<Self, CliError> {
        let cli = ClapApp::parse();

        logger::Builder::new()
            .with_max_level(cli.verbose.log_level_filter())
            .init();

        Ok(Self { cli })
    }
}
