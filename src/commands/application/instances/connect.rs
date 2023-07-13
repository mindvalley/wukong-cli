use std::collections::HashSet;

use crate::{
    commands::{
        deployment::{DeploymentNamespace, DeploymentVersion},
        Context,
    },
    error::CliError,
    graphql::{QueryClient, QueryClientBuilder},
    loader::new_spinner_progress_bar,
    output::colored_println,
};
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::debug;
use owo_colors::OwoColorize;
use tokio::time::sleep;

// 2 mins timeout
const RETRY_WAIT_TIME_IN_SEC: u64 = 3;
const MAX_CHECKING_RETRY: u64 = 40;

struct Status {
    pod: bool,
    issuer: bool,
    ingress: bool,
    service: bool,
}

struct KubernetesPod {
    name: String,
    ready: bool,
    is_livebook: Option<bool>,
}

fn capitalize(data: &str) -> String {
    let mut result = String::new();
    let mut first = true;
    for value in data.chars() {
        if first {
            result.push(value.to_ascii_uppercase());
            first = false;
        } else {
            result.push(value);
        }
    }

    result
}

pub async fn handle_connect(context: Context) -> Result<bool, CliError> {
    let spinner_style =
        ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}").unwrap();

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

    // SAFETY: This is safe to unwrap because we know that `application` is not None.
    let current_application = context.state.application.unwrap();
    colored_println!("Current application: {current_application}\n");

    let namespace = match select_deployment_namespace(&client, &current_application).await? {
        Some(namespace) => namespace,
        None => return Ok(false),
    };

    let version = match select_deployment_version()? {
        Some(version) => version,
        None => return Ok(false),
    };

    let check_permission_progress_bar = new_spinner_progress_bar();
    check_permission_progress_bar.set_style(spinner_style.clone());
    check_permission_progress_bar.set_prefix("[1/4]");
    check_permission_progress_bar
        .set_message("Checking your permission to connect to the remote instance...");

    // Check for permission:
    if !has_permission(&client, &current_application, &namespace, &version).await? {
        eprintln!("You don't have permission to connect to this instance.");
        eprintln!("Please check with your team manager to get approval first.");

        return Ok(false);
    }

    check_permission_progress_bar
        .finish_with_message("Checking your permission to connect to the remote instance...✅");

    let fetch_instance_progress_bar = new_spinner_progress_bar();
    fetch_instance_progress_bar.set_style(spinner_style.clone());
    fetch_instance_progress_bar.set_prefix("[2/4]");
    fetch_instance_progress_bar.set_message(format!(
        "Finding the available instances to connect to in the {} version...",
        version.bright_green()
    ));

    let k8s_pods = get_ready_k8s_pods(&client, &current_application, &namespace, &version).await?;

    debug!("Found {} pods", k8s_pods.len());
    if k8s_pods.is_empty() {
        eprintln!("Found 0 instances. Either there's no running instances, or the instances are not ready to connect to using Livebook remote shell.");

        return Ok(false);
    }

    fetch_instance_progress_bar.finish_with_message(format!(
        "Finding the available instances to connect to in the {} version...✅",
        version.bright_green()
    ));

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
    preparing_progress_bar.set_style(spinner_style.clone());
    preparing_progress_bar.set_prefix("[3/4]");
    preparing_progress_bar.set_message("Preparing your remote instance...");

    cleanup_previous_livebook_instance(
        &client,
        &current_application,
        &namespace,
        &version,
        preparing_progress_bar.clone(),
    )
    .await?;

    debug!("Deploying a new livebook instance.");

    let new_instance = client
        .deploy_livebook(
            &current_application,
            &namespace,
            &version,
            &instance_name,
            8080,
        )
        .await?
        .data
        .unwrap()
        .deploy_livebook;

    preparing_progress_bar.finish();

    if let Some(new_instance) = new_instance {
        let m = MultiProgress::new();

        let (pod_loader, issuer_loader, ingress_loader, service_loader) =
            setup_loaders(&m, spinner_style.clone());

        let mut status = Status {
            pod: false,
            issuer: false,
            ingress: false,
            service: false,
        };

        for i in 0..MAX_CHECKING_RETRY {
            sleep(std::time::Duration::from_secs(RETRY_WAIT_TIME_IN_SEC)).await;
            let livebook_resource = client
                .livebook_resource(&current_application, &namespace, &version)
                .await?
                .data
                .unwrap()
                .livebook_resource;

            if let Some(livebook) = livebook_resource {
                if livebook.pod.status == "ok" && !status.pod {
                    pod_loader.finish_with_message("Pod created successfully ✅");
                    status.pod = true;
                }
                if livebook.issuer.status == "ok" && !status.issuer {
                    issuer_loader.finish_with_message("Issuer created successfully ✅");
                    status.issuer = true;
                }
                if livebook.ingress.status == "ok" && !status.ingress {
                    ingress_loader.finish_with_message("Ingress created successfully ✅");
                    status.ingress = true;
                }
                if livebook.service.status == "ok" && !status.service {
                    service_loader.finish_with_message("Service created successfully ✅");
                    status.service = true;
                }

                if status.pod && status.issuer && status.ingress && status.service {
                    m.clear().unwrap();
                    break;
                }

                if i == MAX_CHECKING_RETRY - 1 {
                    return Err(CliError::Timeout);
                }
            }
        }

        preparing_progress_bar.finish_with_message("Provisioning your livebook instance...✅");

        let connection_test_progress_bar = new_spinner_progress_bar();
        connection_test_progress_bar.set_style(spinner_style.clone());
        connection_test_progress_bar.set_prefix("[4/4]");
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

        if !connection_test_success {
            connection_test_progress_bar
                .finish_with_message("Testing connectivity to your livebook instance...❌");

            let destroy_progress_bar = new_spinner_progress_bar();
            destroy_progress_bar.set_message("Destroying the livebook instances...");
            let _destroyed = client
                .destroy_livebook(&current_application, &namespace, &version)
                .await
                .unwrap();
            destroy_progress_bar.finish_and_clear();
            eprintln!("The session has been terminated.");
            return Ok(false);
        }

        connection_test_progress_bar
            .finish_with_message("Testing connectivity to your livebook instance...✅");

        eprintln!();
        eprintln!();
        eprintln!("✅ Your livebook instance is ready! Use the following details to access:\n");
        eprintln!("URL 🔗: {}", url.cyan());
        eprintln!(
            "Password 🔑: {}",
            new_instance.password.unwrap_or_default().yellow()
        );
        eprintln!();

        let running_progress_bar = new_spinner_progress_bar();
        running_progress_bar
            .set_message("Your livebook instance is running. Press Ctrl-C to terminate...");

        tokio::signal::ctrl_c().await.unwrap();

        running_progress_bar.finish_and_clear();
        let exiting_progress_bar = new_spinner_progress_bar();
        exiting_progress_bar.set_message("You're exiting from your remote session. Cleaning up...");

        let _destroyed = client
            .destroy_livebook(&current_application, &namespace, &version)
            .await
            .unwrap();

        exiting_progress_bar.finish_and_clear();
        eprintln!("Cleanup provisioned resources...✅");
    }

    Ok(true)
}

async fn cleanup_previous_livebook_instance(
    client: &QueryClient,
    application: &str,
    namespace: &str,
    version: &str,
    preparing_progress_bar: indicatif::ProgressBar,
) -> Result<(), CliError> {
    let livebook_resource = client
        .livebook_resource(application, namespace, version)
        .await
        .unwrap()
        .data
        .unwrap()
        .livebook_resource;

    let has_existing_livebook_pod = livebook_resource.is_some();

    if has_existing_livebook_pod {
        preparing_progress_bar.set_message("Found a provisioned Livebook instance belonging to you, re-creating your remote instance...");

        debug!("Destroying the existing livebook instance.");
        match client
            .destroy_livebook(application, namespace, version)
            .await
        {
            Ok(_) => {}
            Err(err) => match &err {
                crate::error::APIError::ResponseError { code, message } => {
                    if !message.contains("pod_not_found") && !code.contains("pod_not_found") {
                        return Err(err.into());
                    }
                }
                _ => return Err(err.into()),
            },
        }

        for i in 0..MAX_CHECKING_RETRY {
            sleep(std::time::Duration::from_secs(RETRY_WAIT_TIME_IN_SEC)).await;

            let livebook_resource = client
                .livebook_resource(application, namespace, version)
                .await?
                .data
                .unwrap()
                .livebook_resource;

            if livebook_resource.is_none() {
                break;
            }

            if i == MAX_CHECKING_RETRY - 1 {
                return Err(CliError::Timeout);
            }
        }
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

async fn select_deployment_namespace(
    client: &QueryClient,
    application: &str,
) -> Result<Option<String>, CliError> {
    let cd_pipelines_resp = client
        .fetch_cd_pipeline_list(&application)
        .await?
        .data
        .unwrap()
        .cd_pipelines;

    let namespace_selections = cd_pipelines_resp
        .iter()
        .map(|pipeline| pipeline.environment.as_str())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<&str>>();

    if namespace_selections.is_empty() {
        println!("This application is not configured with any environment. Please configure at least 1 environment before using this feature.");
        return Ok(None);
    }

    let namespace_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please choose the namespace you want to connect to")
        .default(0)
        .items(
            &namespace_selections
                .iter()
                .map(|namespace| capitalize(namespace))
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let selected_namespace = namespace_selections[namespace_idx].to_string();

    Ok(Some(selected_namespace))
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

fn setup_loaders(
    m: &MultiProgress,
    spinner_style: ProgressStyle,
) -> (ProgressBar, ProgressBar, ProgressBar, ProgressBar) {
    let step = 1_000_000;

    let pod_loader = m.add(ProgressBar::new(step));
    pod_loader.set_style(spinner_style.clone());
    pod_loader.enable_steady_tick(std::time::Duration::from_millis(80));
    pod_loader.set_prefix("[1/?]");
    let issuer_loader = m.add(ProgressBar::new(step));
    issuer_loader.set_style(spinner_style.clone());
    issuer_loader.enable_steady_tick(std::time::Duration::from_millis(80));
    issuer_loader.set_prefix("[2/?]");
    let ingress_loader = m.add(ProgressBar::new(step));
    ingress_loader.set_style(spinner_style.clone());
    ingress_loader.enable_steady_tick(std::time::Duration::from_millis(80));
    ingress_loader.set_prefix("[3/?]");
    let service_loader = m.add(ProgressBar::new(step));
    service_loader.set_style(spinner_style);
    service_loader.enable_steady_tick(std::time::Duration::from_millis(80));
    service_loader.set_prefix("[4/?]");

    pod_loader.set_message("Setting up pod ...");
    issuer_loader.set_message("Setting up issuer ...");
    ingress_loader.set_message("Setting up ingress ...");
    service_loader.set_message("Setting up service ...");

    (pod_loader, issuer_loader, ingress_loader, service_loader)
}
