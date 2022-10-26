use super::{DeploymentNamespace, DeploymentVersion};
use crate::{
    error::CliError, graphql::QueryClientBuilder, loader::new_spinner_progress_bar, GlobalContext,
};
use dialoguer::{theme::ColorfulTheme, Confirm, Editor, Select};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

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
    commit_author: Option<String>,
    commit_id: Option<String>,
    commit_message: Option<String>,
    name: String,
    result: String,
    timestamp: i64,
    total_duration: Option<i64>,
    wait_duration: Option<i64>,
}

pub async fn handle_execute<'a>(
    context: GlobalContext,
    namespace: &Option<DeploymentNamespace>,
    version: &Option<DeploymentVersion>,
    artifact: &Option<i64>,
) -> Result<bool, CliError<'a>> {
    if namespace.is_none() && version.is_none() && artifact.is_none() {
        println!("Not detecting any flags, entering deployment terminal......");
    }

    let selected_namespace: String;
    let selected_version: String;
    let selected_build_number: i64;

    if let Some(namespace) = namespace {
        selected_namespace = namespace.to_string();
        println!(
            "{} {} `{}` {}\n",
            "✔".green(),
            "Step 1: You've selected".bold(),
            selected_namespace.green(),
            "namespace.".bold()
        );
    } else {
        let namespace_selections = vec!["Prod", "Staging"];
        let selected_namespace_index = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Step 1: Please choose the namespace you want to deploy")
            .default(0)
            .items(&namespace_selections[..])
            .interact()
            .unwrap();

        selected_namespace = namespace_selections[selected_namespace_index].to_string();

        println!(
            "You've selected `{}` as the deployment namespace.\n",
            selected_namespace
        );
    }

    if let Some(version) = version {
        selected_version = version.to_string();
        println!(
            "{} {} `{}` {}\n",
            "✔".green(),
            "Step 2: You've selected".bold(),
            selected_version.green(),
            "version.".bold()
        );
    } else {
        let version_selections = vec!["Green", "Blue"];
        let selected_version_index = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Step 2: Please choose the version you want to deploy")
            .default(0)
            .items(&version_selections[..])
            .interact()
            .unwrap();

        selected_version = version_selections[selected_version_index].to_string();

        println!(
            "You selected `{}` as the deployment version.\n",
            selected_version
        );
    }

    // Calling API ...
    let client = QueryClientBuilder::new()
        .with_access_token(context.id_token.unwrap())
        .build()?;

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
                context.application.as_ref().unwrap(),
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
                    jenkins_builds: match cd_pipeline_data.jenkins_builds {
                        Some(data) => data
                            .into_iter()
                            .flatten()
                            .map(|build| JenkinsBuild {
                                build_duration: build.build_duration,
                                build_number: build.build_number,
                                build_url: build.build_url,
                                commit_author: build.commit_author,
                                commit_id: build.commit_id,
                                commit_message: build.commit_msg,
                                name: build.name,
                                result: build.result,
                                timestamp: build.timestamp,
                                total_duration: build.total_duration,
                                wait_duration: build.wait_duration,
                            })
                            .collect(),
                        None => Vec::new(),
                    },
                };

                let build_selections: Vec<String> = cd_pipeline
                    .jenkins_builds
                    .iter()
                    .map(|build| {
                        format!(
                            "build-{}\t{}",
                            build.build_number,
                            build.commit_message.as_ref().unwrap_or(&"".to_string())
                        )
                    })
                    .collect();

                progress_bar.finish_and_clear();

                let selected_build = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Step 3: Please choose the build artifact you want to deploy")
                    .default(0)
                    .items(&build_selections[..])
                    .interact()
                    .unwrap();

                println!(
                    "You selected `build-{}` as the build artifact for this deployment. \n",
                    cd_pipeline.jenkins_builds[selected_build].build_number
                );

                cd_pipeline.jenkins_builds[selected_build].build_number
            }
            None => {
                println!("There is no build for this.");
                return Ok(false);
            }
        };
    }

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Generating changelog ...");

    let changelogs_data = client
        .fetch_changelogs(
            context.application.as_ref().unwrap(),
            &selected_namespace.to_lowercase(),
            &selected_version.to_lowercase(),
            selected_build_number,
        )
        .await?
        .data
        .unwrap()
        .changelogs;

    progress_bar.finish_and_clear();

    println!("{}", "Step 4: Review your deployment".bold());
    println!("Please review your deployment CHANGELOG before execute it.\n");

    match changelogs_data {
        Some(changelogs_data) => {
            let changelogs = changelogs_data
                .into_iter()
                .flatten()
                .map(|changelog| {
                    format!(
                        "{} by {} in {}",
                        changelog.message_headline, changelog.author, changelog.short_hash
                    )
                })
                .collect::<Vec<String>>()
                .join("\n\n");

            if let Some(rv) = Editor::new().edit(&changelogs).unwrap() {
                println!("{}\n", rv);

                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Do you agree to deploy this build ?")
                    .interact()
                    .unwrap()
                {
                    let progress_bar = new_spinner_progress_bar();
                    progress_bar.set_message("Sending deployment ...");

                    let resp = client
                        .execute_cd_pipeline(
                            &context.application.unwrap(),
                            &selected_namespace.to_lowercase(),
                            &selected_version.to_lowercase(),
                            Some(selected_build_number),
                        )
                        .await?
                        .data
                        .unwrap()
                        .execute_cd_pipeline;

                    progress_bar.finish_and_clear();

                    // SAFRTY: the resp SHOULDN'T be None
                    let deployment_url = resp.expect("There is no deployment url").url;
                    println!(
                        "Deployment is succefully sent! Please open this URL to check the deployment progress"
                    );
                    println!("{}", deployment_url);
                }
            } else {
                println!("Abort!");
            }
        }
        None => todo!(),
    }

    Ok(true)
}
