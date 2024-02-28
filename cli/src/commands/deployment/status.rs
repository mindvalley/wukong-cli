use chrono::{DateTime, Local, NaiveDateTime, Utc};
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
    output::table::{fmt_u64_separate_with_commas, TableOutput},
    wukong_client::WKClient,
};

use super::DeploymentVersion;

#[derive(Tabled, Serialize, Deserialize, Debug)]
struct AppsignalData {
    #[tabled(rename = "Open Errors", display_with = "fmt_u64_separate_with_commas")]
    open_errors: u64,
    #[tabled(rename = "In Deploy", display_with = "fmt_u64_separate_with_commas")]
    in_deploy: u64,
    #[tabled(rename = "Total", display_with = "fmt_u64_separate_with_commas")]
    total: u64,
}

pub async fn handle_status(
    context: Context,
    version: &DeploymentVersion,
) -> Result<bool, WKCliError> {
    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    let appsignal_app_id;
    let application_configs = ApplicationConfigs::load()?;
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

        if let Some(ApplicationNamespaceAppsignalConfig {
            enable,
            app_id,
            environment: _,
            default_namespace: _,
        }) = appsignal_config
        {
            if !enable {
                eprintln!("Appsignal is not enabled for this application.");
                return Ok(false);
            }

            appsignal_app_id = app_id;
        } else {
            eprintln!("Appsignal config not found for `prod` namespace.");
            return Ok(false);
        }
    } else {
        eprintln!("Application config is not enabled.");
        return Ok(false);
    }

    let mut appsignal_errors_table = TableOutput {
        title: None,
        header: Some("APM".to_string()),
        data: vec![],
    };

    let cd_pipelines_data = wk_client
        .fetch_cd_pipelines(&context.current_application)
        .await?
        .cd_pipelines;

    let appsignal_deploy_markers = wk_client
        .fetch_appsignal_deploy_markers(&appsignal_app_id, Some(1))
        .await?
        .appsignal_deploy_markers;
    if let Some(latest_deploy_marker) = appsignal_deploy_markers.first() {
        let appsignal_exception_incidents = wk_client
            .fetch_appsignal_exception_incidents(
                &appsignal_app_id,
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

        appsignal_errors_table.data.push(AppsignalData {
            open_errors: open_count as u64,
            in_deploy: in_deploy_count as u64,
            total: total_count as u64,
        });
    }

    let prod_deployment = cd_pipelines_data.iter().find(|cd_pipeline| {
        cd_pipeline.environment == "prod"
            && cd_pipeline.version == version.to_string().to_lowercase()
    });

    if let Some(deployment) = prod_deployment {
        println!(
            "Deployed build artifact: {}",
            deployment
                .build_artifact
                .as_ref()
                .unwrap_or(&"unknown".to_string())
        );

        if let Some(last_deployed_at) = deployment.last_deployment {
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
        } else {
            println!("Deployed since: N/A");
        }

        println!();
        println!("PERFORMANCE DATA");
        println!("{appsignal_errors_table}");

        println!("To view more, open these magic links:");
        println!(
            "AppSignal: https://appsignal.com/mindvalley/sites/{}/exceptions?incident_marker=last",
            appsignal_app_id
        );
    }

    Ok(true)
}
