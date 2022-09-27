use crate::{
    config::CONFIG_FILE, error::CliError, graphql::QueryClientBuilder, Config as CLIConfig,
    GlobalContext,
};
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use chrono_humanize::HumanTime;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use tabled::{style::Style, Panel, Table, Tabled};

fn fmt_option_timestamp(o: &Option<i64>) -> String {
    match o {
        Some(s) => fmt_timestamp(s),
        None => "N/A".to_string(),
    }
}

fn fmt_option_string(o: &Option<String>) -> String {
    match o {
        Some(s) => s.to_string(),
        None => "N/A".to_string(),
    }
}

fn fmt_timestamp(o: &i64) -> String {
    let naive = NaiveDateTime::from_timestamp_opt(o / 1000, (o % 1000) as u32 * 1_000_000).unwrap();
    let dt = DateTime::<Utc>::from_utc(naive, Utc).with_timezone(&Local);
    format!("{}", HumanTime::from(dt))
}

fn fmt_version(o: &String) -> String {
    fn capitalize_first_letter(o: &String) -> String {
        o[0..1].to_uppercase() + &o[1..]
    }
    // capitalize the first letter
    match capitalize_first_letter(o).as_str() {
        "Green" => "Green".green().bold().to_string(),
        "Blue" => "Blue".blue().bold().to_string(),
        version => version.to_string(),
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
    #[tabled(rename = "Last deployment", display_with = "fmt_option_timestamp")]
    last_deployed_at: Option<i64>,
    #[tabled(rename = "Status", display_with = "fmt_status")]
    status: Option<String>,
}

pub async fn handle_list<'a>(context: GlobalContext) -> Result<bool, CliError<'a>> {
    let steps = 1024;
    let progress_bar = ProgressBar::new(steps);
    progress_bar.set_style(ProgressStyle::default_spinner());
    println!("Fetching cd pipelines list ...\n");

    progress_bar.inc(1);

    let config_file = CONFIG_FILE
        .as_ref()
        .expect("Unable to identify user's home directory");

    let application = match context.application {
        Some(application) => application,
        None => CLIConfig::load(config_file).unwrap().core.application,
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

    if let Some(cd_pipelines_data) = cd_pipelines_data {
        let mut prod_pipelines = Vec::new();
        let mut staging_pipelines = Vec::new();

        for raw_cd_pipeline in cd_pipelines_data.into_iter().flatten() {
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

        let prod_pipelines_table = Table::new(prod_pipelines)
            .with(Panel("Prod", 0))
            .with(Style::modern())
            .to_string();
        let staging_pipelines_table = Table::new(staging_pipelines)
            .with(Panel("Staging", 0))
            .with(Style::modern())
            .to_string();

        println!("CD pipeline list for application `{}`:", application);

        println!("{prod_pipelines_table}");
        println!("{staging_pipelines_table}");
    }

    Ok(true)
}
