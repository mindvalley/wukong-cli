use chrono::{DateTime, Local, NaiveDateTime, Utc};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use tabled::Tabled;
use time_humanize::HumanTime;

use crate::{
    application_config::{
        ApplicationConfig, ApplicationConfigs, ApplicationNamespaceAppsignalConfig,
    },
    commands::Context,
    config::Config,
    error::WKCliError,
    loader::new_spinner,
    output::table::{fmt_u64_separate_with_commas, TableOutput},
    wukong_client::WKClient,
};

use super::DeploymentVersion;

#[derive(Tabled, Serialize, Deserialize, Debug, Default)]
struct AppsignalData {
    #[tabled(rename = "Open Errors", display_with = "fmt_u64_separate_with_commas")]
    open_errors: u64,
    #[tabled(rename = "In Deploy", display_with = "fmt_u64_separate_with_commas")]
    in_deploy: u64,
    #[tabled(rename = "Total", display_with = "fmt_u64_separate_with_commas")]
    total: u64,
}

#[derive(Default)]
struct AllStatus {
    deployment: DisplayOrNot<DeploymentStatus, String>,
    appsignal: DisplayOrNot<AppsignalStatus, String>,
}

#[derive(Default)]
struct DeploymentStatus {
    artifact: String,
    deployed_at: Option<i64>,
}
#[derive(Default)]
struct AppsignalStatus {
    data: AppsignalData,
    app_id: String,
}
enum DisplayOrNot<T, String> {
    Display(T),
    NotDisplay(String),
}

impl<T> Default for DisplayOrNot<T, String> {
    fn default() -> Self {
        DisplayOrNot::NotDisplay("N/A".to_string())
    }
}

pub async fn handle_status(
    context: Context,
    version: &DeploymentVersion,
) -> Result<bool, WKCliError> {
    let fetch_loader = new_spinner();
    fetch_loader.set_message("Fetching latest deployment ... ");

    let mut all_status = AllStatus::default();

    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    let cd_pipelines_data = wk_client
        .fetch_cd_pipelines(&context.current_application)
        .await?
        .cd_pipelines;

    match cd_pipelines_data.iter().find(|cd_pipeline| {
        cd_pipeline.environment == "prod"
            && cd_pipeline.version == version.to_string().to_lowercase()
    }) {
        Some(deployment) => {
            all_status.deployment = DisplayOrNot::Display(DeploymentStatus {
                artifact: deployment
                    .build_artifact
                    .clone()
                    .unwrap_or("unknown".to_string()),
                deployed_at: deployment.last_deployment,
            });
        }
        None => {
            // if deployment not found, return early
            // because there is no needed to fetch appsignal data if deployment not found
            fetch_loader.finish_and_clear();
            eprintln!("Deployment not found.");
            return Ok(false);
        }
    }

    fetch_loader.set_message("Checking application config ... ");

    let application_configs = ApplicationConfigs::load()?;
    let appsignal_config = get_appsignal_config(application_configs);

    match appsignal_config {
        Ok(ApplicationNamespaceAppsignalConfig { app_id, .. }) => {
            fetch_loader.set_message("Fetching Appsignal data ... ");
            all_status.appsignal = get_appsignal_status(&mut wk_client, app_id).await?;
        }
        Err(reason) => {
            all_status.appsignal = DisplayOrNot::NotDisplay(reason);
        }
    }

    fetch_loader.finish_and_clear();

    if let DisplayOrNot::Display(deployment) = all_status.deployment {
        println!("Deployed build artifact: {}", deployment.artifact);

        if let Some(last_deployed_at) = deployment.deployed_at {
            let naive = NaiveDateTime::from_timestamp_opt(
                last_deployed_at / 1000,
                (last_deployed_at % 1000) as u32 * 1_000_000,
            )
            .unwrap();
            let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc).with_timezone(&Local);
            println!(
                "Deployed since: {}",
                HumanTime::from(Into::<std::time::SystemTime>::into(dt))
            );
        }

        match all_status.appsignal {
            DisplayOrNot::Display(appsignal) => {
                let table = TableOutput {
                    title: None,
                    header: Some("Prod".to_string()),
                    data: vec![appsignal.data],
                };

                println!();
                println!("PERFORMANCE DATA");
                println!("{table}");
                println!("To view more, open these magic links:");
                println!(
                    "AppSignal: https://appsignal.com/mindvalley/sites/{}/exceptions?incident_marker=last",
                    appsignal.app_id,
                );
            }
            DisplayOrNot::NotDisplay(reason) => {
                println!();
                println!(
                    "* Appsignal status is not display because {}",
                    reason.bold()
                );
            }
        }
    }

    Ok(true)
}

fn get_appsignal_config(
    application_configs: ApplicationConfigs,
) -> Result<ApplicationNamespaceAppsignalConfig, String> {
    if let Some(ApplicationConfig {
        enable: true,
        namespaces,
        ..
    }) = application_configs.application
    {
        let appsignal_config = namespaces
            .iter()
            .find(|ns| ns.namespace_type == "prod")
            .and_then(|ns| ns.appsignal.clone());

        if let Some(appsignal_config) = appsignal_config {
            if !appsignal_config.enable {
                return Err("Appsignal is not enabled for this application.".to_string());
            }

            Ok(appsignal_config)
        } else {
            Err("Appsignal config not found for `prod` namespace.".to_string())
        }
    } else {
        Err("Application config is not enabled.".to_string())
    }
}

async fn get_appsignal_status(
    wk_client: &mut WKClient,
    app_id: String,
) -> Result<DisplayOrNot<AppsignalStatus, String>, WKCliError> {
    let appsignal_deploy_markers = wk_client
        .fetch_appsignal_deploy_markers(&app_id, Some(1))
        .await?
        .appsignal_deploy_markers;
    if let Some(latest_deploy_marker) = appsignal_deploy_markers.first() {
        let appsignal_exception_incidents = wk_client
            .fetch_appsignal_exception_incidents(
                &app_id,
                vec![],
                None,
                Some(latest_deploy_marker.id.clone()),
            )
            .await?
            .appsignal_exception_incidents;

        let open_count = appsignal_exception_incidents.len();
        let in_deploy_count = appsignal_exception_incidents
            .iter()
            .fold(0, |acc, incident| {
                if let Some(count) = incident.per_marker_count {
                    acc + count
                } else {
                    acc
                }
            });
        let total_count = appsignal_exception_incidents
            .iter()
            .fold(0, |acc, incident| acc + incident.count);

        Ok(DisplayOrNot::Display(AppsignalStatus {
            data: AppsignalData {
                open_errors: open_count as u64,
                in_deploy: in_deploy_count as u64,
                total: total_count as u64,
            },
            app_id: app_id.clone(),
        }))
    } else {
        Ok(DisplayOrNot::NotDisplay(
            "No deployment marker found.".to_string(),
        ))
    }
}
