use crate::{
    commands::Context,
    error::CliError,
    graphql::{kubernetes::watch_livebook, QueryClient, QueryClientBuilder},
    loader::new_spinner_progress_bar,
};
use futures::StreamExt;
use tokio::time::sleep;

pub async fn handle_connect(context: Context, name: &str, port: &u16) -> Result<bool, CliError> {
    let (namespace, version, instance_name) = parse_name(name)?;

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Checking your permission to connect to the remote instance...");

    let token = context
        .config
        .auth
        .as_ref()
        .ok_or(CliError::UnAuthenticated)?
        .id_token
        .clone();

    let client = QueryClientBuilder::default()
        .with_access_token(token.clone())
        .with_sub(context.state.sub)
        .with_api_url(context.config.core.wukong_api_url)
        .build()?;

    let application = context.state.application.unwrap();
    // Calling API ...
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

    let preparing_progress_bar = new_spinner_progress_bar();
    preparing_progress_bar.set_message("Preparing your remote instance...");

    let k8s_pods = client
        .fetch_kubernetes_pods(&application, &namespace, &version)
        .await
        .unwrap()
        .data
        .unwrap()
        .kubernetes_pods;

    let user_email = context
        .config
        .auth
        .ok_or(CliError::UnAuthenticated)?
        .account
        .clone()
        .replace("@", "-")
        .replace(".", "-");

    let has_existing_livebook_pod = k8s_pods
        .into_iter()
        .find(|pod| pod.labels.contains(&user_email))
        .is_some();

    if has_existing_livebook_pod {
        preparing_progress_bar.set_message("Found a provisioned Livebook instance belonging to you, re-creating your remote instance...");

        let _destroyed = client
            .destroy_livebook(&application, &namespace, &version)
            .await
            .unwrap();

        sleep(std::time::Duration::from_secs(5)).await;
    }

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

    if let Some(new_instance) = new_instance {
        let variables = watch_livebook::Variables {
            application: application.to_string(),
            namespace: namespace.to_string(),
            version: version.to_string(),
            name: new_instance.name.to_string(),
        };

        let (_client, mut stream) = client
            .subscribe_watch_livebook(variables, &token)
            .await
            .unwrap();

        while let Some(Ok(log)) = stream.next().await {
            if log.data.unwrap().watch_livebook.unwrap().ready {
                break;
            }
        }
        preparing_progress_bar.finish_and_clear();

        eprintln!("Your livebook instance is ready ! Use the following details to access");

        eprintln!("URL: {}", new_instance.url.unwrap_or_default());
        eprintln!("Password: {}", new_instance.password.unwrap_or_default());
    }
    let running_progress_bar = new_spinner_progress_bar();
    running_progress_bar
        .set_message("Your livebook instance is running. Press Ctrl-C to terminate...");

    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        running_progress_bar.finish_and_clear();
        tx.send(()).expect("Could not send signal on channel.")
    });

    rx.await.expect("Could not receive from channel.");
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

fn parse_name(name: &str) -> Result<(String, String, String), CliError> {
    if let Some((instance_info, instance_name)) = name.split_once("/") {
        if let Some((version, namespace)) = instance_info.split_once("@") {
            return Ok((
                namespace.to_string(),
                version.to_string(),
                instance_name.to_string(),
            ));
        }
    }

    todo!()
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
