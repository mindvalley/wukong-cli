use super::PipelineData;
use crate::{
    config::CONFIG_FILE,
    error::CliError,
    graphql::{pipeline::pipelines_query::PipelinesQueryPipelines, QueryClientBuilder},
    loader::new_spinner_progress_bar,
    Config as CLIConfig, GlobalContext,
};
use tabled::{style::Style, Table};

pub async fn handle_list<'a>(context: GlobalContext) -> Result<bool, CliError<'a>> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Fetching pipelines list ...");

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

    if let Some(pipelines_data) = pipelines_data {
        let mut pipelines = Vec::new();

        for raw_pipeline in pipelines_data.into_iter().flatten() {
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

        let table = Table::new(pipelines).with(Style::modern()).to_string();

        println!("Pipeline list for application `{}`:", application);
        println!("{table}");
    }

    Ok(true)
}
