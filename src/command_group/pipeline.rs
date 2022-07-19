use crate::{
    error::CliError,
    graphql::pipeline::{MultiBranchPipelineQuery, PipelineQuery, PipelinesQuery},
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use clap::{Args, Subcommand};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, process::Command, str, time::Instant};
use tabled::{Table, Tabled};

fn fmt_option<T: Display>(o: &Option<T>) -> String {
    match o {
        Some(s) => format!("{}", s),
        None => "N/A".to_string(),
    }
}

#[derive(Tabled, Serialize, Deserialize, Debug)]
struct PipelineData {
    name: String,
    #[tabled(display_with = "fmt_option")]
    last_succeeded_at: Option<i64>,
    #[tabled(display_with = "fmt_option")]
    last_failed_at: Option<i64>,
    #[tabled(display_with = "fmt_option")]
    last_duration: Option<i64>,
}

#[derive(Tabled)]
struct PipelineBranch {
    name: String,
    #[tabled(display_with = "fmt_option")]
    last_succeed_at: Option<i64>,
    #[tabled(display_with = "fmt_option")]
    last_failed_at: Option<i64>,
    #[tabled(display_with = "fmt_option")]
    last_duration: Option<i64>,
}

#[derive(Tabled)]
struct PipelinePullRequest {
    name: String,
    #[tabled(display_with = "fmt_option")]
    last_succeed_at: Option<i64>,
    #[tabled(display_with = "fmt_option")]
    last_failed_at: Option<i64>,
    #[tabled(display_with = "fmt_option")]
    last_duration: Option<i64>,
}

struct JobBuild {
    build_number: i64,
    timestamp: i64,
    wait_duration: Option<i64>,
    build_duration: Option<i64>,
    total_duration: Option<i64>,
    commit_id: Option<String>,
    commit_msg: Option<String>,
    commit_author: Option<String>,
    result: String,
}

impl Display for JobBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let naive = NaiveDateTime::from_timestamp_opt(
            self.timestamp / 1000,
            (self.timestamp % 1000) as u32 * 1_000_000,
        )
        .unwrap();
        let started_at = DateTime::<Utc>::from_utc(naive, Utc);

        let commit_msg = match self.commit_msg {
            Some(ref msg) => msg,
            None => "No commit message",
        };
        let commit_id = match self.commit_id {
            Some(ref id) => &id[0..7],
            None => "",
        };

        write!(
            f,
            "[{}] #{} ({})\n{} (commit: {})\n",
            self.result,
            self.build_number,
            started_at.to_rfc2822(),
            commit_msg,
            commit_id,
        )
    }
}

#[derive(Debug, Args)]
pub struct Pipeline {
    #[clap(subcommand)]
    pub subcommand: PipelineSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum PipelineSubcommand {
    /// List the current pipelines of the application
    List,
    /// Show the details of a pipeline
    Describe {
        /// The pipeline name
        name: String,
    },
    /// Show the build status and (possible) errors on branch ci pipeline
    CiStatus,
}

impl Pipeline {
    pub async fn perform_action<'a>(&self) -> Result<bool, CliError<'a>> {
        match &self.subcommand {
            PipelineSubcommand::List => {
                let started = Instant::now();
                let progress_bar = ProgressBar::new(1234);
                progress_bar.set_style(ProgressStyle::default_spinner());
                println!("Fetching pipelines list ...\n");

                progress_bar.inc(1);

                // Calling API ...
                let pipelines_data = PipelinesQuery::fetch()
                    .await?
                    .data
                    .unwrap()
                    // .ok_or(anyhow::anyhow!("Error"))?
                    .pipelines;

                progress_bar.finish_and_clear();

                if let Some(pipelines_data) = pipelines_data {
                    let mut pipelines = Vec::new();

                    for raw_pipeline in pipelines_data {
                        if let Some(raw_pipeline) = raw_pipeline {
                            let pipeline = match raw_pipeline {
                                crate::graphql::pipeline::pipelines_query::PipelinesQueryPipelines::Job(p) => {
                                    PipelineData {
                                        name: p.name,
                                        last_succeeded_at: p.last_succeeded_at,
                                        last_duration: p.last_duration,
                                        last_failed_at: p.last_failed_at,
                                    }
                                },
                                crate::graphql::pipeline::pipelines_query::PipelinesQueryPipelines::MultiBranchPipeline(p) => {
                                    PipelineData {
                                        name: p.name,
                                        last_succeeded_at: p.last_succeeded_at,
                                        last_duration: p.last_duration,
                                        last_failed_at: p.last_failed_at,
                                    }
                                }
                            };

                            pipelines.push(pipeline);
                        }
                    }

                    let table = Table::new(pipelines).to_string();
                    println!("{table}");
                }
                println!("Fetch in {}.", HumanDuration(started.elapsed()));

                Ok(true)
            }
            PipelineSubcommand::Describe { name } => {
                let deps = 1234;
                let progress_bar = ProgressBar::new(deps);
                progress_bar.set_style(ProgressStyle::default_spinner());
                println!("Fetching pipeline data ...");

                // Calling API ...
                let pipeline_resp = PipelineQuery::fetch(name.to_string())
                    .await?
                    .data
                    // .ok_or(anyhow::anyhow!("Error"))?
                    .unwrap()
                    .pipeline;

                if let Some(pipeline_data) = pipeline_resp {
                    match pipeline_data {
                        crate::graphql::pipeline::pipeline_query::PipelineQueryPipeline::Job(p) => {
                            if let Some(builds) = p.builds {
                                println!("Changes: ");

                                for build in builds.iter() {
                                    if let Some(build) = build {
                                        let build_data = JobBuild {
                                            build_number: build.build_number,
                                            timestamp: build.timestamp,
                                            wait_duration: build.wait_duration,
                                            build_duration: build.build_duration,
                                            total_duration: build.total_duration,
                                            commit_id: build.commit_id.clone(),
                                            commit_msg: build.commit_msg.clone(),
                                            commit_author: build.commit_author.clone(),
                                            result: build.result.clone(),
                                        };

                                        println!("{build_data}");
                                    }
                                }
                            }
                        },
                        crate::graphql::pipeline::pipeline_query::PipelineQueryPipeline::MultiBranchPipeline(p) => {
                            let multi_branch_pipeline_resp = MultiBranchPipelineQuery::fetch(p.name)
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

                                    println!("Branches");
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
            PipelineSubcommand::CiStatus => {
                println!("CI Status");

                let output = Command::new("git")
                    .args(["branch", "--show-current"])
                    .output()
                    .expect("failed to execute process");

                println!("{:?}", str::from_utf8(&output.stdout));

                todo!()
            }
        }
    }
}
