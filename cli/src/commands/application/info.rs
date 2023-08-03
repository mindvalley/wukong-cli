use crate::{commands::Context, config::Config, error::WKCliError, wukong_client::WKClient};
use owo_colors::OwoColorize;
use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[wukong_telemetry(command_event = "application_info")]
pub async fn handle_info(context: Context) -> Result<bool, WKCliError> {
    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::new(&config)?;

    let application_resp = wk_client
        .fetch_application(&context.current_application) // SAFERY: the application is checked on the caller so it will always be Some(x) here
        .await?
        .application;

    if let Some(application_data) = application_resp {
        if let Some(basic_info) = application_data.basic_info {
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

        return Ok(true);
    }

    println!(
        "There is no info for the application {}.",
        context.current_application.green()
    );

    Ok(false)
}
