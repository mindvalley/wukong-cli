use super::{DeploymentNamespace, DeploymentVersion};
use crate::{
    error::{CliError, DeploymentError},
    graphql::QueryClientBuilder,
    loader::new_spinner_progress_bar,
    output::colored_println,
    telemetry::{self, TelemetryData, TelemetryEvent},
    GlobalContext,
};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use wukong_telemetry_macro::wukong_telemetry;

#[derive(Serialize, Deserialize, Debug)]
struct CdPipelineWithPreviousBuilds {
    name: String,
    version: String,
    enabled: bool,
    deployed_ref: Option<String>,
    build_artifact: Option<String>,
    last_deployed_at: Option<i64>,
    previous_deployed_artifacts: Option<Vec<Option<String>>>,
    status: Option<String>,
}

#[wukong_telemetry(command_event = "deployment_rollback")]
pub async fn handle_rollback(
    context: GlobalContext,
    namespace: &Option<DeploymentNamespace>,
    version: &Option<DeploymentVersion>,
) -> Result<bool, CliError> {
    if namespace.is_none() && version.is_none() {
        println!("Not detecting any flags, entering deployment terminal......");
    }

    // SAFETY: the application must not be None here
    let current_application = context.application.unwrap();
    colored_println!("Current application: {}", current_application);

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Checking available CD pipelines ...");

    // Calling API ...
    let client = QueryClientBuilder::new()
        .with_access_token(context.id_token.unwrap())
        .with_sub(context.sub) // for telemetry
        .build()?;

    let cd_pipelines_resp = client
        .fetch_cd_pipeline_list(&current_application)
        .await?
        .data
        .unwrap()
        .cd_pipelines;

    let has_prod_namespace = cd_pipelines_resp
        .iter()
        .any(|pipeline| pipeline.environment == "prod");
    let has_staging_namespace = cd_pipelines_resp
        .iter()
        .any(|pipeline| pipeline.environment == "staging");

    progress_bar.finish_and_clear();

    // if there is no Prod and Staging, return message, end the session
    if !has_prod_namespace && !has_staging_namespace {
        println!("This application is not configured with any CD Pipelines yet, cannot performing any deployment. Please configure at least 1 CD Pipeline before making a deployment");
        return Ok(false);
    }

    let selected_namespace: String;
    let selected_version: String;

    // if user provides namespace using --namespace flag
    if let Some(namespace) = namespace {
        match namespace {
            // if user set `prod` in --namespace flag but there is no `prod` namespace for the
            // application
            DeploymentNamespace::Prod => {
                if !has_prod_namespace {
                    return Err(CliError::DeploymentError(
                        DeploymentError::NamespaceNotAvailable {
                            namespace: "prod".to_string(),
                            application: current_application.clone(),
                        },
                    ));
                }
            }
            // if user set `staging` in --namespace flag but there is no `staging` namespace for the
            // application
            DeploymentNamespace::Staging => {
                if !has_staging_namespace {
                    return Err(CliError::DeploymentError(
                        DeploymentError::NamespaceNotAvailable {
                            namespace: "staging".to_string(),
                            application: current_application.clone(),
                        },
                    ));
                }
            }
        };

        selected_namespace = namespace.to_string();
        println!(
            "{} {} `{}` {}\n",
            "✔".green(),
            "Step 1: You've selected".bold(),
            selected_namespace.green(),
            "namespace.".bold()
        );
    } else {
        let mut namespace_selections = Vec::new();
        if has_prod_namespace {
            namespace_selections.push("Prod");
        }
        if has_staging_namespace {
            namespace_selections.push("Staging");
        }
        let selected_namespace_index = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Step 1: Please choose the namespace you want to deploy")
            .default(0)
            .items(&namespace_selections[..])
            .interact()?;

        selected_namespace = namespace_selections[selected_namespace_index].to_string();

        println!(
            "You've selected `{}` as the deployment namespace.\n",
            selected_namespace
        );
    }

    // after user selected a namespace, then we only can check what versions are available for this
    // application and namespace
    let has_green_version = cd_pipelines_resp
        .iter()
        .filter(|pipeline| pipeline.environment == selected_namespace.to_lowercase())
        .any(|pipeline| pipeline.version == "green");
    let has_blue_version = cd_pipelines_resp
        .iter()
        .filter(|pipeline| pipeline.environment == selected_namespace.to_lowercase())
        .any(|pipeline| pipeline.version == "blue");

    // if user provides version using --version flag
    if let Some(version) = version {
        match version {
            // if user set `blue` in --version flag but there is no `blue` version for the
            // application
            DeploymentVersion::Blue => {
                if !has_blue_version {
                    return Err(CliError::DeploymentError(
                        DeploymentError::VersionNotAvailable {
                            namespace: selected_namespace.to_lowercase(),
                            version: "blue".to_string(),
                            application: current_application.clone(),
                        },
                    ));
                }
            }
            // if user set `green` in --version flag but there is no `green` version for the
            // application
            DeploymentVersion::Green => {
                if !has_green_version {
                    return Err(CliError::DeploymentError(
                        DeploymentError::VersionNotAvailable {
                            namespace: selected_namespace.to_lowercase(),
                            version: "green".to_string(),
                            application: current_application.clone(),
                        },
                    ));
                }
            }
        };
        selected_version = version.to_string();
        println!(
            "{} {} `{}` {}\n",
            "✔".green(),
            "Step 2: You've selected".bold(),
            selected_version.green(),
            "version.".bold()
        );
    } else {
        let mut version_selections = Vec::new();
        if has_green_version {
            version_selections.push("Green");
        }
        if has_blue_version {
            version_selections.push("Blue");
        }
        let selected_version_index = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Step 2: Please choose the version you want to deploy")
            .default(0)
            .items(&version_selections[..])
            .interact()?;

        selected_version = version_selections[selected_version_index].to_string();

        println!(
            "You've selected `{}` as the deployment version.\n",
            selected_version
        );
    }

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Fetch the build artifact information ...");

    let cd_pipeline_data = client
        .fetch_cd_pipeline_for_rollback(
            &current_application,
            &selected_namespace.to_lowercase(),
            &selected_version.to_lowercase(),
        )
        .await?
        .data
        .unwrap()
        .cd_pipeline;

    match cd_pipeline_data {
        Some(cd_pipeline_data) => {
            let cd_pipeline = CdPipelineWithPreviousBuilds {
                name: cd_pipeline_data.name,
                version: cd_pipeline_data.version,
                enabled: cd_pipeline_data.enabled,
                deployed_ref: cd_pipeline_data.deployed_ref,
                build_artifact: cd_pipeline_data.build_artifact,
                last_deployed_at: cd_pipeline_data.last_deployment,
                previous_deployed_artifacts: cd_pipeline_data.previous_deployed_artifacts,
                status: cd_pipeline_data.status,
            };

            progress_bar.finish_and_clear();

            if let Some(build_artifact) = cd_pipeline.build_artifact {
                if let Some(previous_deployed_artifacts) = cd_pipeline.previous_deployed_artifacts {
                    if !previous_deployed_artifacts.is_empty() {
                        println!(
                            "{}",
                            "Step 3: Please confirm the build artifact to rollback to".bold()
                        );
                        println!(
                            "> Your latest deployment is using build artifact {}.",
                            build_artifact
                        );
                        println!(
                            "> The prior build artifact of you latest deployment is {}.",
                            previous_deployed_artifacts[0].as_ref().unwrap()
                        );
                        if Confirm::with_theme(&ColorfulTheme::default())
                            .with_prompt(format!(
                                "Please confirm that you will rollback to the build artifact {}",
                                previous_deployed_artifacts[0].as_ref().unwrap().green()
                            ))
                            .interact()?
                        {
                            let progress_bar = new_spinner_progress_bar();
                            progress_bar.set_message("Sending deployment ...");

                            let resp = client
                                .execute_cd_pipeline(
                                    &current_application,
                                    &selected_namespace.to_lowercase(),
                                    &selected_version.to_lowercase(),
                                    0,
                                    Some(
                                        previous_deployed_artifacts[0]
                                            .as_ref()
                                            .unwrap()
                                            .to_string(),
                                    ),
                                    None,
                                    true,
                                )
                                .await?
                                .data
                                .unwrap()
                                .execute_cd_pipeline;

                            progress_bar.finish_and_clear();

                            let deployment_url = resp.url;
                            println!("Deployment is succefully sent! Please open this URL to check the deployment progress");
                            println!("{}", deployment_url);
                        }
                    }
                }
            } else {
                println!("There is no previous deployed build that can rollback to.");
                return Ok(false);
            }
        }
        None => {
            println!("There is no cd pipeline for this.");
            return Ok(false);
        }
    };

    Ok(true)
}
