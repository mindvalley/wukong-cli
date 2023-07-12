use crate::{
    commands::Context, error::CliError, graphql::QueryClient, loader::new_spinner_progress_bar,
};
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

pub async fn handle_connect(context: Context, name: &str, port: &u16) -> Result<bool, CliError> {
    let spinner_style =
        ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}").unwrap();

    let (namespace, version, instance_name) = parse_name(name)?;

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_style(spinner_style.clone());
    progress_bar.set_prefix("[1/3]");
    progress_bar.set_message("Checking your permission to connect to the remote instance...");

    let mut client = QueryClient::from_default_config()?;

    let application = context.state.application.unwrap();

    if !has_permission(&mut client, &application, &namespace, &version)
        .await
        .unwrap()
    {
        progress_bar.finish_and_clear();
        eprintln!("You don't have permission to connect to this instance.");
        eprintln!("Please check with your team manager to get approval first.");

        return Ok(false);
    }

    progress_bar
        .finish_with_message("Checking your permission to connect to the remote instance...✅");

    let preparing_progress_bar = new_spinner_progress_bar();
    preparing_progress_bar.set_style(spinner_style.clone());
    preparing_progress_bar.set_prefix("[2/3]");
    preparing_progress_bar.set_message("Preparing your remote instance...");

    let livebook_resource = client
        .livebook_resource(&application, &namespace, &version)
        .await?
        .data
        .unwrap()
        .livebook_resource;

    let has_existing_livebook_pod = livebook_resource.is_some();

    if has_existing_livebook_pod {
        preparing_progress_bar.set_message("Found a provisioned Livebook instance belonging to you, re-creating your remote instance...");

        debug!("Destroying the exisiting livebook instance.");
        match client
            .destroy_livebook(&application, &namespace, &version)
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
                .livebook_resource(&application, &namespace, &version)
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

    debug!("Deploying a new livebook instance.");

    let new_instance = client
        .deploy_livebook(
            &application,
            &namespace,
            &version,
            &instance_name,
            *port as i64,
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
                .livebook_resource(&application, &namespace, &version)
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
        connection_test_progress_bar.set_prefix("[3/3]");
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
                .destroy_livebook(&application, &namespace, &version)
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
            .destroy_livebook(&application, &namespace, &version)
            .await
            .unwrap();

        exiting_progress_bar.finish_and_clear();
        eprintln!("Cleanup provisioned resources...✅");
    }

    Ok(true)
}

fn parse_name(name: &str) -> Result<(String, String, String), CliError> {
    if let Some((instance_info, instance_name)) = name.split_once('/') {
        if let Some((version, namespace)) = instance_info.split_once('@') {
            return Ok((
                namespace.to_string(),
                version.to_string(),
                instance_name.to_string(),
            ));
        }
    }

    Err(CliError::InvalidInput {
        value: name.to_string(),
    })
}

async fn has_permission(
    client: &mut QueryClient,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_name_success() {
        let (namespace, version, instance_name) = parse_name("green@prod/wukong-abc").unwrap();

        assert_eq!(namespace, "prod");
        assert_eq!(version, "green");
        assert_eq!(instance_name, "wukong-abc");
    }

    #[test]
    fn test_parse_name_failed() {
        match parse_name("green-prod/wukong-abc") {
            Ok(_) => panic!("the test should be failed"),
            Err(_) => assert!(true),
        }
    }
}
