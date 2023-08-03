use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

use crate::{
    commands::Context,
    config::Config,
    error::WKCliError,
    loader::new_spinner,
    output::{colored_println, table::TableOutput},
    wukong_client::WKClient,
};

#[derive(Tabled, Serialize, Deserialize, Debug)]
struct Instance {
    #[tabled(rename = "INSTANCE-NAME")]
    name: String,
    #[tabled(rename = "INSTANCE-IP")]
    ip: String,
    #[tabled(rename = "INSTANCE-READY")]
    ready: bool,
    #[tabled(rename = "IS_LIVEBOOK_INSTANCE")]
    is_livebook: bool,
}

pub async fn handle_list(
    context: Context,
    namespace: &str,
    version: &str,
) -> Result<bool, WKCliError> {
    let loader = new_spinner();
    loader.set_message("Checking your permission to connect to the remote instance...");

    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::new(&config)?;

    let application = context.current_application;

    if !has_permission(&mut wk_client, &application, namespace, version).await? {
        loader.finish_and_clear();
        eprintln!("You don't have permission to connect to this instance.");
        eprintln!("Please check with your team manager to get approval first.");

        return Ok(false);
    }

    loader.finish_and_clear();
    eprintln!("Checking your permission to connect to the remote instance...✅");

    let fetch_loader = new_spinner();
    fetch_loader.set_message(format!(
        "Listing running instances of the application {}...",
        application.bright_green()
    ));

    let k8s_pods = wk_client
        .fetch_kubernetes_pods(&application, namespace, version)
        .await?
        .kubernetes_pods;

    let instances: Vec<Instance> = k8s_pods
        .into_iter()
        .map(|pod| Instance {
            name: format!("{}@{}/{}", version, namespace, pod.name),
            ip: pod.pod_ip.unwrap_or_default(),
            ready: pod.ready,
            is_livebook: pod.labels.contains(&"livebook".to_string()),
        })
        .collect();

    fetch_loader.finish_and_clear();

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
    wk_client: &mut WKClient,
    application: &str,
    namespace: &str,
    version: &str,
) -> Result<bool, WKCliError> {
    Ok(wk_client
        .fetch_is_authorized(application, namespace, version)
        .await?
        .is_authorized)
}
