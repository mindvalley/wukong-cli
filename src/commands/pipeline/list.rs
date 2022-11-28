use super::PipelineData;
use crate::{
    config::CONFIG_FILE,
    error::CliError,
    graphql::{pipeline::pipelines_query::PipelinesQueryPipelines, QueryClientBuilder},
    loader::new_spinner_progress_bar,
    output::table::TableOutput,
    telemetry::{self, Command, TelemetryData},
    Config as CLIConfig, GlobalContext,
};
use tokio::time::{sleep, Duration};

pub async fn handle_list(context: GlobalContext) -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Fetching pipelines list ...");

    // sleep(Duration::from_millis(2000)).await;

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

    let pipelines_data = client
        .fetch_pipeline_list(&application)
        .await?
        .data
        .unwrap()
        .pipelines;

    progress_bar.finish_and_clear();

    // let telemetry_data = TelemetryData::new(
    //     // Some(Command {
    //     //     name: "pipeline list".to_string(),
    //     //     run_mode: telemetry::CommandRunMode::NonInteractive,
    //     // }),
    //     None,
    //     None,
    //     "api call".to_string(),
    // );

    // println!("{:?}", telemetry_data);
    // let handle = tokio::spawn(async move {
    //     // send telemetry event on command

    //     telemetry_data.send_event().await;
    //     println!("api call event sent finished");
    // });

    let mut pipelines = Vec::new();

    for raw_pipeline in pipelines_data {
        let pipeline = match raw_pipeline {
            PipelinesQueryPipelines::Job(p) => PipelineData {
                name: p.name,
                last_succeeded_at: p.last_succeeded_at,
                last_duration: p.last_duration,
                last_failed_at: p.last_failed_at,
            },
            PipelinesQueryPipelines::MultiBranchPipeline(p) => PipelineData {
                name: p.name,
                last_succeeded_at: p.last_succeeded_at,
                last_duration: p.last_duration,
                last_failed_at: p.last_failed_at,
            },
        };

        pipelines.push(pipeline);
    }

    let output = TableOutput {
        title: Some(format!("Pipeline list for application: `{}`:", application)),
        header: None,
        data: pipelines,
    };
    println!("{output}");
    // handle.await;

    Ok(true)
}
