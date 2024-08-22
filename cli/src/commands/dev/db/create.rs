use wukong_telemetry::*;
use wukong_telemetry_macro::*;

use crate::{
    commands::Context, config::Config, error::WKCliError, loader::new_spinner,
    output::colored_println, wukong_client::WKClient,
};

#[wukong_telemetry(command_event = "dev_db_create")]
pub async fn handle_db_create(context: Context, branch_name: &str) -> Result<bool, WKCliError> {
    let loader = new_spinner();
    loader.set_message("Creating a new database branch ... ");

    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    let db_branch = wk_client
        .create_database_branch(&context.current_application, branch_name)
        .await?
        .create_database_branch;

    loader.finish_and_clear();

    colored_println!("Database branch {} created successfully!", db_branch.name);
    colored_println!("Hostname: {}", db_branch.hostname);
    colored_println!("Username: {}", db_branch.username);
    colored_println!("Password: {}", db_branch.password);

    Ok(true)
}
