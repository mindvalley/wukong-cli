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

struct KubernetesPod {
    name: String,
    ready: bool,
    is_livebook: Option<bool>,
}

pub async fn handle_connect(context: Context) -> Result<bool, CliError> {
    let namespace = match select_deployment_namespace()? {
        Some(namespace) => namespace,
        None => return Ok(false),
    };

    let version = match select_deployment_version()? {
        Some(version) => version,
        None => return Ok(false),
    };

    let check_permission_progress_bar = new_spinner_progress_bar();

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

    // Check for permission:
    check_permission_progress_bar
        .set_message("Checking your permission to connect to the remote instance...");
    if !has_permission(&client, &application, &namespace, &version)
        .await
        .unwrap()
    {
        check_permission_progress_bar.finish_and_clear();
        eprintln!("You don't have permission to connect to this instance.");
        eprintln!("Please check with your team manager to get approval first.");

        return Ok(false);
    }

    check_permission_progress_bar.finish_and_clear();

    let fetch_instance_progress_bar = new_spinner_progress_bar();
    fetch_instance_progress_bar.set_message(format!(
        "Finding the available instances to connect to in the {} version...",
        version.bright_green()
    ));

    let k8s_pods = get_ready_k8s_pods(&client, &application, &namespace, &version).await?;

    fetch_instance_progress_bar.finish_and_clear();

    let instance_name_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please choose the instance you want to connect")
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

    cleanup_previous_livebook_instance(
        &client,
        &application,
        &namespace,
        &version,
        &auth_config,
        preparing_progress_bar.clone(),
    )
    .await?;

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

async fn cleanup_previous_livebook_instance(
    client: &QueryClient,
    application: &str,
    namespace: &str,
    version: &str,
    auth_config: &&crate::config::AuthConfig,
    preparing_progress_bar: indicatif::ProgressBar,
) -> Result<(), CliError> {
    let k8s_pods = client
        .fetch_kubernetes_pods(application, namespace, version)
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
            .destroy_livebook(application, namespace, version)
            .await
            .unwrap();

        // wait 5 seconds for the pod to be destroyed, otherwise it will failed when deploying new
        // livebook on the next step
        sleep(std::time::Duration::from_secs(5)).await;

        debug!("Destroyed the exisiting livebook instance.");
    }

    Ok(())
}

async fn get_ready_k8s_pods(
    client: &QueryClient,
    application: &str,
    namespace: &str,
    version: &str,
) -> Result<Vec<KubernetesPod>, CliError> {
    let k8s_pods = client
        .fetch_kubernetes_pods(application, namespace, version)
        .await?
        .data
        .unwrap()
        .kubernetes_pods;

    // filter out the pods that are not ready and livebook pods
    let ready_pods = k8s_pods
        .into_iter()
        .map(|pod| KubernetesPod {
            name: pod.name,
            ready: pod.ready,
            is_livebook: Some(pod.labels.contains(&"livebook".to_string())),
        })
        .filter(|pod| pod.ready && !pod.is_livebook.unwrap_or_default())
        .collect();

    Ok(ready_pods)
}

fn select_deployment_namespace() -> Result<Option<String>, CliError> {
    let namespace_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please choose the namespace you want to connect to")
        .default(0)
        .items(&[
            DeploymentNamespace::Prod.to_string(),
            DeploymentNamespace::Staging.to_string(),
        ])
        .interact()?;

    let namespace = match namespace_idx {
        0 => DeploymentNamespace::Prod.to_string().to_lowercase(),
        1 => DeploymentNamespace::Staging.to_string().to_lowercase(),
        _ => {
            eprintln!("You didn't choose any namespace to connect to.");
            return Ok(None);
        }
    };

    Ok(Some(namespace))
}

fn select_deployment_version() -> Result<Option<String>, CliError> {
    let version_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please choose the version you want to connect to")
        .default(0)
        .items(&[
            DeploymentVersion::Green.to_string(),
            DeploymentVersion::Blue.to_string(),
        ])
        .interact()?;

    let version = match version_idx {
        0 => DeploymentVersion::Green.to_string().to_lowercase(),
        1 => DeploymentVersion::Blue.to_string().to_lowercase(),
        _ => {
            eprintln!("You didn't choose any version to connect to.");
            return Ok(None);
        }
    };

    Ok(Some(version))
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
