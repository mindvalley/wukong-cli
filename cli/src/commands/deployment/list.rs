use crate::{
    commands::Context,
    config::Config,
    error::WKCliError,
    loader::new_spinner,
    output::{
        colored_println,
        table::{fmt_option_human_timestamp, fmt_option_string, TableOutput},
    },
    wukong_client::WKClient,
};
use serde::{Deserialize, Serialize};
use tabled::Tabled;
use wukong_telemetry::*;
use wukong_telemetry_macro::*;

fn fmt_version(o: &str) -> String {
    fn capitalize_first_letter(o: &str) -> String {
        o[0..1].to_uppercase() + &o[1..]
    }

    capitalize_first_letter(o)
}

fn fmt_enabled(o: &bool) -> String {
    match o {
        true => "Ready".to_string(),
        false => "Unavailable".to_string(),
    }
}

fn fmt_status(o: &Option<String>) -> String {
    match o {
        Some(status) => status.to_string(),
        None => "N/A".to_string(),
    }
}

#[derive(Tabled, Serialize, Deserialize, Debug)]
struct CdPipeline {
    #[tabled(skip)]
    name: String,
    #[tabled(rename = "Name", display_with = "fmt_version")]
    version: String,
    #[tabled(rename = "Enabled", display_with = "fmt_enabled")]
    enabled: bool,
    #[tabled(rename = "Deployed Ref", display_with = "fmt_option_string")]
    deployed_ref: Option<String>,
    #[tabled(rename = "Build Artifact", display_with = "fmt_option_string")]
    build_artifact: Option<String>,
    #[tabled(rename = "Triggered By", display_with = "fmt_option_string")]
    deployed_by: Option<String>,
    #[tabled(
        rename = "Last deployment",
        display_with = "fmt_option_human_timestamp"
    )]
    last_deployed_at: Option<i64>,
    #[tabled(rename = "Status", display_with = "fmt_status")]
    status: Option<String>,
}

#[wukong_telemetry(command_event = "deployment_list")]
pub async fn handle_list(context: Context) -> Result<bool, WKCliError> {
    let fetch_loader = new_spinner();
    fetch_loader.set_message("Fetching cd pipeline list ... ");

    let config = Config::load_from_default_path()?;
    let wk_client = WKClient::new(&config);

    let cd_pipelines_data = wk_client
        .fetch_cd_pipelines(&context.current_application)
        .await?
        .cd_pipelines;

    let mut prod_pipelines = Vec::new();
    let mut staging_pipelines = Vec::new();

    for raw_cd_pipeline in cd_pipelines_data {
        let cd_pipeline = CdPipeline {
            name: raw_cd_pipeline.name,
            version: raw_cd_pipeline.version,
            enabled: raw_cd_pipeline.enabled,
            deployed_ref: raw_cd_pipeline
                .deployed_ref
                .map(|deployed_ref| deployed_ref[..7].to_string()),
            build_artifact: raw_cd_pipeline.build_artifact,
            deployed_by: raw_cd_pipeline.deployed_by,
            last_deployed_at: raw_cd_pipeline.last_deployment,
            status: raw_cd_pipeline.status,
        };

        match raw_cd_pipeline.environment.as_str() {
            "prod" => {
                prod_pipelines.push(cd_pipeline);
            }
            "staging" => {
                staging_pipelines.push(cd_pipeline);
            }
            _ => {}
        };
    }

    fetch_loader.finish_and_clear();

    let prod_pipelines_table = TableOutput {
        title: None,
        header: Some("Prod".to_string()),
        data: prod_pipelines,
    };
    let staging_pipelines_table = TableOutput {
        title: None,
        header: Some("Staging".to_string()),
        data: staging_pipelines,
    };

    colored_println!(
        "CD pipeline list for application {}:",
        context.current_application
    );
    colored_println!("{}", prod_pipelines_table);
    colored_println!("{}", staging_pipelines_table);

    Ok(true)
}
