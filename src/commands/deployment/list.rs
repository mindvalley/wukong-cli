use crate::{
    config::CONFIG_FILE,
    error::CliError,
    graphql::QueryClientBuilder,
    loader::new_spinner_progress_bar,
    output::table::TableOutput,
    output::table::{fmt_option_human_timestamp, fmt_option_string},
    Config as CLIConfig, GlobalContext,
};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

fn fmt_version(o: &str) -> String {
    fn capitalize_first_letter(o: &str) -> String {
        o[0..1].to_uppercase() + &o[1..]
    }
    // capitalize the first letter
    match capitalize_first_letter(o).as_str() {
        "Green" => "Green".green().bold().to_string(),
        "Blue" => "Blue".blue().bold().to_string(),
        version => version.bold().to_string(),
    }
}

fn fmt_enabled(o: &bool) -> String {
    match o {
        true => "Ready".to_string(),
        false => "Unavailable".to_string(),
    }
}

fn fmt_status(o: &Option<String>) -> String {
    match o {
        Some(status) => match status.as_str() {
            "SUCCEEDED" => "SUCCEEDED".green().to_string(),
            "FAILURE" => "FAILURE".red().to_string(),
            "TERMINAL" => "TERMINAL".yellow().to_string(),
            status => status.to_owned(),
        },
        None => "N/A".black().to_string(),
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
    #[tabled(
        rename = "Last deployment",
        display_with = "fmt_option_human_timestamp"
    )]
    last_deployed_at: Option<i64>,
    #[tabled(rename = "Status", display_with = "fmt_status")]
    status: Option<String>,
}

pub async fn handle_list(context: GlobalContext) -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Fetching cd pipeline list ... ");

    let application = match context.application {
        Some(application) => application,
        None => {
            let config_file = CONFIG_FILE
                .as_ref()
                .expect("Unable to identify user's home directory");

            CLIConfig::load(config_file).unwrap().core.application
        }
    };

    // Calling API ...
    let client = QueryClientBuilder::new()
        .with_access_token(context.id_token.unwrap())
        .build()?;

    let cd_pipelines_data = client
        .fetch_cd_pipeline_list(&application)
        .await?
        .data
        .unwrap()
        .cd_pipelines;

    let mut prod_pipelines = Vec::new();
    let mut staging_pipelines = Vec::new();

    for raw_cd_pipeline in cd_pipelines_data {
        let cd_pipeline = CdPipeline {
            name: raw_cd_pipeline.name,
            version: raw_cd_pipeline.version,
            enabled: raw_cd_pipeline.enabled,
            deployed_ref: raw_cd_pipeline.deployed_ref,
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

    progress_bar.finish_and_clear();

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

    println!("CD pipeline list for application `{}`:", application);

    println!("{prod_pipelines_table}");
    println!("{staging_pipelines_table}");

    Ok(true)
}
