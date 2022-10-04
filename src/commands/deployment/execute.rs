use super::DeploymentVersion;
use crate::{error::CliError, graphql::QueryClientBuilder, GlobalContext};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
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
    namespace: &Option<String>,
    version: &Option<DeploymentVersion>,
    artifact: &Option<String>,
) -> Result<bool, CliError<'a>> {
    if namespace.is_none() && version.is_none() && artifact.is_none() {
        println!("Not detecting any flags, entering deployment terminal......");
    }

    let env_selections = vec!["Prod", "Staging"];
    let selected_env = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Step 1: Please choose the namespace you want to deploy")
        .default(0)
        .items(&env_selections[..])
        .interact()
        .unwrap();

    println!(
        "You selected `{}` as the deployment namespace.\n",
        env_selections[selected_env]
    );

    let version_selections = vec!["Green", "Blue"];
    let selected_version = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Step 2: Please choose the version you want to deploy")
        .default(0)
        .items(&version_selections[..])
        .interact()
        .unwrap();

    println!(
        "You selected `{}` as the deployment version.\n",
        version_selections[selected_version]
    );

    // Calling API ...
    let client = QueryClientBuilder::new()
        .with_access_token(context.id_token.unwrap())
        .build()?;

    let cd_pipeline_data = client
        .fetch_cd_pipeline(
            context.application.as_ref().unwrap(),
            &env_selections[selected_env].to_lowercase(),
            &version_selections[selected_version].to_lowercase(),
        )
        .await?
        .data
        .unwrap()
        .cd_pipeline;

    let selected_build_number = match cd_pipeline_data {
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

    println!("Step 4: Review your deployment");
    println!("Please review your deployment CHANGELOG before execute it.\n");
    println!("CHANGELOG:");
    let changelog = r#"Move product id to tier charge plan mapping (#1508) (commit: 460ba03) (details / githubweb)
Add target sub id to upgrade endpoint (#1509) (commit: 765f3e5) (details / githubweb)
[MVC-444] [Backend] As a B2B user, I can receive Onboard Email when I was added individually to Org through MVE Platform (#1510) (commit: 3d4d673) (details / githubweb)
added next billing date to segment property (#1515) (commit: b46870d) (details / githubweb)
better refund resilience for Zuora (#1511) (commit: 2a327ed) (details / githubweb)
Modify preview upgrade params (#1516) (commit: ff63c8f) (details / githubweb)
Up 1583 downgrade schema and UI (#1513) (commit: 78e5fbf) (details / githubweb)
added sales api for tier charge plans (#1514) (commit: 14f0b26) (details / githubweb)"#;
    println!("{changelog}\n");

    let mut deploy = false;
    let mut _post_changelog_to_slack = false;

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you agree to deploy this build ?")
        .interact()
        .unwrap()
    {
        deploy = true;
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "Do you want to post the changelog to this Slack channel #mv-up-deployment ?",
            )
            .interact()
            .unwrap()
        {
            _post_changelog_to_slack = true;
        }
    }

    if deploy {
        println!("Sending deployment .....");

        let resp = client
            .execute_cd_pipeline(
                &context.application.unwrap(),
                &env_selections[selected_env].to_lowercase(),
                &version_selections[selected_version].to_lowercase(),
                Some(selected_build_number),
            )
            .await?
            .data
            .unwrap()
            .execute_cd_pipeline;

        if let Some(resp) = resp {
            println!("Deployment is succefully sent! Please open this URL to check the deployment progress");
            println!("{}", resp.url.unwrap());
        }
    }

    Ok(true)
}
