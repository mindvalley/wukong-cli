use crate::{
    commands::Context,
    services::vault::Vault,
    telemetry::{self, TelemetryData, TelemetryEvent},
    CliError,
};
use wukong_telemetry_macro::wukong_telemetry;

#[wukong_telemetry(command_event = "vault_list")]
pub async fn handle_list(context: Context) -> Result<bool, CliError> {
    // Call the vault client:
    let _client = Vault::new().get_lists().await?;

    Ok(true)
}
