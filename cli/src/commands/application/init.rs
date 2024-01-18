use std::thread;
use std::time::Duration;

use crate::commands::application::config::{
    ApplicationConfig, ApplicationConfigs, ApplicationNamespaceAppsignalConfig,
    ApplicationNamespaceBuildConfig, ApplicationNamespaceCloudsqlConfig,
    ApplicationNamespaceConfig, ApplicationNamespaceDeliveryConfig,
    ApplicationNamespaceHoneycombConfig, ApplicationWorkflowConfig,
};
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
    let mut application_configs = ApplicationConfigs::new()?;

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

    let excluded_workflows: Vec<String> = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Workflows to exclude")
        .items(&workflows[..])
        .interact()
        .expect("Unable to get user input")
        .iter()
        .map(|&index| workflows[index].to_string())
        .collect();
    // [Leave blank to generate a random name]

    let rollup_strategy_options = ["Rolling Upgrade", "Blue/Green", "Canary"];

    // Rollup strategy selection; rolling upgrade, blue-green, canary
    let rollout_strategy: String = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Rollup strategy")
        .items(&rollup_strategy_options)
        .default(0)
        .interact()
        .map(|selected_index| rollup_strategy_options[selected_index].to_string())
        .expect("Unable to get user input");

    // Please select the baseline of replicas for your application. Enter to select the default (3) >
    let base_replica = Input::<u32>::with_theme(&ColorfulTheme::default())
        .with_prompt("Baseline Replicas")
        .default(3)
        .interact_text()
        .expect("Unable to get user input");

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
    let cloudsql_project_id = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            "(Optional)".bright_black(),
            "Google CloudSQL Project"
        ))
        .default("my-prod-project".to_string())
        .interact_text()?;

    // ? (Optional) Please select the addons for your application. Leave blank to ignore, select with Space key, press Enter when done.
    let _addons = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{} {}", "(Optional)".bright_black(), "Addons"))
        .items(&["Honeycomb", "AppSignal", "Google CloudSQL"])
        .interact()?;

    // // Finished configuring the prod namespace. Do you want to configure the staging namespace as well ? (Y/n) > Y
    // let configure_staging_namespace = Confirm::with_theme(&ColorfulTheme::default())
    //     .with_prompt("Configure the staging namespace as well?")
    //     .default(true)
    //     .interact()?;

    // Please review your generated configurations below
    println!("\nPlease review your generated configurations below:");

    // ? Do you want to write this configuration into your repo ? (Y/n) Y
    let agree_to_save = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to write this configuration into your repo?")
        .default(true)
        .interact()?;

    if agree_to_save {
        // Writing configuration... success ! You can check your configurations by open the .wukong.toml file in the root level of this repo.
        println!("{}", "Writing configuration...\n".green().bold().italic());

        let namespaces = ApplicationNamespaceConfig {
            namespace_type: "prod".to_string(),
            build: Some(ApplicationNamespaceBuildConfig {
                build_workflow: "CI/CD".to_string(),
            }),
            delivery: Some(ApplicationNamespaceDeliveryConfig {
                target: "GKE".to_string(),
                base_replica,
                rollout_strategy,
            }),
            appsignal: Some(ApplicationNamespaceAppsignalConfig {
                enable: true,
                environment: appsignal_environment,
                default_namespace: "prod".to_string(),
            }),
            honeycomb: Some(ApplicationNamespaceHoneycombConfig {
                enable: true,
                dataset: honeycomb_dataset,
            }),
            cloudsql: Some(ApplicationNamespaceCloudsqlConfig {
                enable: true,
                project_id: cloudsql_project_id,
            }),
        };

        let workflows = ApplicationWorkflowConfig {
            provider: "github_actions".to_string(),
            excluded_workflows,
            enable: true,
        };

        application_configs.application = Some(ApplicationConfig {
            name,
            enable: true,
            workflows: Some(workflows),
            namespaces: vec![namespaces],
        });

        application_configs.save()?;

        // Now please commit and push this file to you main branch to active the configurations for your application.
        println!(
        "{}",
        "Now please commit and push this file to you main branch to active the configurations for your application."
            .green()
            .bold()
            .italic()
        );
    }

    Ok(true)
}
