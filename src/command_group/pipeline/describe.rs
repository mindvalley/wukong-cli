use super::{JobBuild, PipelineBranch, PipelinePullRequest};
use crate::{
    error::CliError,
    graphql::pipeline::{
        pipeline_query::PipelineQueryPipeline, MultiBranchPipelineQuery, PipelineQuery,
    },
};
use indicatif::{ProgressBar, ProgressStyle};
use tabled::Table;

pub async fn handle_describe<'a>(name: &str) -> Result<bool, CliError<'a>> {
    let deps = 1234;
    let progress_bar = ProgressBar::new(deps);
    progress_bar.set_style(ProgressStyle::default_spinner());
    println!("Fetching pipeline data ...");

    // Calling API ...
    let pipeline_resp = PipelineQuery::fetch(name)
        .await?
        .data
        // .ok_or(anyhow::anyhow!("Error"))?
        .unwrap()
        .pipeline;

    if let Some(pipeline_data) = pipeline_resp {
        match pipeline_data {
            PipelineQueryPipeline::Job(p) => {
                if let Some(builds) = p.builds {
                    println!("Changes: ");

                    for build in builds.iter() {
                        if let Some(build) = build {
                            let build_data = JobBuild {
                                build_number: build.build_number,
                                timestamp: build.timestamp,
                                // wait_duration: build.wait_duration,
                                // build_duration: build.build_duration,
                                // total_duration: build.total_duration,
                                commit_id: build.commit_id.clone(),
                                commit_msg: build.commit_msg.clone(),
                                // commit_author: build.commit_author.clone(),
                                result: build.result.clone(),
                            };

                            println!("{build_data}");
                        }
                    }
                }
            }
            PipelineQueryPipeline::MultiBranchPipeline(p) => {
                let multi_branch_pipeline_resp = MultiBranchPipelineQuery::fetch(&p.name)
                    .await?
                    .data
                    // .ok_or(anyhow::anyhow!("Error"))?
                    .unwrap()
                    .multi_branch_pipeline;

                if let Some(multi_branch_pipeline) = multi_branch_pipeline_resp {
                    if let Some(pipeline_branches) = multi_branch_pipeline.branches {
                        let mut branches = Vec::new();
                        for pipeline_branch in pipeline_branches {
                            if let Some(branch) = pipeline_branch {
                                branches.push(PipelineBranch {
                                    name: branch.name,
                                    last_succeed_at: branch.last_succeeded_at,
                                    last_failed_at: branch.last_failed_at,
                                    last_duration: branch.last_duration,
                                });
                            }
                        }

                        let table = Table::new(branches).to_string();

                        println!("Branches:");
                        println!("{table}");
                    }
                    if let Some(pipeline_pull_requests) = multi_branch_pipeline.pull_requests {
                        let mut pull_requests = Vec::new();
                        for pipeline_pull_request in pipeline_pull_requests {
                            if let Some(pull_request) = pipeline_pull_request {
                                pull_requests.push(PipelinePullRequest {
                                    name: pull_request.name,
                                    last_succeed_at: pull_request.last_succeeded_at,
                                    last_failed_at: pull_request.last_failed_at,
                                    last_duration: pull_request.last_duration,
                                });
                            }
                        }

                        let table = Table::new(pull_requests).to_string();

                        println!("Pull Requests:");
                        println!("{table}");
                    }
                }
            }
        }

        progress_bar.finish_and_clear();
    }
    Ok(true)
}
