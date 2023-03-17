use crate::{
    commands::Context,
    loader::new_spinner_progress_bar,
    services::vault::Vault,
    telemetry::{self, TelemetryData, TelemetryEvent},
    CliError,
};
use wukong_telemetry_macro::wukong_telemetry;

#[wukong_telemetry(command_event = "vault_list")]
pub async fn handle_list(context: Context) -> Result<bool, CliError> {
    // let progress_bar = new_spinner_progress_bar();
    // progress_bar.set_message("Fetching vault list ... ");

    // let api_key = context
    //     .config
    //     .vault
    //     .ok_or(CliError::UnAuthenticated)?
    //     .api_key;

    // Call the vault client:
    let _client = Vault::new(None).get_lists().await?;

    // progress_bar.finish_and_clear();

    Ok(true)
}
