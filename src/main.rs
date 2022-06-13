#![forbid(unsafe_code)]

mod command_group;
mod config;

use clap::Parser;
use command_group::{pipeline::PipelineSubcommand, CommandGroup};
use config::{Config, CONFIG_FILE};
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use std::{
    fmt::Display,
    process::Command,
    str, thread,
    time::{Duration, Instant},
};
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
        CommandGroup::Config(config) => {
            config.perform_action();
        }
        CommandGroup::Init => {
            println!("Welcome! This command will take you through the configuration of Wukong.\n");

            // if Confirm::with_theme(&ColorfulTheme::default())
            //     .with_prompt("Do you really want to continue?")
            //     .wait_for_newline(true)
            //     .interact()
            //     .unwrap()
            // {
            //     println!("Looks like you want to continue");
            // } else {
            //     println!("nevermind then :(");
            // }

            let login_selections = &["junkai.gan@mindvalley.com", "Log in with a new account"];

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose the account you would like to use to perform operations for this configuration:")
                .default(0)
                .items(&login_selections[..])
                .interact()
                .unwrap();

            println!("You are logged in as: [{}].\n", login_selections[selection]);

            let application_selections = &[
                "mv-prod-applications-hub",
                "mv-prod-linode",
                "mv-prod-platform-osiris",
                "mv-stg-applications-hub",
                "mv-stg-dev-platform-osiris",
                "mv-stg-linode",
            ];

            let application_selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Please select the application")
                .default(0)
                .items(&application_selections[..])
                .interact()
                .unwrap();

            println!(
                "Your current application has been set to: [{}].",
                &application_selections[application_selection]
            );

            println!(
                r#"
Your Wukong CLI is configured and ready to use!

* Commands that require authentication will use junkai.gan@mindvalley.com by default
* Commands will reference application `mv-prod-applications-hub` by default
Run `wukong config help` to learn how to change individual settings

Some things to try next:

* Run `wukong --help` to see the wukong command groups you can interact with. And run `wukong COMMAND help` to get help on any wukong command.
                     "#
            );

            if let Some(ref config_file) = *CONFIG_FILE {
                let mut config = Config::default();
                config.core.application = application_selections[application_selection].to_string();
                config.save(config_file).expect("Config file save failed");
            }
        }
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
