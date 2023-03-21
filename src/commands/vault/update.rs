use crate::{
    commands::Context,
    services::vault::Vault,
    telemetry::{self, TelemetryData, TelemetryEvent},
    CliError,
};
use wukong_telemetry_macro::wukong_telemetry;

#[wukong_telemetry(command_event = "vault_list")]
pub async fn update_secret(context: Context) -> Result<bool, CliError> {
    // Call the vault client:
    let _client = Vault::new()
        .update_secret("wukong-cli/development", "tests", "test3")
        .await?;

    print!("{:?}", _client);
    Ok(true)
}
