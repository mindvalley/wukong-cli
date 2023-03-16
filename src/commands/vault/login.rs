use crate::{
    commands::Context,
    services::vault::client::VaultClient,
    telemetry::{self, TelemetryData, TelemetryEvent},
    CliError,
};
use wukong_telemetry_macro::wukong_telemetry;

#[wukong_telemetry(command_event = "vault_login")]
pub async fn handle_login(context: Context) -> Result<bool, CliError> {
    VaultClient::new(None).handle_login().await?;

    Ok(true)
}
