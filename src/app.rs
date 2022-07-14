use crate::{
    clap_app::ClapApp,
    config::{Config, CONFIG_FILE},
    error::CliError,
};
use clap::Parser;

pub struct App {
    pub config: Config,
    pub cli: ClapApp,
}

impl App {
    pub fn new<'a>() -> Result<Self, CliError<'a>> {
        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let config = Config::load(config_file)?;
        Ok(Self {
            config,
            cli: ClapApp::parse(),
        })
    }
}
