use std::thread;
use std::time::Duration;

use crate::commands::application::config::ApplicationConfig;
use crate::config::{ApplicationWokflowConfig, Config};
use crate::error::ConfigError;
use crate::{error::WKCliError, loader::new_spinner};
use crossterm::style::Stylize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use owo_colors::OwoColorize;
use tokio::runtime::Runtime;
use tokio::task;

pub async fn handle_application_init() -> Result<bool, WKCliError> {
    println!("Welcome! Initializing per-repo configuration for your application.");
    let mut configs = ApplicationConfig::new()?;

    let mut config = Config::load_from_default_path()?;

    let name = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Application Name")
        .default("my-first-application".to_string())
        .interact_text()?;

    // TODO: Check for available workflows
    // Checking available Github Actions workflows.
    let available_github_actions_loader = new_spinner();
    available_github_actions_loader.set_message("Checking available Github Actions workflows...");

    let workflows = vec!["CI/CD", "CI/CD with Terraform"];

    available_github_actions_loader.finish_and_clear();

    let excluded_workflows = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Workflows to exclude")
        .items(&workflows[..])
        .interact()?;
    // [Leave blank to generate a random name]

    // println!("Continue to configure the namespace.\n");
    // Rollup strategy selection; rolling upgrade, blue-green, canary
    let rollup_strategy = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Rollup strategy")
        .items(&["Rolling Upgrade", "Blue/Green", "Canary"])
        .default(0)
        .interact()?;

    // Please select the baseline of replicas for your application. Enter to select the default (3) >
    let baseline_replicas = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Baseline Replicas")
        .default("3".to_string())
        .interact_text()?;

    let appsignal_environment = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            "(Optional)".bright_black(),
            "AppSignal Environment",
        ))
        .default("my-prod-app".to_string())
        .interact_text()?;

    // ? (Optional) Please put the name of the Honeycomb dataset if your application has tracing data. Leave it blank to disable Honeycomb integration > my-prod-ds
    let honeycomb_dataset = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            "(Optional)".bright_black(),
            "Honeycomb Dataset"
        ))
        .default("my-prod-ds".to_string())
        .interact_text()?;

    // ? (Optional) Please specify the Google Project of your CloudSQL instance(s). Leave blank to disable Google CloudSQL integration > my-prod-project
    let google_cloudsql_project = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            "(Optional)".bright_black(),
            "Google CloudSQL Project"
        ))
        .default("my-prod-project".to_string())
        .interact_text()?;

    // ? (Optional) Please select the addons for your application. Leave blank to ignore, select with Space key, press Enter when done.
    let addons = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{} {}", "(Optional)".bright_black(), "Addons"))
        .items(&["Honeycomb", "AppSignal", "Google CloudSQL"])
        .interact()?;

    // Finished configuring the prod namespace. Do you want to configure the staging namespace as well ? (Y/n) > Y
    let configure_staging_namespace = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Configure the staging namespace as well?")
        .default(true)
        .interact()?;

    // Please review your generated configurations below
    println!("Please review your generated configurations below:\n");

    // ? Do you want to write this configuration into your repo ? (Y/n) Y
    let agree_to_save = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to write this configuration into your repo?")
        .default(true)
        .interact()?;

    if agree_to_save {
        // Writing configuration... success ! You can check your configurations by open the .wukong.toml file in the root level of this repo.
        println!("{}", "Writing configuration...".green().bold().italic());

        // Now please commit and push this file to you main branch to active the configurations for your application.
        println!(
        "{}",
        "Now please commit and push this file to you main branch to active the configurations for your application."
            .green()
            .bold()
            .italic()
        );

        // let workflows = vec![ApplicationWokflowConfig {}];
        config.application = Some(ApplicationConfig {
            name,
            enable: true,
            workflows: ApplicationWokflowConfig {
                provider: "github_actions".to_string(),
                excluded_workflows: vec![],
                enable: true,
            },
            namespaces: vec![],
        });

        config.save_to_default_path()?;
    }

    Ok(true)
}

pub fn get_current_directory() -> Result<String, WKCliError> {
    let current_dir = std::env::current_dir()?;
    let path = current_dir
        .to_str()
        .expect("Unable to get current working directory");
    Ok(path.to_owned())
}
