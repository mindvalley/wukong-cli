use super::{JobBuild, PipelineBranch, PipelinePullRequest};
use crate::{
    error::CliError,
    graphql::{pipeline::pipeline_query::PipelineQueryPipeline, QueryClientBuilder},
    loader::new_spinner_progress_bar,
    output::table::TableOutput,
    GlobalContext,
};

pub async fn handle_describe(context: GlobalContext, name: &str) -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Fetching pipeline data ...");

    // Calling API ...
    let client = QueryClientBuilder::new()
        .with_access_token(context.id_token.unwrap())
        .build()?;

    let pipeline_resp = client
        .fetch_pipeline(name)
        .await?
        .data
        // .ok_or(anyhow::anyhow!("Error"))?
        .unwrap()
        .pipeline;

    progress_bar.finish_and_clear();

    if let Some(pipeline_data) = pipeline_resp {
        match pipeline_data {
            PipelineQueryPipeline::Job(p) => {
                if let Some(builds) = p.builds {
                    println!("Changes: ");

                    for build in builds.into_iter().flatten() {
                        let build_data = JobBuild {
                            build_number: build.build_number,
                            timestamp: build.timestamp,
                            // wait_duration: build.wait_duration,
                            // build_duration: build.build_duration,
                            // total_duration: build.total_duration,
                            commit_id: build.commits.first().map(|commit| commit.id.clone()),
                            commit_msg: build
                                .commits
                                .first()
                                .map(|commit| commit.message_headline.clone()),
                            // commit_id: build.commit_id.clone(),
                            // commit_msg: build.commit_msg.clone(),
                            // commit_author: build.commit_author.clone(),
                            result: build.result.clone(),
                        };

                        println!("{build_data}");
                    }
                }
            }
            PipelineQueryPipeline::MultiBranchPipeline(p) => {
                let multi_branch_pipeline_resp = client
                    .fetch_multi_branch_pipeline(&p.name)
                    .await?
                    .data
                    .unwrap()
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
                    println!("{output}");

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
                    println!("{output}");
                }
            }
        }
    }
    Ok(true)
}
