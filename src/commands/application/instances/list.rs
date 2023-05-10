use crate::{
    commands::Context,
    error::CliError,
    graphql::QueryClientBuilder,
    loader::new_spinner_progress_bar,
    output::{colored_println, table::TableOutput},
};
use serde::{Deserialize, Serialize};
use tabled::Tabled;
use tokio::time::sleep;

#[derive(Tabled, Serialize, Deserialize, Debug)]
struct Instance {
    #[tabled(rename = "INSTANCE-NAME")]
    name: String,
    #[tabled(rename = "INSTANCE-IP")]
    ip: String,
}

pub async fn handle_list(context: Context, namespace: &str) -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Checking your permission to connect to the remote instance...");
    if has_permission().await {
        progress_bar.finish_and_clear();
        eprintln!("Checking your permission to connect to the remote instance...✅");
    } else {
        progress_bar.finish_and_clear();
        eprintln!("You don't have permission to connect to this instance.");
        eprintln!("Please check with your team manager to get approval first.");
        todo!();
    }

    let fetching_progress_bar = new_spinner_progress_bar();
    fetching_progress_bar.set_message(
        "Listing running instances of the application mv-wukong-ci-mock on namespace production...",
    );

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

    let k8s_pods = client
        .fetch_kubernetes_pods(namespace)
        .await?
        .data
        .unwrap()
        .kubernetes_pods;

    let instances: Vec<Instance> = k8s_pods
        .into_iter()
        .map(|pod| Instance {
            name: pod.name,
            ip: pod.host_ip,
        })
        .collect();

    fetching_progress_bar.finish_and_clear();

    eprintln!("Listing running instances of the application mv-wukong-ci-mock on namespace production...✅");
    let instances_table = TableOutput {
        title: None,
        header: None,
        data: instances,
    };

    colored_println!("{}", instances_table);

    Ok(true)
}

async fn has_permission() -> bool {
    sleep(std::time::Duration::from_secs(2)).await;
    true
}

async fn fetch_instances() -> Result<Vec<Instance>, CliError> {
    sleep(std::time::Duration::from_secs(2)).await;
    Ok(vec![
        Instance {
            name: "mv-wukong-ci-mock-blue-12c9d447c2-aaaa".to_string(),
            ip: "12.1.2.111".to_string(),
        },
        Instance {
            name: "mv-wukong-ci-mock-green-72bdd799bf-bbbb".to_string(),
            ip: "12.1.3.112".to_string(),
        },
    ])
}
