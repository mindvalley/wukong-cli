use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::debug;
use owo_colors::OwoColorize;
use tokio::time::sleep;
use wukong_sdk::error::{APIError, WKError};
use wukong_telemetry::*;
use wukong_telemetry_macro::*;

use crate::{
    commands::{
        application::{ApplicationNamespace, ApplicationVersion},
        deployment::{DeploymentNamespace, DeploymentVersion},
        Context,
    },
    config::Config,
    error::{ApplicationInstanceError, WKCliError},
    loader::new_spinner,
    output::colored_println,
    wukong_client::WKClient,
};

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
    pod_ip: Option<String>,
    node_name: Option<String>,
    cookie: Option<String>,
    is_livebook: Option<bool>,
}

#[wukong_telemetry(command_event = "application_instances_connect")]
pub async fn handle_connect(
    context: Context,
    namespace_arg: &Option<ApplicationNamespace>,
    version_arg: &Option<ApplicationVersion>,
) -> Result<bool, WKCliError> {
    let spinner_style =
        ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}").unwrap();

    let application = context.current_application;
    colored_println!("Current application: {application}\n");

    let mut namespace: String = match namespace_arg {
        Some(namespace) => namespace.to_string(),
        None => "".to_string(),
    };
    let mut version: String = match version_arg {
        Some(version) => version.to_string(),
        None => "".to_string(),
    };

    if namespace_arg.is_none() {
        namespace = match select_deployment_namespace()? {
            Some(data) => data,
            None => return Ok(false),
        };
    } else {
        println!(
            "{} {} `{}` {}\n",
            "✔".green(),
            "Step 1: You've selected".bold(),
            namespace.green(),
            "namespace.".bold()
        );
    }

    if version_arg.is_none() {
        version = match select_deployment_version()? {
            Some(version) => version,
            None => return Ok(false),
        };
    } else {
        println!(
            "{} {} `{}` {}\n",
            "✔".green(),
            "Step 2: You've selected".bold(),
            version.green(),
            "version.".bold()
        );
    }

    let check_permission_loader = new_spinner();
    check_permission_loader.set_style(spinner_style.clone());
    check_permission_loader.set_prefix("[1/2]");
    check_permission_loader
        .set_message("Checking your permission to connect to the remote instance...");

    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    if !has_permission(&mut wk_client, &application, &namespace, &version).await? {
        check_permission_loader.finish_and_clear();
        eprintln!("You don't have permission to connect to this instance.");
        eprintln!("Please check with your team manager to get approval first.");

        return Ok(false);
    }

    check_permission_loader
        .finish_with_message("Checking your permission to connect to the remote instance...✅");

    let fetch_instance_loader = new_spinner();
    fetch_instance_loader.set_style(spinner_style.clone());
    fetch_instance_loader.set_prefix("[2/2]");
    fetch_instance_loader.set_message(format!(
        "Finding the available instances to connect to in the {} version...",
        version.bright_green()
    ));

    let k8s_pods = get_ready_k8s_pods(&mut wk_client, &application, &namespace, &version).await?;

    debug!("Found {} pods", k8s_pods.len());
    if k8s_pods.is_empty() {
        eprintln!("Found 0 instances. Either there's no running instances, or the instances are not ready to connect to using Livebook remote shell.");

        return Ok(false);
    }

    fetch_instance_loader.finish_with_message(format!(
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
    let instance_object = &k8s_pods[instance_name_idx];

    let preparing_loader = new_spinner();
    preparing_loader.set_style(spinner_style.clone());
    preparing_loader.set_prefix("[1/2]");
    preparing_loader.set_message("Preparing your remote instance...");

    cleanup_previous_livebook_instance(
        &mut wk_client,
        &application,
        &namespace,
        &version,
        preparing_loader.clone(),
    )
    .await?;

    debug!("Deploying a new livebook instance.");

    let new_instance = wk_client
        .deploy_livebook(&application, &namespace, &version, &instance_name, 8080)
        .await?
        .deploy_livebook;

    preparing_loader.finish();

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
            let livebook_resource = wk_client
                .check_livebook_resource(&application, &namespace, &version)
                .await?
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
                    return Err(WKCliError::Timeout);
                }
            }
        }

        preparing_loader.finish_with_message("Provisioning your livebook instance...✅");

        let connection_test_loader = new_spinner();
        connection_test_loader.set_style(spinner_style.clone());
        connection_test_loader.set_prefix("[2/2]");
        connection_test_loader.set_message("Testing connectivity to your livebook instance...");

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
            connection_test_loader
                .finish_with_message("Testing connectivity to your livebook instance...❌");

            let destroy_loader = new_spinner();
            destroy_loader.set_message("Destroying the livebook instances...");
            let _ = wk_client
                .destroy_livebook(&application, &namespace, &version)
                .await?;
            destroy_loader.finish_and_clear();
            eprintln!("The session has been terminated.");
            return Ok(false);
        }

        connection_test_loader
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

        match (
            &instance_object.node_name,
            &instance_object.pod_ip,
            &instance_object.cookie,
        ) {
            (Some(node_name), Some(pod_ip), Some(cookie)) => {
                eprintln!("To connect to mv-wukong-api-proxy:");
                eprintln!("  1. start a new Notebook");
                eprintln!("  2. click on `+Smart`");
                eprintln!("  3. select `Remote Execution`");
                eprintln!("  4. follow the instructions that pop up to restart the Notebook");
                eprintln!("  5. enter these values into the `NODE` and `COOKIE` input boxes:");
                eprintln!("    NODE: 🖥️: {}@{}", node_name.cyan(), pod_ip.cyan());
                eprintln!("    COOKIE: 🍪: {}", cookie.yellow());
                eprintln!();
            }
            (None, ..) => {
                eprintln!("We were unable to determine the node name for the app node you wish to connect to. Please contact your administrator for assistance.")
            }
            (_, None, _) => {
                eprintln!("We were unable to determine the ip for the app node you wish to connect to. Please contact your administrator for assistance.")
            }
            (.., None) => {
                eprintln!("We were unable to determine the cookie for the app node you wish to connect to. Please contact your administrator for assistance.")
            }
        }

        eprintln!();
        eprintln!();

        let running_loader = new_spinner();
        running_loader
            .set_message("Your livebook instance is running. Press Ctrl-C to terminate...");

        tokio::signal::ctrl_c().await.unwrap();

        running_loader.finish_and_clear();
        let exiting_loader = new_spinner();
        exiting_loader.set_message("You're exiting from your remote session. Cleaning up...");

        let _ = wk_client
            .destroy_livebook(&application, &namespace, &version)
            .await
            .inspect_err(|_err| {
                eprintln!("Failed to destroy the livebook instance.");
            })?;

        exiting_loader.finish_and_clear();
        eprintln!("Cleanup provisioned resources...✅");
    }

    Ok(true)
}

async fn cleanup_previous_livebook_instance(
    wk_client: &mut WKClient,
    application: &str,
    namespace: &str,
    version: &str,
    preparing_progress_bar: indicatif::ProgressBar,
) -> Result<(), WKCliError> {
    let livebook_resource = wk_client
        .check_livebook_resource(application, namespace, version)
        .await?
        .livebook_resource;

    let has_existing_livebook_pod = livebook_resource.is_some();

    if has_existing_livebook_pod {
        preparing_progress_bar.set_message("Found a provisioned Livebook instance belonging to you, re-creating your remote instance...");

        debug!("Destroying the existing livebook instance.");
        if let Err(err) = wk_client
            .destroy_livebook(application, namespace, version)
            .await
        {
            match &err {
                WKCliError::WKSdkError(WKError::APIError(APIError::ResponseError {
                    code,
                    message,
                })) => {
                    if !message.contains("pod_not_found") && !code.contains("pod_not_found") {
                        return Err(err);
                    }
                }
                _ => return Err(err),
            }
        }

        for i in 0..MAX_CHECKING_RETRY {
            sleep(std::time::Duration::from_secs(RETRY_WAIT_TIME_IN_SEC)).await;

            let livebook_resource = wk_client
                .check_livebook_resource(application, namespace, version)
                .await?
                .livebook_resource;

            if livebook_resource.is_none() {
                break;
            }

            if i == MAX_CHECKING_RETRY - 1 {
                return Err(WKCliError::Timeout);
            }
        }
    }

    Ok(())
}

async fn get_ready_k8s_pods(
    wk_client: &mut WKClient,
    application: &str,
    namespace: &str,
    version: &str,
) -> Result<Vec<KubernetesPod>, WKCliError> {
    let k8s_pods = wk_client
        .fetch_kubernetes_pods(application, namespace, version)
        .await?
        .kubernetes_pods;

    // filter out the pods that are not ready and livebook pods
    let ready_pods = k8s_pods
        .into_iter()
        .map(|pod| KubernetesPod {
            name: pod.name,
            ready: pod.ready,
            pod_ip: pod.pod_ip,
            node_name: pod.node_name,
            cookie: pod.cookie,
            is_livebook: Some(pod.labels.contains(&"livebook".to_string())),
        })
        .filter(|pod| pod.ready && !pod.is_livebook.unwrap_or_default())
        .collect();

    Ok(ready_pods)
}

fn select_deployment_namespace() -> Result<Option<String>, WKCliError> {
    let namespace_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Step 1: Please choose the namespace you want to connect to")
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

fn select_deployment_version() -> Result<Option<String>, WKCliError> {
    let version_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Step 2: Please choose the version you want to connect to")
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
    wk_client: &mut WKClient,
    application: &str,
    namespace: &str,
    version: &str,
) -> Result<bool, WKCliError> {
    let is_authorized = match wk_client
        .fetch_is_authorized(application, namespace, version)
        .await
    {
        Ok(data) => data.is_authorized,
        Err(err) => match &err {
            WKCliError::WKSdkError(WKError::APIError(APIError::ResponseError {
                code,
                message: _,
            })) => {
                if code == "k8s_cluster_namespace_config_not_defined" {
                    return Err(WKCliError::ApplicationInstanceError(
                        ApplicationInstanceError::NamespaceNotAvailable,
                    ));
                } else if code == "k8s_cluster_version_config_not_defined" {
                    return Err(WKCliError::ApplicationInstanceError(
                        ApplicationInstanceError::VersionNotAvailable {
                            version: version.to_string(),
                        },
                    ));
                } else if code == "application_config_not_defined" {
                    return Err(WKCliError::ApplicationInstanceError(
                        ApplicationInstanceError::ApplicationNotFound,
                    ));
                } else {
                    return Err(err);
                }
            }
            _ => return Err(err),
        },
    };

    Ok(is_authorized)
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
