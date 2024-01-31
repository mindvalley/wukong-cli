use crate::commands::application::config::{
    ApplicationAddonElixirLivebookConfig, ApplicationAddonsConfig, ApplicationConfig,
    ApplicationConfigs, ApplicationNamespaceAppsignalConfig, ApplicationNamespaceBuildConfig,
    ApplicationNamespaceCloudsqlConfig, ApplicationNamespaceConfig,
    ApplicationNamespaceDeliveryConfig, ApplicationNamespaceHoneycombConfig,
    ApplicationWorkflowConfig,
};
use crate::error::WKCliError;
use crossterm::style::Stylize;
use heck::ToSnakeCase;
use inquire::ui::{
    Attributes, Color, ErrorMessageRenderConfig, IndexPrefix, RenderConfig, StyleSheet, Styled,
};
use inquire::{required, CustomType, Text};
use std::fs;

pub fn get_render_config() -> RenderConfig {
    RenderConfig {
        prompt_prefix: Styled::new("?").with_style_sheet(
            StyleSheet::new()
                .with_fg(inquire::ui::Color::LightCyan)
                .with_attr(Attributes::BOLD),
        ),
        answered_prompt_prefix: Styled::new("❯").with_fg(Color::LightGreen),
        prompt: StyleSheet::empty(),
        default_value: StyleSheet::empty().with_fg(Color::DarkGrey),
        placeholder: StyleSheet::new().with_fg(Color::DarkGrey),
        help_message: StyleSheet::empty()
            .with_fg(Color::LightMagenta)
            .with_attr(Attributes::BOLD),
        text_input: StyleSheet::empty(),
        error_message: ErrorMessageRenderConfig::default_colored().with_prefix(Styled::new("")),
        password_mask: '*',
        answer: StyleSheet::empty()
            .with_fg(Color::LightCyan)
            .with_attr(Attributes::BOLD),
        canceled_prompt_indicator: Styled::new("<canceled>").with_fg(Color::DarkRed),
        highlighted_option_prefix: Styled::new("❯").with_fg(Color::LightCyan),
        scroll_up_prefix: Styled::new("↑"),
        scroll_down_prefix: Styled::new("↓"),
        selected_checkbox: Styled::new("[x]")
            .with_fg(Color::LightGreen)
            .with_attr(Attributes::BOLD),
        unselected_checkbox: Styled::new("[ ]").with_attr(Attributes::BOLD),
        option_index_prefix: IndexPrefix::None,
        option: StyleSheet::empty(),
        selected_option: Some(StyleSheet::new().with_fg(Color::LightCyan)),
        editor_prompt: StyleSheet::new().with_fg(Color::DarkCyan),
    }
}

pub async fn handle_application_init() -> Result<bool, WKCliError> {
    println!("Welcome! Initializing per-repo configuration for your application.");

    let mut application_configs = ApplicationConfigs::new()?;

    let name = Text::new("Application name")
        .with_render_config(get_render_config())
        .with_validator(required!("Application name is required"))
        .with_placeholder("my-first-application")
        .prompt()?;

    let workflows = get_workflows_from_current_dir()?;
    let excluded_workflows = inquire::MultiSelect::new("Workflows to exclude", workflows)
        .with_render_config(get_render_config())
        .with_help_message(
            "Leave blank to ignore, ↑↓ to move, space to select one, → to all, ← to none",
        )
        .prompt()?;

    let mut namespaces: Vec<ApplicationNamespaceConfig> = Vec::new();
    namespaces.push(configure_namespace("prod".to_string())?);

    let addons = vec!["Elixir livebook"];
    let selected_addons = inquire::MultiSelect::new("Addons", addons.to_vec())
        .with_render_config(get_render_config())
        .with_help_message(
            "Leave blank to ignore, ↑↓ to move, space to select one, → to all, ← to none",
        )
        .prompt()?;

    println!();

    let configure_staging_namespace = inquire::Confirm::new("Configure the staging namespace?")
        .with_render_config(get_render_config())
        .with_default(true)
        .prompt()?;

    if configure_staging_namespace {
        namespaces.push(configure_namespace("staging".to_string())?);
    }

    let workflows = ApplicationWorkflowConfig {
        provider: "github_actions".to_string(),
        excluded_workflows,
        enable: true,
    };

    let elixir_livebook_enabled = selected_addons
        .iter()
        .find(|addon| addon == &&"Elixir livebook");

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

    let updated_application_configs =
        inquire::Editor::new("Do you want to review the .wukong.toml file?")
            .with_render_config(get_render_config())
            .with_file_extension("toml")
            .with_predefined_text(&application_configs.to_string()?)
            .prompt()?
            .parse::<ApplicationConfigs>()?;

    application_configs.application = updated_application_configs.application;
    println!();

    let agree_to_save =
        inquire::Confirm::new("Do you want to write this configuration into your repo?")
            .with_render_config(get_render_config())
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

fn configure_namespace(namespace_type: String) -> Result<ApplicationNamespaceConfig, WKCliError> {
    let rollup_strategy_options = ["Rolling Upgrade", "Blue/Green", "Canary"];
    let rollout_strategy =
        inquire::Select::new("Rollup strategy", rollup_strategy_options.to_vec())
            .with_render_config(get_render_config())
            .prompt()?;

    let base_replica = CustomType::<u32>::new("Baseline Replicas")
        .with_render_config(get_render_config())
        .with_default(3)
        .with_error_message("Please enter a valid number")
        .prompt()?;

    let appsignal_environment = inquire::Text::new("AppSignal Environment")
        .with_render_config(get_render_config())
        .with_placeholder(" Optional")
        .with_help_message("Leave it blank to disable AppSignal integration")
        .prompt()?;

    let honeycomb_dataset = inquire::Text::new("Honeycomb Dataset")
        .with_render_config(get_render_config())
        .with_placeholder(" Optional")
        .with_help_message("Leave it blank to disable Honeycomb integration")
        .prompt()?;

    let cloudsql_project_id = inquire::Text::new("Google CloudSQL Project")
        .with_render_config(get_render_config())
        .with_placeholder(" Optional")
        .with_help_message("Leave it blank to disable Google CloudSQL integration")
        .prompt()?;

    Ok(ApplicationNamespaceConfig {
        namespace_type: namespace_type.clone(),
        build: Some(ApplicationNamespaceBuildConfig {
            build_workflow: format!("{}_workflow", namespace_type.clone()),
        }),
        delivery: Some(ApplicationNamespaceDeliveryConfig {
            target: namespace_type.clone(),
            base_replica,
            rollout_strategy: rollout_strategy.to_string().to_snake_case(),
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