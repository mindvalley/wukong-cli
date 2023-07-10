use crate::{
    commands::{
        deployment::{DeploymentNamespace, DeploymentVersion},
        Context,
    },
    error::CliError,
    graphql::{kubernetes::watch_livebook, QueryClient, QueryClientBuilder},
    loader::new_spinner_progress_bar,
};
use dialoguer::{theme::ColorfulTheme, Select};
use futures::StreamExt;
use log::debug;
use owo_colors::OwoColorize;
use tokio::time::sleep;

pub async fn handle_connect(context: Context) -> Result<bool, CliError> {
    let namespace_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Please choose the namespace you want to connect to",
        ))
        .default(0)
        .items(&vec![
            DeploymentNamespace::Prod.to_string(),
            DeploymentNamespace::Staging.to_string(),
        ])
        .interact_opt()?;

    let namespace = match namespace_idx {
        Some(0) => DeploymentNamespace::Prod.to_string().to_lowercase(),
        Some(1) => DeploymentNamespace::Staging.to_string().to_lowercase(),
        _ => {
            eprintln!("You didn't choose any namespace to connect to.");
            return Ok(false);
        }
    };

    let version_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Please choose the version you want to connect to",))
        .default(0)
        .items(&vec![
            DeploymentVersion::Green.to_string(),
            DeploymentVersion::Blue.to_string(),
        ])
        .interact()?;

    let version = match version_idx {
        0 => DeploymentVersion::Green.to_string().to_lowercase(),
        1 => DeploymentVersion::Blue.to_string().to_lowercase(),
        _ => {
            eprintln!("You didn't choose any version to connect to.");
            return Ok(false);
        }
    };

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Checking your permission to connect to the remote instance...");

    let auth_config = context
        .config
        .auth
        .as_ref()
        .ok_or(CliError::UnAuthenticated)?;

    let client = QueryClientBuilder::default()
        .with_access_token(auth_config.id_token.clone())
        .with_sub(context.state.sub)
        .with_api_url(context.config.core.wukong_api_url)
        .build()?;

    let application = context.state.application.unwrap();

    if !has_permission(&client, &application, &namespace, &version)
        .await
        .unwrap()
    {
        progress_bar.finish_and_clear();
        eprintln!("You don't have permission to connect to this instance.");
        eprintln!("Please check with your team manager to get approval first.");

        return Ok(false);
    }

    progress_bar.finish_and_clear();

    eprintln!("Checking your permission to connect to the remote instance...✅");

    let fetching_progress_bar = new_spinner_progress_bar();
    fetching_progress_bar.set_message(format!(
        "Listing running instances of the application {}...",
        application.bright_green()
    ));

    let k8s_pods = client
        .fetch_kubernetes_pods(&application, &namespace, &version)
        .await?
        .data
        .unwrap()
        .kubernetes_pods;

    fetching_progress_bar.finish_and_clear();

    let instance_name_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Please choose the instance you want to connect",))
        .default(0)
        .items(
            &k8s_pods
                .iter()
                .map(|pod| pod.name.clone())
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let instance_name = k8s_pods[instance_name_idx].name.clone();

    let preparing_progress_bar = new_spinner_progress_bar();
    preparing_progress_bar.set_message("Preparing your remote instance...");

    let k8s_pods = client
        .fetch_kubernetes_pods(&application, &namespace, &version)
        .await
        .unwrap()
        .data
        .unwrap()
        .kubernetes_pods;

    let user_email = auth_config.account.clone().replace(['@', '.'], "-");

    let has_existing_livebook_pod = k8s_pods
        .into_iter()
        .any(|pod| pod.labels.contains(&user_email));

    if has_existing_livebook_pod {
        preparing_progress_bar.set_message("Found a provisioned Livebook instance belonging to you, re-creating your remote instance...");

        debug!("Destroying the exisiting livebook instance.");
        let _destroyed = client
            .destroy_livebook(&application, &namespace, &version)
            .await
            .unwrap();

        // wait 5 seconds for the pod to be destroyed, otherwise it will failed when deploying new
        // livebook on the next step
        sleep(std::time::Duration::from_secs(5)).await;

        debug!("Destroyed the exisiting livebook instance.");
    }

    debug!("Deploying a new livebook instance.");
    let new_instance = client
        .deploy_livebook(&application, &namespace, &version, &instance_name, 8080)
        .await?
        .data
        .unwrap()
        .deploy_livebook;

    if let Some(new_instance) = new_instance {
        let variables = watch_livebook::Variables {
            application: application.to_string(),
            namespace: namespace.to_string(),
            version: version.to_string(),
            name: new_instance.name.to_string(),
        };

        debug!("Start watching to the new livebook instance.");
        let (_client, mut stream) = client.subscribe_watch_livebook(variables).await?;

        while let Some(Ok(resp)) = stream.next().await {
            debug!("{:?}", resp);
            if resp.data.unwrap().watch_livebook.unwrap().ready {
                break;
            }
        }
        preparing_progress_bar.finish_and_clear();
        eprintln!("Provisioning your livebook instance...✅");

        let connection_test_progress_bar = new_spinner_progress_bar();
        connection_test_progress_bar
            .set_message("Testing connectivity to your livebook instance...");

        let url = new_instance.url.unwrap_or_default();

        let mut connection_test_success = false;
        for i in 0..20 {
            match reqwest::get(&url).await {
                Ok(rs) => {
                    if rs.status().is_success() || rs.status().is_redirection() {
                        connection_test_success = true;
                        break;
                    }
                }
                Err(err) => {
                    debug!("{:?}", err);
                }
            }

            if i < 19 {
                debug!("wait for 5 seconds and test again.");
                sleep(std::time::Duration::from_secs(5)).await;
            }
        }

        connection_test_progress_bar.finish_and_clear();
        if !connection_test_success {
            eprintln!("Testing connectivity to your livebook instance...❌");

            let destroy_progress_bar = new_spinner_progress_bar();
            destroy_progress_bar.set_message("Destroying the livebook instances...");
            let _destroyed = client
                .destroy_livebook(&application, &namespace, &version)
                .await
                .unwrap();
            destroy_progress_bar.finish_and_clear();
            eprintln!("The session has been terminated.");
            return Ok(false);
        }

        eprintln!("Testing connectivity to your livebook instance...✅");

        eprintln!();
        eprintln!("✅ Your livebook instance is ready! Use the following details to access:\n");
        eprintln!("URL 🔗: {}", url.cyan());
        eprintln!(
            "Password 🔑: {}",
            new_instance.password.unwrap_or_default().yellow()
        );
        eprintln!();
    }
    let running_progress_bar = new_spinner_progress_bar();
    running_progress_bar
        .set_message("Your livebook instance is running. Press Ctrl-C to terminate...");

    tokio::signal::ctrl_c().await.unwrap();

    running_progress_bar.finish_and_clear();
    let exiting_progress_bar = new_spinner_progress_bar();
    exiting_progress_bar.set_message("You're exiting from your remote session. Cleaning up...");

    let _destroyed = client
        .destroy_livebook(&application, &namespace, &version)
        .await
        .unwrap();

    exiting_progress_bar.finish_and_clear();
    eprintln!("Cleanup provisioned resources...✅");

    Ok(true)
}

async fn has_permission(
    client: &QueryClient,
    application: &str,
    namespace: &str,
    version: &str,
) -> Result<bool, CliError> {
    Ok(client
        .fetch_is_authorized(application, namespace, version)
        .await?
        .data
        .unwrap()
        .is_authorized)
}
