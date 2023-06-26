mod app;
mod auth;
mod commands;
mod config;
mod error;
mod graphql;
mod loader;
mod logger;
pub mod output;
pub mod services {
    pub mod gcloud;
    pub mod vault;
}
mod telemetry;
mod utils;

use app::App;
use config::Config;
use error::CliError;

pub async fn run() -> Result<bool, CliError> {
    let app = App::new()?;

    app.cli.execute().await
}
