use super::PipelineData;
use crate::{
    commands::Context,
    error::CliError,
    graphql::{pipeline::pipelines_query::PipelinesQueryPipelines, QueryClientBuilder},
    loader::new_spinner_progress_bar,
    output::{colored_println, table::TableOutput},
    telemetry::{self, TelemetryData, TelemetryEvent},
};
use wukong_telemetry_macro::wukong_telemetry;

#[wukong_telemetry(command_event = "pipeline_list")]
pub async fn handle_list(context: Context) -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Fetching pipelines list ...");

    // SAFETY: This is safe to unwrap because we know that `application` is not None.
    let application = context.state.application.unwrap();

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

    let pipelines_data = client
        .fetch_pipeline_list(&application)
        .await?
        .data
        .unwrap()
        .pipelines;

    progress_bar.finish_and_clear();

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
        title: Some(format!("Pipeline list for application {application}:")),
        header: None,
        data: pipelines,
    };
    // let token = crate::output::tokenizer::OutputTokenizer::tokenize(output.to_string());
    // dbg!(token);
    colored_println!("{}", output);

    Ok(true)
}
