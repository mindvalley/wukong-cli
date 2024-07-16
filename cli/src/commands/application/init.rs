use crate::{
    application_config::{
        ApplicationAddonElixirLivebookConfig, ApplicationAddonsConfig, ApplicationConfig,
        ApplicationConfigs, ApplicationNamespaceAppsignalConfig, ApplicationNamespaceBuildConfig,
        ApplicationNamespaceCloudsqlConfig, ApplicationNamespaceConfig,
        ApplicationNamespaceDeliveryConfig, ApplicationNamespaceHoneycombConfig,
        ApplicationNamespaceSlackNotificationsConfig, ApplicationWorkflowConfig,
    },
    commands::Context,
    config::Config,
    error::WKCliError,
    loader::new_spinner,
    utils::inquire::inquire_render_config,
    wukong_client::WKClient,
};
use crossterm::style::Stylize;
use heck::ToSnakeCase;
use inquire::{required, CustomType};
use std::fs;
use wukong_sdk::{
    error::{APIError, WKError},
    graphql::{
        application_config_query::{self, ApplicationConfigQueryApplicationConfig},
        appsignal_apps_query::AppsignalAppsQueryAppsignalApps,
    },
};

pub async fn handle_application_init(context: Context) -> Result<bool, WKCliError> {
    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;
    let mut appsignal_apps = None;

    println!("Welcome! Initializing per-repo configuration for your application.");

    let mut application_configs = ApplicationConfigs::new();
    let mut name: String;

    loop {
        name = inquire::Text::new("Name of your application")
            .with_render_config(inquire_render_config())
            .with_validator(required!("Application name is required"))
            .with_placeholder("my-first-application")
            .prompt()?;

        let fetch_loader = new_spinner();
        fetch_loader.set_message("Validating application name ...");

        let has_application_config = get_application_config(&mut wk_client, &name)
            .await?
            .is_some();

        fetch_loader.finish_and_clear();

        if has_application_config {
            println!(
                "{}",
                format!(
                    " Application '{}' already exists. Please choose a different name",
                    name
                )
                .red()
            );
        } else {
            break;
        }
    }

    let workflows = get_workflows_from_current_dir()?;
    let mut excluded_workflows = Vec::new();

    if !workflows.is_empty() {
        excluded_workflows = inquire::MultiSelect::new(
            "Workflows to exclude from the Wukong CLI & TUI",
            workflows.to_vec(),
        )
        .with_render_config(inquire_render_config())
        .with_help_message(
            "Leave blank to ignore, ↑↓ to move, space to select one, → to all, ← to none",
        )
        .prompt()?;
    }

    eprintln!("\nNext is to configure the prod namespace for your application.\n");
    let mut namespaces: Vec<ApplicationNamespaceConfig> = Vec::new();
    namespaces
        .push(configure_namespace("prod".to_string(), &mut wk_client, &mut appsignal_apps).await?);

    let addons = ["Elixir Livebook"];
    let selected_addons = inquire::MultiSelect::new("Addons", addons.to_vec())
        .with_render_config(inquire_render_config())
        .with_help_message(
            "Leave blank to ignore, ↑↓ to move, space to select one, → to all, ← to none",
        )
        .prompt()?;

    println!();

    let configure_staging_namespace =
        inquire::Confirm::new("Do you want to configure the staging namespace?")
            .with_render_config(inquire_render_config())
            .with_default(true)
            .prompt()?;

    if configure_staging_namespace {
        namespaces.push(
            configure_namespace("staging".to_string(), &mut wk_client, &mut appsignal_apps).await?,
        );
    }

    let workflows = ApplicationWorkflowConfig {
        provider: "github_actions".to_string(),
        excluded_workflows,
    };

    let elixir_livebook_enabled = selected_addons
        .iter()
        .find(|addon| addon == &&"Elixir livebook");

    application_configs.application = ApplicationConfig {
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
    };

    let updated_application_configs = inquire::Editor::new(
        "Do you want to review the .wukong.toml file before writing to disk ?",
    )
    .with_render_config(inquire_render_config())
    .with_file_extension("toml")
    .with_predefined_text(&application_configs.to_string()?)
    .prompt()?
    .parse::<ApplicationConfigs>()?;

    application_configs.application = updated_application_configs.application;
    println!();

    let agree_to_save =
        inquire::Confirm::new("Do you want to write this configuration into your repo?")
            .with_render_config(inquire_render_config())
            .with_default(true)
            .prompt()?;

    if agree_to_save {
        println!(
            "{}",
            "\nWriting configuration to .wukong.toml..."
                .green()
                .bold()
                .italic()
        );

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

async fn configure_namespace(
    namespace_type: String,
    wk_client: &mut WKClient,
    appsignal_apps: &mut Option<Vec<AppsignalAppsQueryAppsignalApps>>,
) -> Result<ApplicationNamespaceConfig, WKCliError> {
    let workflows = get_workflows_from_current_dir()?;
    let setup_build_workflow =
        inquire::Confirm::new("Do you want to select a build workflow?")
            .with_render_config(inquire_render_config())
            .with_default(true)
            .with_help_message("If this is a new project you can skip this and configure it later once you have a build workflow")
            .prompt()?;

    let build_workflow = if setup_build_workflow {
        select_workflow(&workflows)
    } else {
        None
    };

    let application_name = inquire::Text::new("Pipeline application name")
        .with_render_config(inquire_render_config())
        .with_placeholder(" Optional")
        .with_help_message("Leave it blank to disable Spinnaker integration")
        .prompt()?;

    let pipeline_name = inquire::Text::new("Pipeline name")
        .with_render_config(inquire_render_config())
        .with_placeholder(" Optional")
        .with_help_message("Leave it blank to disable Spinnaker integration")
        .prompt()?;

    let rollup_strategy_options = ["Rolling Upgrade", "Blue/Green", "Canary"];
    let rollout_strategy =
        inquire::Select::new("Rollup strategy", rollup_strategy_options.to_vec())
            .with_render_config(inquire_render_config())
            .prompt()?;

    let base_replica = CustomType::<u32>::new("Number of replicas")
        .with_render_config(inquire_render_config())
        .with_default(3)
        .with_error_message("Please enter a valid number")
        .prompt()?;

    let mut selected_appsignal_app_id = None;
    let mut selected_appsignal_environment = None;
    let mut selected_appsignal_namespace = None;
    let setup_appsignal_environment =
        inquire::Confirm::new("Do you want to setup AppSignal integration?")
            .with_render_config(inquire_render_config())
            .with_default(false)
            .with_help_message("This is Optional. You may configure this manually later.")
            .prompt()?;

    if setup_appsignal_environment {
        if appsignal_apps.is_none() {
            let fetch_loader = new_spinner();
            fetch_loader.set_message("Fetching Appsignal apps ...");

            let apps = wk_client.fetch_appsignal_apps().await?.appsignal_apps;

            appsignal_apps.replace(apps);

            fetch_loader.finish_and_clear();
        }

        let appsignal_environment = inquire::Select::new(
            "AppSignal Environment",
            appsignal_apps
                .as_ref()
                .unwrap() // SAFRTY: the appsignal_apps is fetched above so it will always be Some(x) here
                .iter()
                .map(|app| format!("{} - {}", app.name, app.environment))
                .collect(),
        )
        .with_render_config(inquire_render_config())
        .prompt()?;

        // WHY inquire don't return index ?!!!
        let index = appsignal_apps
            .as_ref()
            .unwrap() // SAFRTY: the appsignal_apps is fetched above so it will always be Some(x) here
            .iter()
            .position(|app| format!("{} - {}", app.name, app.environment) == appsignal_environment)
            .unwrap(); // SAFETY: the appsignal_environment value is from the appsignal_apps list, so it is always present in the list

        let appsignal_namespace = inquire::Select::new(
            "Appsignal Namespace",
            appsignal_apps.as_ref().unwrap()[index].namespaces.clone(),
        )
        .with_render_config(inquire_render_config())
        .prompt()?;

        selected_appsignal_app_id = Some(appsignal_apps.as_ref().unwrap()[index].id.clone());
        selected_appsignal_environment =
            Some(appsignal_apps.as_ref().unwrap()[index].environment.clone());
        selected_appsignal_namespace = Some(appsignal_namespace);
    }

    let honeycomb_dataset = inquire::Text::new("Honeycomb Dataset")
        .with_render_config(inquire_render_config())
        .with_placeholder(" Optional")
        .with_help_message("Leave it blank to disable Honeycomb integration")
        .prompt()?;

    let cloudsql_project_id = inquire::Text::new("Google Project ID of your CloudSQL instance(s)")
        .with_render_config(inquire_render_config())
        .with_placeholder(" Optional")
        .with_help_message("Leave it blank to disable Google CloudSQL integration")
        .prompt()?;

    let slack_notifications = inquire::Text::new("Slack #channel for notifications")
        .with_render_config(inquire_render_config())
        .with_placeholder(" Optional")
        .with_help_message("Leave it blank to disable Slack notifications. Use 'channel-name' format without the '#'. \n\nIt is your responsibility to ensure the channel name exists, and to add the bot integration if the channel is private.")
        .prompt()?;

    Ok(ApplicationNamespaceConfig {
        namespace_type: namespace_type.clone(),
        build: build_workflow.map(|workflow| ApplicationNamespaceBuildConfig {
            build_workflow: workflow,
        }),
        delivery: Some(ApplicationNamespaceDeliveryConfig {
            target: namespace_type.clone(),
            base_replica,
            rollout_strategy: rollout_strategy.to_string().to_snake_case(),
            application_name: if application_name.is_empty() {
                None
            } else {
                Some(application_name)
            },
            pipeline_name: if pipeline_name.is_empty() {
                None
            } else {
                Some(pipeline_name)
            },
        }),
        appsignal: if setup_appsignal_environment {
            Some(ApplicationNamespaceAppsignalConfig {
                enable: true,
                app_id: selected_appsignal_app_id.unwrap(),
                environment: selected_appsignal_environment.unwrap(),
                default_namespace: selected_appsignal_namespace.unwrap(),
            })
        } else {
            None
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
        slack_notifications: if slack_notifications.is_empty() {
            None
        } else {
            Some(ApplicationNamespaceSlackNotificationsConfig {
                enable: true,
                channel: slack_notifications,
            })
        },
    })
}

fn select_workflow(workflows: &[String]) -> Option<String> {
    let chosen_workflow = inquire::Select::new("Select a Build Workflow", workflows.to_vec())
        .with_render_config(inquire_render_config())
        .with_help_message("You must select one Build Workflow")
        .prompt();

    match chosen_workflow {
        Ok(workflow) => Some(workflow),
        Err(_e) => {
            println!(
                "{}",
                "Skipped selecting build workflow. You may configure this manually later"
                    .red()
                    .bold()
            );
            None
        }
    }
}

fn get_workflows_from_current_dir() -> Result<Vec<String>, WKCliError> {
    let mut workflow_names = Vec::new();

    if let Ok(workflows) = fs::read_dir(".github/workflows") {
        for workflow in workflows {
            let workflow = workflow?;

            if workflow.file_type()?.is_file()
                && workflow
                    .path()
                    .extension()
                    .map_or(false, |ext| ext == "yml" || ext == "yaml")
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
    }

    Ok(workflow_names)
}

async fn get_application_config(
    wk_client: &mut WKClient,
    name: &str,
) -> Result<Option<ApplicationConfigQueryApplicationConfig>, WKCliError> {
    let application_config = match wk_client.fetch_application_config(name).await {
        Ok(resp) => Ok(resp),
        Err(err) => match &err {
            WKCliError::WKSdkError(WKError::APIError(APIError::ApplicationConfigNotFound)) => {
                Ok(application_config_query::ResponseData {
                    application_config: None,
                })
            }
            _ => Err(err),
        },
    }?
    .application_config;

    Ok(application_config)
}
