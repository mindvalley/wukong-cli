use wukong_sdk::graphql::pipeline_query::PipelineQueryPipeline;

use crate::{
    commands::{
        pipeline::{JobBuild, PipelineBranch, PipelinePullRequest},
        Context,
    },
    config::Config,
    error::WKCliError,
    loader::new_spinner,
    output::{colored_println, table::TableOutput},
    wukong_client::WKClient,
};
use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[wukong_telemetry(command_event = "pipeline_describe")]
pub async fn handle_describe(context: Context, name: &str) -> Result<bool, WKCliError> {
    let fetch_loader = new_spinner();
    fetch_loader.set_message("Fetching pipeline data ...");

    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    let pipeline = wk_client.fetch_pipeline(name).await?.pipeline;

    fetch_loader.finish_and_clear();

    if pipeline.is_none() {
        colored_println!("Pipeline not found.");
        return Ok(false);
    }

    match pipeline.unwrap() {
        PipelineQueryPipeline::Job(p) => {
            if let Some(builds) = p.builds {
                println!("Changes: ");

                for build in builds.into_iter().flatten() {
                    let build_data = JobBuild {
                        build_number: build.build_number,
                        timestamp: build.timestamp,
                        commit_id: build.commits.first().map(|commit| commit.id.clone()),
                        commit_msg: build
                            .commits
                            .first()
                            .map(|commit| commit.message_headline.clone()),
                        result: build.result.clone(),
                    };

                    colored_println!("{}", build_data);
                }
            }
        }
        PipelineQueryPipeline::MultiBranchPipeline(p) => {
            let multi_branch_pipeline_resp = wk_client
                .fetch_multi_branch_pipeline(&p.name)
                .await?
                .multi_branch_pipeline;

            if let Some(multi_branch_pipeline) = multi_branch_pipeline_resp {
                let mut branches = Vec::new();
                for branch in multi_branch_pipeline.branches {
                    branches.push(PipelineBranch {
                        name: branch.name,
                        last_succeed_at: branch.last_succeeded_at,
                        last_failed_at: branch.last_failed_at,
                        last_duration: branch.last_duration,
                    });
                }

                let output = TableOutput {
                    title: Some("Branches:".to_string()),
                    header: None,
                    data: branches,
                };
                colored_println!("{}", output);

                let mut pull_requests = Vec::new();
                for pull_request in multi_branch_pipeline.pull_requests {
                    pull_requests.push(PipelinePullRequest {
                        name: pull_request.name,
                        last_succeed_at: pull_request.last_succeeded_at,
                        last_failed_at: pull_request.last_failed_at,
                        last_duration: pull_request.last_duration,
                    });
                }

                let output = TableOutput {
                    title: Some("Pull Requests:".to_string()),
                    header: None,
                    data: pull_requests,
                };
                colored_println!("{}", output);
            }
        }
    }

    // if let Some(pipeline_data) = pipeline_resp {}
    Ok(true)
}
