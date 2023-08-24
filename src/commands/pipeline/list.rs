use super::PipelineData;
use crate::{
    commands::Context,
    error::CliError,
    graphql::{
        github_pipeline::github_pipelines_query::GithubPipelinesQueryGithubPipelines,
        pipeline::pipelines_query::PipelinesQueryPipelines, QueryClient,
    },
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
    let mut client = QueryClient::from_default_config()?;

    let mut pipelines = Vec::new();

    let github_pipelines = get_github_pipelines(&mut client, &application).await?;

    if github_pipelines.is_empty() {
        let jenkins_pipelines = get_jenkins_pipelines(&mut client, &application).await?;
        pipelines.extend(jenkins_pipelines);
    } else {
        pipelines.extend(github_pipelines);
    }

    let output = TableOutput {
        title: Some(format!("Pipeline list for application {application}:")),
        header: None,
        data: pipelines,
    };

    colored_println!("{}", output);

    Ok(true)
}

async fn get_github_pipelines(
    client: &mut QueryClient,
    application: &str,
) -> Result<Vec<PipelineData>, CliError> {
    let github_pipelines = client
        .fetch_github_pipeline_list(application)
        .await?
        .data
        .unwrap()
        .github_pipelines;

    let mut pipelines = Vec::new();

    match github_pipelines {
        None => (),
        Some(github_pipelines) => {
            for github_pipeline in github_pipelines {
                let pipeline_data = match github_pipeline {
                    GithubPipelinesQueryGithubPipelines::Job(job) => PipelineData {
                        name: job.name,
                        last_succeeded_at: job.last_succeeded_at,
                        last_duration: job.last_duration,
                        last_failed_at: job.last_failed_at,
                    },
                    GithubPipelinesQueryGithubPipelines::MultiBranchPipeline(
                        multi_branch_pipeline,
                    ) => PipelineData {
                        name: multi_branch_pipeline.name,
                        last_succeeded_at: multi_branch_pipeline.last_succeeded_at,
                        last_duration: multi_branch_pipeline.last_duration,
                        last_failed_at: multi_branch_pipeline.last_failed_at,
                    },
                };

                pipelines.push(pipeline_data);
            }
        }
    };

    Ok(pipelines)
}

async fn get_jenkins_pipelines(
    client: &mut QueryClient,
    application: &str,
) -> Result<Vec<PipelineData>, CliError> {
    let jenkins_pipelines = client
        .fetch_pipeline_list(application)
        .await?
        .data
        .unwrap()
        .pipelines;

    let mut pipelines = Vec::new();

    for jenkins_pipeline in jenkins_pipelines {
        let pipeline_data = match jenkins_pipeline {
            PipelinesQueryPipelines::Job(job) => PipelineData {
                name: job.name,
                last_succeeded_at: job.last_succeeded_at,
                last_duration: job.last_duration,
                last_failed_at: job.last_failed_at,
            },
            PipelinesQueryPipelines::MultiBranchPipeline(multi_branch_pipeline) => PipelineData {
                name: multi_branch_pipeline.name,
                last_succeeded_at: multi_branch_pipeline.last_succeeded_at,
                last_duration: multi_branch_pipeline.last_duration,
                last_failed_at: multi_branch_pipeline.last_failed_at,
            },
        };

        pipelines.push(pipeline_data);
    }

    Ok(pipelines)
}
