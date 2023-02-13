use crate::{
    commands::Context,
    error::CliError,
    graphql::QueryClientBuilder,
    telemetry::{self, TelemetryData, TelemetryEvent},
};
use owo_colors::OwoColorize;
use wukong_telemetry_macro::wukong_telemetry;

#[wukong_telemetry(command_event = "application_info")]
pub async fn handle_info(context: Context) -> Result<bool, CliError> {
    // Calling API ...
    let client = QueryClientBuilder::default()
        .with_access_token(
            context
                .config
                .auth
                .ok_or(CliError::UnAuthenticated)?
                .id_token,
        )
        .with_sub(context.state.sub)
        .with_api_url(context.config.core.wukong_api_url)
        .build()?;

    let application_resp = client
        .fetch_application(&context.state.application.unwrap()) // SAFERY: the application is checked on the caller so it will always be Some(x) here
        .await?
        .data
        .unwrap()
        .application;

    if let Some(application_data) = application_resp {
        match application_data.basic_info {
            Some(basic_info) => {
                println!("Application Info for {}:", application_data.name.green());
                println!();

                println!("Deployment:");
                println!("Target - {}", basic_info.deployment_target);
                println!("Stragety - {}", basic_info.deployment_strategy);

                if let Some(links) = basic_info.links {
                    println!();
                    println!("Links:");

                    for link in links.into_iter().flatten() {
                        println!("{} - {}", link.title, link.url);
                    }
                }
            }
            None => {
                println!(
                    "There is no info for the application {}.",
                    application_data.name.green()
                );
            }
        }
    }

    Ok(true)
}
