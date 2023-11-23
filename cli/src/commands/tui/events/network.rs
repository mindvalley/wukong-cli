use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};
use wukong_sdk::services::gcloud::{google::logging::v2::LogEntry, LogEntries, LogEntriesOptions};

use crate::{
    auth,
    commands::{
        application::generate_filter,
        tui::{
            app::{App, Build, Commit, Deployment, MAX_LOG_ENTRIES_LENGTH},
            StatefulList,
        },
    },
    config::{ApiChannel, Config},
    error::WKCliError,
    wukong_client::WKClient,
};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkEvent {
    GetBuilds,
    GetDeployments,
    GetGCloudLogs,
}

pub async fn handle_network_event(
    app: Arc<Mutex<App>>,
    network_event: NetworkEvent,
    channel: &ApiChannel,
) -> Result<(), WKCliError> {
    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, channel)?;

    match network_event {
        NetworkEvent::GetBuilds => get_builds(app, &mut wk_client).await?,
        NetworkEvent::GetDeployments => get_deployments(app, &mut wk_client).await?,
        NetworkEvent::GetGCloudLogs => get_gcloud_logs(app, &mut wk_client).await?,
    }

    Ok(())
}

fn set_version_selections(app_ref: &mut MutexGuard<'_, App>) {
    let deployments = &app_ref.state.deployments;

    let version_selections: Vec<String> = match &app_ref.state.current_namespace {
        Some(namespace) => deployments
            .iter()
            .filter_map(|pipeline| {
                if pipeline.environment == *namespace
                    && (pipeline.version == "green" || pipeline.version == "blue")
                {
                    Some(pipeline.version.to_string())
                } else {
                    None
                }
            })
            .collect(),
        None => Vec::new(),
    };

    // set version selections to state
    let mut version_selections_list = StatefulList::with_items(version_selections.clone());
    version_selections_list.select(0);

    // only select the first one if the current_version is None
    if app_ref.state.current_version.is_none() {
        app_ref.state.current_version = version_selections.first().cloned();
    }
    app_ref.version_selections = version_selections_list;

    app_ref.state.is_checking_version = false;
}

fn set_namespace_selections(app_ref: &mut MutexGuard<'_, App>) {
    let mut namespace_selections: Vec<String> = app_ref
        .state
        .deployments
        .iter()
        .filter_map(|pipeline| {
            if pipeline.environment == "prod" || pipeline.environment == "staging" {
                Some(pipeline.environment.to_string())
            } else {
                None
            }
        })
        .collect();

    // Filter out duplicates:
    namespace_selections.dedup();

    let mut namespace_selections_list = StatefulList::with_items(namespace_selections.clone());
    namespace_selections_list.select(0);

    // only select the first one if the current_namespace is None
    if app_ref.state.current_namespace.is_none() {
        app_ref.state.current_namespace = namespace_selections.first().cloned();
    }
    app_ref.namespace_selections = namespace_selections_list;

    app_ref.state.is_checking_namespaces = false;
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
                ..Default::default()
            },
            gcloud_access_token,
        )
        .await
}

async fn update_logs_entries(app: Arc<Mutex<App>>, log_entries: Option<Vec<LogEntry>>, id: &str) {
    if let Some(entries) = log_entries {
        if !entries.is_empty() {
            let mut app_ref = app.lock().await;

            // if the id are not same, we don't need to update
            // it was the old api call
            if app_ref.state.log_entries.0 != id {
                return;
            }

            app_ref.state.last_log_entry_timestamp = Some(
                entries
                    .last()
                    .unwrap()
                    .timestamp
                    .clone()
                    .unwrap_or_default()
                    .to_string(),
            );

            // Keep the log entries length to the max_log_entries_length:
            if app_ref.state.log_entries_length + entries.len() > MAX_LOG_ENTRIES_LENGTH {
                let excess =
                    (app_ref.state.log_entries_length + entries.len()) - MAX_LOG_ENTRIES_LENGTH;

                if excess > 0 {
                    app_ref.state.log_entries.1.drain(..excess);
                }
            }

            app_ref.state.log_entries.1.extend(entries);
            app_ref.state.log_entries_length = app_ref.state.log_entries.1.len();

            // Currently ratatui don't provide scroll to bottom function,
            // so we need to set the scroll to the bottom manually by this hack
            // waiting this https://github.com/fdehau/tui-rs/issues/89
            if app_ref.state.logs_enable_auto_scroll_to_bottom {
                let widget_height = app_ref.state.logs_widget_height - 4;

                app_ref.state.logs_vertical_scroll =
                    if app_ref.state.log_entries_length > widget_height as usize {
                        app_ref.state.log_entries_length - widget_height as usize
                    } else {
                        0
                    };

                app_ref.state.logs_vertical_scroll_state = app_ref
                    .state
                    .logs_vertical_scroll_state
                    .position(app_ref.state.log_entries_length as u16);
            }
        }
    }
}

async fn get_builds(app: Arc<Mutex<App>>, wk_client: &mut WKClient) -> Result<(), WKCliError> {
    let mut app_ref = app.lock().await;
    app_ref.state.is_fetching_builds = true;

    let application = app_ref.state.current_application.clone();
    let namespace = if let Some(namespace) = &app_ref.state.current_namespace {
        namespace.clone()
    } else {
        return Ok(());
    };

    let version = if let Some(version) = &app_ref.state.current_version {
        version.clone()
    } else {
        return Ok(());
    };

    drop(app_ref);

    let mut builds = vec![];

    let cd_pipeline_data = match wk_client
        .fetch_cd_pipeline(&application, &namespace, &version)
        .await
    {
        Ok(resp) => Ok(resp),
        Err(err) => {
            let mut app_ref = app.lock().await;
            app_ref.state.builds_error = Some(format!("{err}"));
            Err(err)
        }
    }?
    .cd_pipeline;

    if let Some(cd_pipeline_data) = cd_pipeline_data {
        builds = cd_pipeline_data
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
    }

    let mut app_ref = app.lock().await;

    app_ref.state.builds = builds;
    app_ref.state.is_fetching_builds = false;

    Ok(())
}

async fn get_deployments(app: Arc<Mutex<App>>, wk_client: &mut WKClient) -> Result<(), WKCliError> {
    let mut app_ref = app.lock().await;
    let application = app_ref.state.current_application.clone();

    app_ref.state.is_fetching_deployments = true;
    app_ref.state.is_checking_namespaces = true;
    app_ref.state.is_checking_version = true;

    drop(app_ref);

    let cd_pipelines_data = match wk_client.fetch_cd_pipelines(&application).await {
        Ok(resp) => Ok(resp),
        Err(err) => {
            let mut app_ref = app.lock().await;
            app_ref.state.deployments_error = Some(format!("{err}"));
            Err(err)
        }
    }?
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

    // we only know the available namespaces & versions after the deployments is fetched
    // so updated namespace selections & version selections are here
    set_namespace_selections(&mut app_ref);
    set_version_selections(&mut app_ref);

    app_ref.state.is_checking_namespaces = false;
    Ok(())
}

async fn get_gcloud_logs(app: Arc<Mutex<App>>, wk_client: &mut WKClient) -> Result<(), WKCliError> {
    let app_ref = app.lock().await;
    let application = app_ref.state.current_application.clone();
    let version = app_ref.state.current_version.clone();
    let namespace = app_ref.state.current_namespace.clone();
    let time_filter = app_ref.state.current_time_filter;
    let logs_severity = app_ref.state.logs_severity;

    let id = app_ref.state.log_entries.0.clone();

    let since = match app_ref.state.last_log_entry_timestamp.clone() {
        Some(t) => Some(t),
        None => match time_filter {
            Some(time_filter) => Some(time_filter.to_string()),
            None => Some("5m".to_string()),
        },
    };

    drop(app_ref);

    let gcloud_access_token = auth::google_cloud::get_token_or_login().await;

    if let Some(namespace) = namespace {
        if let Some(version) = version {
            let application_resp = match wk_client
                .fetch_application_with_k8s_cluster(&application, &namespace, &version)
                .await
            {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    let mut app_ref = app.lock().await;
                    app_ref.state.log_entries_error = Some(format!("{err}"));
                    Err(err)
                }
            }?
            .application;

            if let Some(application_data) = application_resp {
                if let Some(cluster) = application_data.k8s_cluster {
                    let filter = generate_filter(
                        &version,
                        &cluster.cluster_name,
                        &cluster.k8s_namespace,
                        &since,
                        &None,
                        &logs_severity,
                    )?;
                    let resource_names = vec![format!("projects/{}", cluster.google_project_id)];

                    let mut log = match fetch_log_entries(
                        Some(resource_names.clone()),
                        Some(500),
                        Some(filter.clone()),
                        None,
                        gcloud_access_token.clone(),
                        wk_client,
                    )
                    .await
                    {
                        Ok(data) => data,
                        Err(error) => {
                            let mut app_ref = app.lock().await;
                            app_ref.state.log_entries_error = Some(format!("{error}"));
                            return Err(error);
                        }
                    };

                    let mut next_page_token = log.next_page_token.clone();
                    update_logs_entries(Arc::clone(&app), log.entries, &id).await;

                    // repeat fetching logs until there is no next_page_token
                    let app = Arc::clone(&app);
                    while next_page_token.is_some() && next_page_token != Some("".to_string()) {
                        log = fetch_log_entries(
                            Some(resource_names.clone()),
                            Some(500),
                            Some(filter.clone()),
                            log.next_page_token.clone(),
                            gcloud_access_token.clone(),
                            wk_client,
                        )
                        .await
                        .unwrap();

                        // update next_page_token value
                        next_page_token = log.next_page_token.clone();

                        update_logs_entries(Arc::clone(&app), log.entries, &id).await;
                    }
                }
            }
        }
    }

    let mut app_ref = app.lock().await;
    app_ref.state.is_fetching_log_entries = false;
    Ok(())
}
