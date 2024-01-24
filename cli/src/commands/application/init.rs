use crate::commands::application::config::{
    ApplicationConfig, ApplicationConfigs, ApplicationNamespaceAppsignalConfig,
    ApplicationNamespaceBuildConfig, ApplicationNamespaceCloudsqlConfig,
    ApplicationNamespaceConfig, ApplicationNamespaceDeliveryConfig,
    ApplicationNamespaceHoneycombConfig, ApplicationWorkflowConfig, ApplicationAddonsConfig, ApplicationAddonElixirLivebookConfig,
};
use crate::{error::WKCliError, loader::new_spinner};
use crossterm::style::Stylize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use heck::ToSnakeCase;
use owo_colors::OwoColorize;

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

    let workflows = vec!["Elixir CI", "Elixir Release Prod CD Image"];

    available_github_actions_loader.finish_and_clear();

    let excluded_workflows: Vec<String> = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Workflows to exclude")
        .items(&workflows[..])
        .interact()
        .expect("Unable to get user input")
        .iter()
        .map(|&index| workflows[index].to_string())
        .collect();

    let mut namespaces: Vec<ApplicationNamespaceConfig> = Vec::new();
    namespaces.push(configure_namespace("prod".to_string())?);

    let addons = vec!["Elixir livebook"];

    let selected_addons: Vec<String> = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{} {}", "(Optional)".bright_black(), "Addons"))
        .items(&["Elixir livebook"])
        .interact()
        .expect("Unable to get user input")
        .iter()
        .map(|&index| addons[index].to_string())
        .collect();

    let configure_staging_namespace = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("\nConfigure the staging namespace?")
        .default(true)
        .interact()?;

    if configure_staging_namespace {
        namespaces.push(configure_namespace("staging".to_string())?);
    }

    println!("\nPlease review your generated configurations below:");

    let agree_to_save = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to write this configuration into your repo?")
        .default(true)
        .interact()?;

    if agree_to_save {
        println!("{}", "Writing configuration...\n".green().bold().italic());

        let workflows = ApplicationWorkflowConfig {
            provider: "github_actions".to_string(),
            excluded_workflows,
            enable: true,
        };
        let elixir_livebook = selected_addons.iter().find(|&addon| addon == "Elixir livebook");

        application_configs.application = Some(ApplicationConfig {
            name,
            enable: true,
            workflows: Some(workflows),
            namespaces,
            addons: Some(ApplicationAddonsConfig {
                elixir_livebook: if elixir_livebook.is_none() {
                    None
                } else {
                    Some(ApplicationAddonElixirLivebookConfig {
                        enable: true,
                        allowed_admins: vec![],
                    })
                },
            }),
        });

        application_configs.save()?;

        println!(
        "{}",
        "Now please commit and push this file to you main branch to active the configurations for your application."
            .green()
            .bold()
            .italic()
        );
    };

    Ok(true)
}

fn configure_namespace(namespace_type: String) -> Result<ApplicationNamespaceConfig, WKCliError> {
    let rollup_strategy_options = ["Rolling Upgrade", "Blue/Green", "Canary"];

    let rollout_strategy: String = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Rollup strategy")
        .items(&rollup_strategy_options)
        .default(0)
        .interact()
        .map(|selected_index| {
            rollup_strategy_options[selected_index]
                .to_string()
                .to_snake_case()
        })
        .expect("Unable to get user input");

    let base_replica = Input::<u32>::with_theme(&ColorfulTheme::default())
        .with_prompt("Baseline Replicas")
        .default(3)
        .interact_text()
        .expect("Unable to get user input");

    let appsignal_environment = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            "(Optional)".bright_black(),
            "AppSignal Environment",
        ))
        .allow_empty(true)
        .interact_text()?;

    let honeycomb_dataset = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            "(Optional)".bright_black(),
            "Honeycomb Dataset"
        ))
        .allow_empty(true)
        .interact_text()?;

    let cloudsql_project_id = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            "(Optional)".bright_black(),
            "Google CloudSQL Project"
        ))
        .allow_empty(true)
        .interact_text()?;

    Ok(ApplicationNamespaceConfig {
        namespace_type: namespace_type.clone(),
        build: Some(ApplicationNamespaceBuildConfig {
            build_workflow: format!("{}-workflow", namespace_type.clone()),
        }),
        delivery: Some(ApplicationNamespaceDeliveryConfig {
            target: namespace_type.clone(),
            base_replica,
            rollout_strategy,
        }),
        appsignal: if appsignal_environment.is_empty() {
            None
        } else {
            Some(ApplicationNamespaceAppsignalConfig {
                enable: true,
                environment: appsignal_environment,
                default_namespace: namespace_type.clone(),
            })
        },
        honeycomb: if honeycomb_dataset.is_empty() {
            None
        } else {
            Some(ApplicationNamespaceHoneycombConfig {
                enable: true,
                dataset: honeycomb_dataset,
            })
        },
        cloudsql: if cloudsql_project_id.is_empty() {
            None
        } else {
            Some(ApplicationNamespaceCloudsqlConfig {
                enable: true,
                project_id: cloudsql_project_id,
            })
        },
    })
}
