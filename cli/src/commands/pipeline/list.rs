use crate::{
    commands::{pipeline::PipelineData, Context},
    config::Config,
    error::WKCliError,
    loader::new_spinner,
    output::{colored_println, table::TableOutput},
};
use wukong_sdk::{graphql::pipelines_query::PipelinesQueryPipelines, WKClient, WKConfig};

// #[wukong_telemetry(command_event = "pipeline_list")]
pub async fn handle_list(context: Context) -> Result<bool, WKCliError> {
    let fetch_loader = new_spinner();
    fetch_loader.set_message("Fetching pipelines list ...");

    let config = Config::load_from_default_path()?;
    // Calling API ...
    let wk_client = WKClient::new(WKConfig {
        api_url: config.core.wukong_api_url,
        access_token: config.auth.map(|auth| auth.id_token),
    });

    let pipelines_data = wk_client
        .fetch_pipelines(&context.current_application)
        .await?
        .pipelines;

    fetch_loader.finish_and_clear();

    let pipelines: Vec<PipelineData> = pipelines_data
        .iter()
        .map(|raw_pipeline| match raw_pipeline {
            PipelinesQueryPipelines::Job(p) => PipelineData {
                name: p.name.clone(),
                last_succeeded_at: p.last_succeeded_at,
                last_duration: p.last_duration,
                last_failed_at: p.last_failed_at,
            },
            PipelinesQueryPipelines::MultiBranchPipeline(p) => PipelineData {
                name: p.name.clone(),
                last_succeeded_at: p.last_succeeded_at,
                last_duration: p.last_duration,
                last_failed_at: p.last_failed_at,
            },
        })
        .collect();

    let output = TableOutput {
        title: Some(format!(
            "Pipeline list for application {}:",
            context.current_application
        )),
        header: None,
        data: pipelines,
    };

    colored_println!("{}", output);

    Ok(true)
}
