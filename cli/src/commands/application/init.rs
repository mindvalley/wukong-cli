use crate::commands::application::config::{
    ApplicationAddonElixirLivebookConfig, ApplicationAddonsConfig, ApplicationConfig,
    ApplicationConfigs, ApplicationNamespaceAppsignalConfig, ApplicationNamespaceBuildConfig,
    ApplicationNamespaceCloudsqlConfig, ApplicationNamespaceConfig,
    ApplicationNamespaceDeliveryConfig, ApplicationNamespaceHoneycombConfig,
    ApplicationWorkflowConfig,
};
use crate::error::WKCliError;
use crossterm::style::Stylize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use heck::ToSnakeCase;
use owo_colors::OwoColorize;
use std::fs;

pub async fn handle_application_init() -> Result<bool, WKCliError> {
    println!("Welcome! Initializing per-repo configuration for your application.");
    let mut application_configs = ApplicationConfigs::new()?;

    let name = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Application Name")
        .default("my-first-application".to_string())
        .interact_text()?;

    let workflows = get_workflows_from_current_dir()?;

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
        println!("{}", "\nWriting configuration to .wukong.toml...".green().bold().italic());

        let workflows = ApplicationWorkflowConfig {
            provider: "github_actions".to_string(),
            excluded_workflows,
            enable: true,
        };

        let elixir_livebook_enabled = selected_addons
            .iter()
            .find(|&addon| addon == "Elixir livebook");

        application_configs.application = Some(ApplicationConfig {
            name,
            enable: true,
            workflows: Some(workflows),
            namespaces,
            addons: Some(ApplicationAddonsConfig {
                elixir_livebook: if elixir_livebook_enabled.is_none() {
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
            build_workflow: format!("{}_workflow", namespace_type.clone()),
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

fn get_workflows_from_current_dir() -> Result<Vec<String>, WKCliError> {
    let mut workflow_names = Vec::new();
    let workflows = fs::read_dir(".github/workflows")?;

    for workflow in workflows {
        let workflow = workflow?;

        if workflow.file_type()?.is_file()
            && workflow
                .path()
                .extension()
                .map_or(false, |ext| ext == "yml")
        {
            let workflow_content = fs::read_to_string(workflow.path())?;

            let workflow_values: serde_yaml::Value = serde_yaml::from_str(&workflow_content)
                .map_err(|_| WKCliError::UnableToParseYmlFile)?;

            if let Some(workflow_name) = workflow_values
                .get("name")
                .and_then(serde_yaml::Value::as_str)
            {
                workflow_names.push(workflow_name.to_string());
            }
        }
    }

    Ok(workflow_names)
}
