use super::{DeploymentNamespace, DeploymentVersion};
use crate::{
    commands::Context,
    error::{CliError, DeploymentError},
    graphql::QueryClient,
    loader::new_spinner_progress_bar,
    output::colored_println,
    telemetry::{self, TelemetryData, TelemetryEvent},
};
use base64::Engine;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use edit::Builder;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use wukong_telemetry_macro::wukong_telemetry;

enum BuildSelectionLayout {
    TwoColumns { data: Vec<TwoColumns> },
    ThreeColumns { data: Vec<ThreeColumns> },
}

#[derive(Default)]
struct TwoColumns {
    left: String,
    right: Vec<String>,
    left_width: usize,
}

impl Display for TwoColumns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.right.is_empty() {
            write!(f, "{0: <width$}", self.left, width = self.left_width)?;
        } else {
            for (i, value) in self.right.iter().enumerate() {
                if i == 0 {
                    write!(
                        f,
                        "{0: <width$} {1}",
                        self.left,
                        value,
                        width = self.left_width
                    )?;
                } else {
                    write!(f, "  {0: <width$} {1}", "", value, width = self.left_width)?;
                }
                if i != (self.right.len() - 1) {
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct ThreeColumns {
    left: String,
    middle: String,
    right: Vec<String>,
    left_width: usize,
}

impl Display for ThreeColumns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.right.is_empty() {
            write!(
                f,
                "{0: <width$} {1: <1}",
                self.left,
                self.middle,
                width = self.left_width
            )?;
        } else {
            for (i, value) in self.right.iter().enumerate() {
                if i == 0 {
                    write!(
                        f,
                        "{0: <width$} {1: <1} {2}",
                        self.left,
                        self.middle,
                        value,
                        width = self.left_width
                    )?;
                } else {
                    write!(
                        f,
                        "  {0: <width$} {1: <1} {2}",
                        "",
                        "",
                        value,
                        width = self.left_width
                    )?;
                }
                if i != (self.right.len() - 1) {
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct CdPipelineWithBuilds {
    name: String,
    version: String,
    enabled: bool,
    deployed_ref: Option<String>,
    build_artifact: Option<String>,
    last_deployed_at: Option<i64>,
    last_successfully_deployed_artifact: Option<String>,
    status: Option<String>,
    jenkins_builds: Vec<JenkinsBuild>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JenkinsBuild {
    build_duration: Option<i64>,
    build_number: i64,
    build_artifact_name: String,
    build_branch: String,
    build_url: String,
    name: String,
    result: String,
    timestamp: i64,
    total_duration: Option<i64>,
    wait_duration: Option<i64>,
    commits: Vec<Commit>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Commit {
    id: String,
    author: String,
    message_headline: String,
}

fn capitalize_first_letter(o: &str) -> String {
    o[0..1].to_uppercase() + &o[1..]
}

#[wukong_telemetry(command_event = "deployment_execute")]
pub async fn handle_execute(
    context: Context,
    namespace: &Option<DeploymentNamespace>,
    version: &Option<DeploymentVersion>,
    artifact: &Option<String>,
) -> Result<bool, CliError> {
    if namespace.is_none() && version.is_none() && artifact.is_none() {
        println!("Not detecting any flags, entering deployment terminal......");
    }

    // SAFETY: This is safe to unwrap because we know that `application` is not None.
    let current_application = context.state.application.unwrap();
    colored_println!("Current application: {current_application}");

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Checking available CD pipelines ...");

    // Calling API ...
    let mut client = QueryClient::from_default_config()?;

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
    let selected_build: String;

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

        println!("You've selected `{selected_namespace}` as the deployment namespace.\n");
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

        println!("You've selected `{selected_version}` as the deployment version.\n");
    }

    let inverse_version = if selected_version.to_lowercase() == "green" {
        "blue".to_string()
    } else {
        "green".to_string()
    };

    println!(
        "{} {} {} {}",
        "✔".green(),
        "Step 3: Checking the status of the latest".bold(),
        capitalize_first_letter(&inverse_version).green(),
        "deployment...".bold()
    );

    let deployment_status = get_deployment_status(
        &mut client,
        &current_application,
        &selected_namespace.to_lowercase(),
        &inverse_version,
    )
    .await?;

    println!("Deployment status: {}\n", deployment_status.bold());

    if deployment_status != "SUCCEEDED" {
        let agree_to_continue = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(
                    format!(
                    "It seems the {} deployment is not in a stable state, are you still want to proceed with the {} deployment ?",
                        capitalize_first_letter(&inverse_version).green(),
                        capitalize_first_letter(&selected_version).green(),
                    )
                )
                .default(false)
                .interact()?;

        if !agree_to_continue {
            return Ok(false);
        }
    }

    if let Some(artifact) = artifact {
        selected_build = artifact.to_string();
        println!(
            "{} {} `{}`.\n",
            "✔".green(),
            "Step 4: You've selected build artifact".bold(),
            selected_build.green()
        );
    } else {
        let progress_bar = new_spinner_progress_bar();
        progress_bar.set_message("Fetching available build artifacts ...");

        let cd_pipeline_data = client
            .fetch_cd_pipeline(
                &current_application,
                &selected_namespace.to_lowercase(),
                &selected_version.to_lowercase(),
            )
            .await?
            .data
            .unwrap()
            .cd_pipeline;

        selected_build = match cd_pipeline_data {
            Some(cd_pipeline_data) => {
                let cd_pipeline = CdPipelineWithBuilds {
                    name: cd_pipeline_data.name,
                    version: cd_pipeline_data.version,
                    enabled: cd_pipeline_data.enabled,
                    deployed_ref: cd_pipeline_data.deployed_ref,
                    build_artifact: cd_pipeline_data.build_artifact,
                    last_successfully_deployed_artifact: cd_pipeline_data
                        .last_successfully_deployed_artifact,
                    last_deployed_at: cd_pipeline_data.last_deployment,
                    status: cd_pipeline_data.status,
                    jenkins_builds: cd_pipeline_data
                        .jenkins_builds
                        .into_iter()
                        .map(|build| {
                            let commits: Vec<Commit> = build
                                .commits
                                .into_iter()
                                .map(|commit| Commit {
                                    id: commit.id,
                                    author: commit.author,
                                    message_headline: commit.message_headline,
                                })
                                .collect();

                            JenkinsBuild {
                                build_duration: build.build_duration,
                                build_number: build.build_number,
                                build_branch: build.build_branch,
                                build_url: build.build_url,
                                build_artifact_name: build.build_artifact_name,
                                name: build.name,
                                result: build.result,
                                timestamp: build.timestamp,
                                total_duration: build.total_duration,
                                wait_duration: build.wait_duration,
                                commits,
                            }
                        })
                        .collect(),
                };

                let build_selections = if let Some(build_artifact) =
                    &cd_pipeline.last_successfully_deployed_artifact
                {
                    if build_artifact.contains("-build-") {
                        let mut width = 0;
                        let mut build_selection: Vec<ThreeColumns> = cd_pipeline
                            .jenkins_builds
                            .iter()
                            .map(|build| {
                                let commits: Vec<String> = build
                                    .commits
                                    .iter()
                                    .map(|commit| commit.message_headline.clone())
                                    .collect();

                                let build_artifact_name = build.build_artifact_name.clone();
                                if build_artifact_name.len() > width {
                                    width = build_artifact_name.len();
                                }

                                if *build_artifact == build_artifact_name {
                                    ThreeColumns {
                                        left: build_artifact_name,
                                        middle: "*".to_string(),
                                        right: commits,
                                        left_width: 0,
                                    }
                                } else {
                                    ThreeColumns {
                                        left: build_artifact_name,
                                        middle: "".to_string(),
                                        right: commits,
                                        left_width: 0,
                                    }
                                }
                            })
                            .collect();

                        build_selection.iter_mut().for_each(|build| {
                            build.left_width = width;
                        });

                        BuildSelectionLayout::ThreeColumns {
                            data: build_selection,
                        }
                    } else {
                        BuildSelectionLayout::TwoColumns {
                            data: generate_two_columns_build_selection(&cd_pipeline),
                        }
                    }
                } else {
                    BuildSelectionLayout::TwoColumns {
                        data: generate_two_columns_build_selection(&cd_pipeline),
                    }
                };

                progress_bar.finish_and_clear();

                let selected_build_index = match build_selections {
                    BuildSelectionLayout::TwoColumns { data } => {
                        Select::with_theme(&ColorfulTheme::default())
                            .with_prompt(
                                "Step 4: Please choose the build artifact you want to deploy",
                            )
                            .default(0)
                            .items(&data[..])
                            .interact()?
                    }
                    BuildSelectionLayout::ThreeColumns { data } => {
                        Select::with_theme(&ColorfulTheme::default())
                            .with_prompt(
                                "Step 4: Please choose the build artifact you want to deploy (* is the current deployed build)",
                            )
                            .default(0)
                            .items(&data[..])
                            .interact()?
                    }
                };

                let selected_build =
                    &cd_pipeline.jenkins_builds[selected_build_index].build_artifact_name;

                println!(
                    "You've selected `{selected_build}` as the build artifact for this deployment. \n"
                );

                selected_build.to_string()
            }
            None => {
                println!("There is no build for this.");
                return Ok(false);
            }
        };
    }

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Generating changelog ...");

    let changelogs_resp = client
        .fetch_changelogs(
            &current_application,
            &selected_namespace.to_lowercase(),
            &selected_version.to_lowercase(),
            &selected_build,
        )
        .await;

    progress_bar.finish_and_clear();

    let mut is_same_build = false;
    let mut changelog;

    match changelogs_resp {
        Ok(response) => {
            let changelogs_data = response.data.unwrap().changelogs;

            changelog = changelogs_data
                .into_iter()
                .map(|changelog| {
                    format!(
                        "{} by {} in {}",
                        changelog.message_headline, changelog.author, changelog.short_hash
                    )
                })
                .collect::<Vec<String>>()
                .join("\n\n");

            let instructions = r#"
<!-- You are in the CHANGELOG editor. -->
<!-- -->
<!-- The CHANGELOG above is generated by Wukong. You can edit the CHANGELOG here if you want. -->
<!-- The CHANGELOG will be used to send to slack. -->
<!-- Save the CHANGELOG (:wq if you are using vim) to go to the next deployment step. -->
<!-- -->
<!-- Lines in between '<!--' and '-->' will be ignored in the final CHANGELOG. -->"#;
            changelog = format!("{changelog}\n{instructions}");
        }
        Err(error) => match error {
            crate::error::APIError::ChangelogComparingSameBuild => {
                is_same_build = true;
                let instructions = r#"
<!-- You are in the CHANGELOG editor. -->
<!-- -->
<!-- You're selecting the same build artifact as the currently deployed version. -->
<!-- Because of that no CHANGELOG will be generated. That's why the above is empty. -->
<!-- -->
<!-- You can leave it blank or you can add your own CHANGELOG here. -->
<!-- The CHANGELOG will be used to send to slack. -->
<!-- Save the CHANGELOG (:wq if you are using vim) to go to the next deployment step. -->
<!-- -->
<!-- Lines in between '<!--' and '-->' will be ignored in the final CHANGELOG. -->"#;
                changelog = format!("{}\n{instructions}", "");
            }
            _ => {
                return Err(error.into());
            }
        },
    }

    if let Ok(edited) = edit::edit_with_builder(
        &changelog,
        Builder::new()
            .prefix("my-temporary-file")
            .suffix(".md")
            .rand_bytes(5),
    ) {
        // remove all comments
        let cleaned_changelog = edited
            .split('\n')
            .filter(|each| !each.starts_with("<!--"))
            .collect::<Vec<&str>>()
            .join("\n");

        println!("{}", "Step 5: Review your deployment".bold());
        println!("Please review your deployment CHANGELOG before execute it.\n");
        println!("{cleaned_changelog}");

        let agree_to_deploy = if !is_same_build {
            Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you agree to deploy this build ?")
                .interact()?
        } else {
            Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Are you sure to deploy the same build artifact with the current running one anyway?")
                .default(false)
                .interact()?
        };

        if agree_to_deploy {
            let progress_bar = new_spinner_progress_bar();
            progress_bar.set_message("Sending deployment ...");

            let base64_encoded_changelog =
                base64::engine::general_purpose::STANDARD.encode(cleaned_changelog);

            let resp = client
                .execute_cd_pipeline(
                    &current_application,
                    &selected_namespace.to_lowercase(),
                    &selected_version.to_lowercase(),
                    &selected_build,
                    Some(base64_encoded_changelog),
                    true,
                )
                .await?
                .data
                .unwrap()
                .execute_cd_pipeline;

            progress_bar.finish_and_clear();

            let deployment_url = resp.url;
            println!("Deployment is succefully sent! Please open this URL to check the deployment progress");
            println!("{deployment_url}");
        }
    } else {
        println!("Aborting deployment process!");
    }

    Ok(true)
}

fn generate_two_columns_build_selection(cd_pipeline: &CdPipelineWithBuilds) -> Vec<TwoColumns> {
    let mut width = 0;

    let mut two_columns: Vec<TwoColumns> = cd_pipeline
        .jenkins_builds
        .iter()
        .map(|build| {
            let commits: Vec<String> = build
                .commits
                .iter()
                .map(|commit| commit.message_headline.clone())
                .collect();

            let build_artifact_name = build.build_artifact_name.clone();
            if build_artifact_name.len() > width {
                width = build_artifact_name.len();
            }

            TwoColumns {
                left: build_artifact_name,
                right: commits,
                left_width: 0,
            }
        })
        .collect();

    two_columns
        .iter_mut()
        .for_each(|each| each.left_width = width);
    two_columns
}

async fn get_deployment_status(
    client: &mut QueryClient,
    application: &str,
    namespace: &str,
    version: &str,
) -> Result<String, CliError> {
    let deployments = client
        .fetch_cd_pipeline(application, namespace, version)
        .await?
        .data
        .unwrap()
        .cd_pipeline;

    let latest_deployment = deployments
        .iter()
        .find(|deployment| deployment.last_successfully_deployed_artifact.is_some());

    if latest_deployment.is_none() {
        Ok(String::from("TERMINAL"))
    } else {
        let deployment_status = latest_deployment.unwrap().status.clone();

        if deployment_status.is_none() {
            Ok(String::from("TERMINAL"))
        } else {
            Ok(deployment_status.unwrap())
        }
    }
}
