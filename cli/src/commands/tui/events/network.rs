use std::sync::Arc;

use tokio::sync::Mutex;
use wukong_sdk::services::gcloud::{google::logging::v2::LogEntry, LogEntries, LogEntriesOptions};

use crate::{
    auth,
    commands::{
        application::generate_filter,
        tui::{
            app::{App, Build, Commit, Deployment},
            StatefulList,
        },
    },
    config::Config,
    error::WKCliError,
    wukong_client::WKClient,
};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkEvent {
    FetchBuilds,
    FetchDeployments,
    FetchGCloudLogs,
}

pub async fn handle_network_event(
    app: Arc<Mutex<App>>,
    network_event: NetworkEvent,
) -> Result<(), WKCliError> {
    match network_event {
        NetworkEvent::FetchBuilds => {
            let mut app_ref = app.lock().await;
            let application = app_ref.state.current_application.clone();
            let namespace = app_ref.state.current_namespace.clone();
            app_ref.state.is_fetching_builds = true;

            drop(app_ref);

            let config = Config::load_from_default_path()?;
            let mut wk_client = WKClient::new(&config)?;

            let cd_pipeline_data = wk_client
                .fetch_cd_pipeline(&application, &namespace, "green")
                .await?
                .cd_pipeline;

            if let Some(cd_pipeline_data) = cd_pipeline_data {
                let mut app_ref = app.lock().await;
                app_ref.state.builds = cd_pipeline_data
                    .jenkins_builds
                    .into_iter()
                    .map(|build| {
                        let commits = build
                            .commits
                            .into_iter()
                            .map(|commit| Commit {
                                id: commit.id,
                                message_headline: commit.message_headline,
                            })
                            .collect();

                        Build {
                            name: build.build_artifact_name,
                            commits,
                        }
                    })
                    .collect();
            } else {
                let mut app_ref = app.lock().await;
                app_ref.state.builds = vec![];
            }

            let mut app_ref = app.lock().await;
            app_ref.state.is_fetching_builds = false;
        }
        NetworkEvent::FetchDeployments => {
            let mut app_ref = app.lock().await;
            let application = app_ref.state.current_application.clone();
            app_ref.state.is_fetching_deployments = true;
            app_ref.state.is_checking_namespaces = true;

            drop(app_ref);

            let config = Config::load_from_default_path()?;
            let mut wk_client = WKClient::new(&config)?;

            let cd_pipelines_data = wk_client
                .fetch_cd_pipelines(&application)
                .await?
                .cd_pipelines;

            let mut app_ref = app.lock().await;
            app_ref.state.deployments = cd_pipelines_data
                .into_iter()
                .map(|pipeline| Deployment {
                    name: pipeline.name,
                    environment: pipeline.environment,
                    version: pipeline.version,
                    enabled: pipeline.enabled,
                    deployed_ref: pipeline
                        .deployed_ref
                        .map(|deployed_ref| deployed_ref[..7].to_string()),
                    build_artifact: pipeline.build_artifact,
                    deployed_by: pipeline.deployed_by,
                    last_deployed_at: pipeline.last_deployment,
                    status: pipeline.status,
                })
                .collect();

            app_ref.state.is_fetching_deployments = false;

            // we only know the available namespaces after the deployments is fetched
            // so update namespace selections here
            let has_prod_namespace = app_ref
                .state
                .deployments
                .iter()
                .any(|pipeline| pipeline.environment == "prod");
            let has_staging_namespace = app_ref
                .state
                .deployments
                .iter()
                .any(|pipeline| pipeline.environment == "staging");
            let mut selections = vec![];
            if has_prod_namespace {
                selections.push(String::from("prod"));
            }
            if has_staging_namespace {
                selections.push(String::from("staging"));
            }

            let mut namespace_selections = StatefulList::with_items(selections);
            namespace_selections.select(0);
            app_ref.namespace_selections = namespace_selections;
            app_ref.state.is_checking_namespaces = false;
        }
        NetworkEvent::FetchGCloudLogs => {
            let app_ref = app.lock().await;
            let application = app_ref.state.current_application.clone();
            let namespace = app_ref.state.current_namespace.clone();
            let version = "green";
            let since = match app_ref.state.last_log_entry_timestamp.clone() {
                Some(t) => Some(t),
                None => Some("1m".to_string()),
            };

            drop(app_ref);

            let config = Config::load_from_default_path()?;
            let mut wk_client = WKClient::new(&config)?;

            let gcloud_access_token = auth::google_cloud::get_token_or_login().await;

            let application_resp = wk_client
                .fetch_application_with_k8s_cluster(&application, &namespace, version)
                .await?
                .application;

            if let Some(application_data) = application_resp {
                if let Some(cluster) = application_data.k8s_cluster {
                    let filter = generate_filter(
                        version,
                        &cluster.cluster_name,
                        &cluster.k8s_namespace,
                        &since,
                        &None,
                        &true,
                    )?;
                    let resource_names = vec![format!("projects/{}", cluster.google_project_id)];

                    let mut log = fetch_log_entries(
                        Some(resource_names.clone()),
                        Some(500),
                        Some(filter.clone()),
                        None,
                        gcloud_access_token.clone(),
                        &mut wk_client,
                    )
                    .await?;
                    let mut next_page_token = log.next_page_token.clone();
                    update_logs_entries(Arc::clone(&app), log.entries).await;

                    let app = Arc::clone(&app);
                    while next_page_token.is_some() && next_page_token != Some("".to_string()) {
                        log = fetch_log_entries(
                            Some(resource_names.clone()),
                            Some(500),
                            Some(filter.clone()),
                            log.next_page_token.clone(),
                            gcloud_access_token.clone(),
                            &mut wk_client,
                        )
                        .await
                        .unwrap();

                        // update next_page_token value
                        next_page_token = log.next_page_token.clone();

                        update_logs_entries(Arc::clone(&app), log.entries).await;
                    }
                }
            } else {
                let mut app_ref = app.lock().await;
                app_ref.state.log_entries = vec![];
            }

            let mut app_ref = app.lock().await;
            app_ref.state.is_fetching_log_entries = false;
        }
    }

    Ok(())
}

async fn fetch_log_entries(
    resource_names: Option<Vec<String>>,
    page_size: Option<i32>,
    filter: Option<String>,
    page_token: Option<String>,
    gcloud_access_token: String,
    wk_client: &mut WKClient,
) -> Result<LogEntries, WKCliError> {
    wk_client
        .get_gcloud_log_entries(
            LogEntriesOptions {
                resource_names,
                page_size,
                filter,
                page_token,
                // order_by: todo!(),
                ..Default::default()
            },
            gcloud_access_token,
        )
        .await
}

async fn update_logs_entries(app: Arc<Mutex<App>>, log_entries: Option<Vec<LogEntry>>) {
    if let Some(entries) = log_entries {
        if !entries.is_empty() {
            let mut app_ref = app.lock().await;

            app_ref.state.last_log_entry_timestamp = Some(
                entries
                    .last()
                    .unwrap()
                    .timestamp
                    .clone()
                    .unwrap_or_default()
                    .to_string(),
            );

            entries.into_iter().for_each(|entry| {
                if app_ref
                    .state
                    .log_entries_hash_map
                    .get(&entry.insert_id)
                    .is_none()
                {
                    app_ref.state.log_entries_ids.push(entry.insert_id.clone());
                    app_ref
                        .state
                        .log_entries_hash_map
                        .insert(entry.insert_id.clone(), entry);
                }
            });
        }
    }
}
