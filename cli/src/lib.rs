mod app;
mod application_config;
mod auth;
mod commands;
mod config;
mod error;
mod loader;
mod logger;
pub mod output;
mod update;
mod utils;
mod wukong_client;

use app::App;
use error::WKCliError;

pub async fn run() -> Result<bool, WKCliError> {
    let app = App::new()?;

    app.cli.execute().await
}
