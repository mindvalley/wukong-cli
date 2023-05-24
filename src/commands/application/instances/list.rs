use crate::{
    commands::Context,
    error::CliError,
    graphql::{QueryClient, QueryClientBuilder},
    loader::new_spinner_progress_bar,
    output::{colored_println, table::TableOutput},
};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Tabled, Serialize, Deserialize, Debug)]
struct Instance {
    #[tabled(rename = "INSTANCE-NAME")]
    name: String,
    #[tabled(rename = "INSTANCE-IP")]
    ip: String,
    #[tabled(rename = "INSTANCE-READY")]
    ready: bool,
}

pub async fn handle_list(
    context: Context,
    namespace: &str,
    version: &str,
) -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Checking your permission to connect to the remote instance...");

    // Calling API ...
    let client = QueryClientBuilder::default()
        .with_access_token(
            context
                .config
                .auth
                .ok_or(CliError::UnAuthenticated)?
                .id_token,
        )
        .with_sub(context.state.sub)
        .with_api_url(context.config.core.wukong_api_url)
        .build()?;

    let application = &context.config.core.application;

    if has_permission(&client, application, namespace, version).await? {
        progress_bar.finish_and_clear();
        eprintln!("Checking your permission to connect to the remote instance...✅");
    } else {
        progress_bar.finish_and_clear();
        eprintln!("You don't have permission to connect to this instance.");
        eprintln!("Please check with your team manager to get approval first.");

        return Ok(false);
    }

    let fetching_progress_bar = new_spinner_progress_bar();
    fetching_progress_bar.set_message(format!(
        "Listing running instances of the application {}...",
        application.bright_green()
    ));

    let k8s_pods = client
        .fetch_kubernetes_pods(application, namespace, version)
        .await?
        .data
        .unwrap()
        .kubernetes_pods;

    let instances: Vec<Instance> = k8s_pods
        .into_iter()
        .map(|pod| Instance {
            name: format!("{}@{}/{}", version, namespace, pod.name),
            ip: pod.pod_ip.unwrap_or_default(),
            ready: pod.ready,
        })
        .collect();

    fetching_progress_bar.finish_and_clear();

    eprintln!(
        "Listing running instances of the application {}...✅",
        application.bright_green()
    );
    let instances_table = TableOutput {
        title: None,
        header: None,
        data: instances,
    };

    colored_println!("{}", instances_table);

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
