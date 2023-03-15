use crate::{
    commands::Context,
    loader::new_spinner_progress_bar,
    services::vault::client::VaultClient,
    telemetry::{self, TelemetryData, TelemetryEvent},
    CliError,
};
use wukong_telemetry_macro::wukong_telemetry;

#[wukong_telemetry(command_event = "vault_list")]
pub async fn handle_list(context: Context) -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Fetching vault list ... ");

    // Get the vaule token from the context:
    let api_key = context
        .config
        .auth
        .ok_or(CliError::UnAuthenticated)?
        .id_token;

    // Call the vault client:
    let client = VaultClient::new(Some(&api_key)).fetch_lists().await?;

    progress_bar.finish_and_clear();

    Ok(true)
}
