use super::{DeploymentNamespace, DeploymentVersion};
use crate::{
    error::{CliError, DeploymentError},
    graphql::QueryClientBuilder,
    loader::new_spinner_progress_bar,
    GlobalContext,
};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use edit::Builder;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Default)]
struct TwoColumns {
    left: String,
    right: Vec<String>,
}

impl Display for TwoColumns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.right.is_empty() {
            write!(f, "{0: <13}", self.left)?;
        } else {
            for (i, value) in self.right.iter().enumerate() {
                if i == 0 {
                    write!(f, "{0: <13} {1}", self.left, value)?;
                } else {
                    write!(f, "  {0: <13} {1}", "", value)?;
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
    last_deployed_at: Option<i64>,
    status: Option<String>,
    jenkins_builds: Vec<JenkinsBuild>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JenkinsBuild {
    build_duration: Option<i64>,
    build_number: i64,
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

pub async fn handle_execute(
    context: GlobalContext,
    namespace: &Option<DeploymentNamespace>,
    version: &Option<DeploymentVersion>,
    artifact: &Option<i64>,
) -> Result<bool, CliError> {
    if namespace.is_none() && version.is_none() && artifact.is_none() {
        println!("Not detecting any flags, entering deployment terminal......");
    }

    // SAFETY: the application must not be None here
    let current_application = context.application.unwrap();
    println!("Current application: {}", current_application.green());

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Checking available CD pipelines ...");

    // Calling API ...
    let client = QueryClientBuilder::new()
        .with_access_token(context.id_token.unwrap())
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
    let selected_build_number: i64;

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

    if let Some(artifact) = artifact {
        selected_build_number = *artifact;
        println!(
            "{} {} `{}`.\n",
            "✔".green(),
            "Step 3: You've selected build artifact".bold(),
            selected_build_number.green()
        );
    } else {
        let progress_bar = new_spinner_progress_bar();
        progress_bar.set_message("Fetch available build artifacts ...");

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

        selected_build_number = match cd_pipeline_data {
            Some(cd_pipeline_data) => {
                let cd_pipeline = CdPipelineWithBuilds {
                    name: cd_pipeline_data.name,
                    version: cd_pipeline_data.version,
                    enabled: cd_pipeline_data.enabled,
                    deployed_ref: cd_pipeline_data.deployed_ref,
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
                                build_url: build.build_url,
                                commits,
                                name: build.name,
                                result: build.result,
                                timestamp: build.timestamp,
                                total_duration: build.total_duration,
                                wait_duration: build.wait_duration,
                            }
                        })
                        .collect(),
                };

                let build_selections: Vec<TwoColumns> = cd_pipeline
                    .jenkins_builds
                    .iter()
                    .map(|build| {
                        let commits: Vec<String> = build
                            .commits
                            .iter()
                            .map(|commit| commit.message_headline.clone())
                            .collect();

                        TwoColumns {
                            left: format!("build-{}", build.build_number),
                            right: commits,
                        }
                    })
                    .collect();

                progress_bar.finish_and_clear();

                let selected_build = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Step 3: Please choose the build artifact you want to deploy")
                    .default(0)
                    .items(&build_selections[..])
                    .interact()?;

                let selected_build_number = cd_pipeline.jenkins_builds[selected_build].build_number;

                println!(
                    "You've selected `build-{}` as the build artifact for this deployment. \n",
                    selected_build_number
                );

                selected_build_number
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
            selected_build_number,
        )
        .await;

    progress_bar.finish_and_clear();

    let mut is_same_build = false;
    let mut changelogs;

    match changelogs_resp {
        Ok(response) => {
            let changelogs_data = response.data.unwrap().changelogs;

            println!("{}", "Step 4: Review your deployment".bold());
            println!("Please review your deployment CHANGELOG before execute it.\n");

            changelogs = changelogs_data
                .into_iter()
                .map(|changelog| {
                    format!(
                        "{} by {} in {}",
                        changelog.message_headline, changelog.author, changelog.short_hash
                    )
                })
                .collect::<Vec<String>>()
                .join("\n\n");

            let comments = r#"
<!-- This is changelogs editor. -->
<!-- This changelogs above is generated by Wukong. You can edit the changelogs here. -->
<!-- Save the changelogs (:wq if you are using vim) to go to the next deployment step. -->
<!-- Lines in between '<!--' and '-->' will be ignored. -->
"#;
            changelogs = format!("{}\n{}", changelogs, comments);
        }
        Err(error) => match error {
            crate::error::APIError::ChangelogComparingSameBuild => {
                is_same_build = true;
                println!("You're selecting the same build artifact as the currently deployed version. Because of that no CHANGELOG will be generated.");

                let comments = r#"
<!-- You're selecting the same build artifact as the currently deployed version. -->
<!-- Because of that no CHANGELOG will be generated. -->
<!-- You can leave it blank or you can add your own changelogs here. -->
<!-- Save the changelogs (:wq if you are using vim) to go to the next deployment step. -->
<!-- Lines in between '<!--' and '-->' will be ignored. -->
"#;
                changelogs = format!("{}\n{}", "", comments);
            }
            _ => {
                return Err(error.into());
            }
        },
    }

    if let Ok(edited) = edit::edit_with_builder(
        &changelogs,
        Builder::new()
            .prefix("my-temporary-file")
            .suffix(".md")
            .rand_bytes(5),
    ) {
        // remove all comments
        let cleaned_changelogs = edited
            .split('\n')
            .into_iter()
            .filter(|each| !each.starts_with("<!--"))
            .collect::<Vec<&str>>()
            .join("\n");

        println!("{}", &cleaned_changelogs);

        let agree_to_deploy = if !is_same_build {
            Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you agree to deploy this build ?")
                .interact()?
        } else {
            Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Are you sure to deploy this build artifact anyway?")
                .default(false)
                .interact()?
        };

        if agree_to_deploy {
            let progress_bar = new_spinner_progress_bar();
            progress_bar.set_message("Sending deployment ...");

            let base64_encoded_changelogs = base64::encode(cleaned_changelogs);

            let resp = client
                .execute_cd_pipeline(
                    &current_application,
                    &selected_namespace.to_lowercase(),
                    &selected_version.to_lowercase(),
                    selected_build_number,
                    Some(base64_encoded_changelogs),
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
    } else {
        println!("Aborting deployment process!");
    }

    Ok(true)
}
