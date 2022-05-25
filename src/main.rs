#![forbid(unsafe_code)]

mod command_group;

use clap::Parser;
use command_group::{pipeline::PipelineSubcommand, CommandGroup};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use std::fmt::Display;
use std::process::Command;
use std::str;
use std::thread;
use std::time::{Duration, Instant};
use tabled::{Table, Tabled};

/// A Swiss-army Knife CLI For Mindvalley Developers
#[derive(Debug, Parser)]
#[clap(version)]
struct Cli {
    #[clap(subcommand)]
    command_group: CommandGroup,
}

fn fmt_option<T: Display>(o: &Option<T>) -> String {
    match o {
        Some(s) => format!("{}", s),
        None => "N/A".to_string(),
    }
}

#[derive(Tabled)]
struct PipelineData {
    name: &'static str,
    #[tabled(display_with = "fmt_option")]
    last_succeed_at: Option<&'static str>,
    #[tabled(display_with = "fmt_option")]
    last_failed_at: Option<&'static str>,
    #[tabled(display_with = "fmt_option")]
    last_duration: Option<&'static str>,
}

#[derive(Tabled)]
struct PipelineBranch {
    name: &'static str,
    #[tabled(display_with = "fmt_option")]
    last_succeed_at: Option<&'static str>,
    #[tabled(display_with = "fmt_option")]
    last_failed_at: Option<&'static str>,
    #[tabled(display_with = "fmt_option")]
    last_duration: Option<&'static str>,
}

#[derive(Tabled)]
struct PipelinePullRequest {
    name: &'static str,
    #[tabled(display_with = "fmt_option")]
    last_succeed_at: Option<&'static str>,
    #[tabled(display_with = "fmt_option")]
    last_failed_at: Option<&'static str>,
    #[tabled(display_with = "fmt_option")]
    last_duration: Option<&'static str>,
}

fn main() {
    let cli = Cli::parse();

    match cli.command_group {
        CommandGroup::Pipeline(pipeline) => match pipeline.subcommand {
            PipelineSubcommand::List => {
                let started = Instant::now();
                let deps = 1234;
                let progress_bar = ProgressBar::new(deps);
                progress_bar.set_style(ProgressStyle::default_spinner());
                println!("Fetching pipelines list ...");
                for _ in 0..deps {
                    progress_bar.inc(1);
                    thread::sleep(Duration::from_millis(3));
                }
                progress_bar.finish_and_clear();

                let pipelines = vec![
                    PipelineData {
                        name: "mv-platform-ci",
                        last_succeed_at: None,
                        last_failed_at: None,
                        last_duration: None,
                    },
                    PipelineData {
                        name: "mv-platform-production-master-branch-build",
                        last_succeed_at: Some("1 hr 20 min"),
                        last_failed_at: Some("6 days 19 hr"),
                        last_duration: Some("14 min"),
                    },
                    PipelineData {
                        name: "mv-platform-staging-build",
                        last_succeed_at: Some("1 min 2 sec"),
                        last_failed_at: None,
                        last_duration: Some("2.3 sec"),
                    },
                    PipelineData {
                        name: "mv-platform-staging-developer-build",
                        last_succeed_at: Some("18 hr"),
                        last_failed_at: None,
                        last_duration: Some("34 sec"),
                    },
                ];

                let table = Table::new(pipelines).to_string();
                println!("{table}");

                println!("Fetch in {}.", HumanDuration(started.elapsed()));

                todo!()
            }
            PipelineSubcommand::Describe { name } => {
                let deps = 1234;
                let progress_bar = ProgressBar::new(deps);
                progress_bar.set_style(ProgressStyle::default_spinner());
                println!("Fetching pipeline data ...");
                for _ in 0..deps {
                    progress_bar.inc(1);
                    thread::sleep(Duration::from_millis(3));
                }
                progress_bar.finish_and_clear();

                println!("Pipeline \"{}\":\n", name);

                let branches = vec![PipelineBranch {
                    name: "master",
                    last_succeed_at: Some("1 hr 20 min"),
                    last_failed_at: Some("6 days 19 hr"),
                    last_duration: Some("14 min"),
                }];

                let table = Table::new(branches).to_string();

                println!("Branches");
                println!("{table}");

                let pull_requests = vec![
                    PipelinePullRequest {
                        name: "PR-3985",
                        last_succeed_at: None,
                        last_failed_at: None,
                        last_duration: None,
                    },
                    PipelinePullRequest {
                        name: "PR-4037",
                        last_succeed_at: Some("1 hr 20 min"),
                        last_failed_at: Some("6 days 19 hr"),
                        last_duration: Some("14 min"),
                    },
                    PipelinePullRequest {
                        name: "PR-4086",
                        last_succeed_at: Some("1 min 2 sec"),
                        last_failed_at: None,
                        last_duration: Some("2.3 sec"),
                    },
                    PipelinePullRequest {
                        name: "PR-4096",
                        last_succeed_at: Some("1 min 2 sec"),
                        last_failed_at: None,
                        last_duration: Some("4.3 sec"),
                    },
                    PipelinePullRequest {
                        name: "PR-4113",
                        last_succeed_at: Some("18 hr"),
                        last_failed_at: None,
                        last_duration: Some("34 sec"),
                    },
                ];

                let table = Table::new(pull_requests).to_string();

                println!("Pull Requests");
                println!("{table}");

                todo!()
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
        },
    }
}

#[cfg(test)]
mod test {
    use crate::Cli;

    #[test]
    fn verify_app() {
        use clap::CommandFactory;

        Cli::command().debug_assert()
    }
}
