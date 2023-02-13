mod app;
mod auth;
mod commands;
mod config;
mod error;
mod graphql;
mod loader;
mod logger;
pub mod output;
mod telemetry;

use app::App;
use config::{Config, CONFIG_FILE};
use error::CliError;

pub async fn run() -> Result<bool, CliError> {
    let app = App::new()?;

    app.cli.execute().await
}
