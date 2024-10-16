use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};
use wukong_sdk::{
    graphql::AppsignalTimeFrame,
    services::gcloud::{google::logging::v2::LogEntry, LogEntries, LogEntriesOptions},
};

use crate::{
    application_config::{
        ApplicationConfig, ApplicationConfigs, ApplicationNamespaceAppsignalConfig,
    },
    auth::{self, okta::introspect_token},
    commands::{
        application::generate_filter,
        tui::{
            app::{
                App, AppsignalAverageLatecies, AppsignalState, Build, Commit, Deployment,
                MAX_LOG_ENTRIES_LENGTH,
            },
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
    GetGCloudLogsTail,
    GetAppsignalData,
    GetDatabaseMetrics,
    VerifyOktaRefreshToken,
    VerifyGCloudToken,
}

pub async fn handle_network_event(
    app: Arc<Mutex<App>>,
    network_event: NetworkEvent,
    channel: Arc<ApiChannel>,
    config: Arc<Config>,
) -> Result<(), WKCliError> {
    let mut wk_client = WKClient::for_channel(&config, &channel)?;

    match network_event {
        NetworkEvent::GetBuilds => get_builds(app, &mut wk_client).await?,
        NetworkEvent::GetDeployments => get_deployments(app, &mut wk_client).await?,
        NetworkEvent::GetGCloudLogs => get_gcloud_logs(app, &mut wk_client, false).await?,
        NetworkEvent::GetGCloudLogsTail => get_gcloud_logs(app, &mut wk_client, true).await?,
        NetworkEvent::VerifyOktaRefreshToken => verify_okta_refresh_token(app).await?,
        NetworkEvent::VerifyGCloudToken => verify_gcloud_token(app, &mut wk_client).await?,
        NetworkEvent::GetAppsignalData => fetch_appsignal_data(app, channel, config).await?,
        NetworkEvent::GetDatabaseMetrics => get_database_metrics(app, &mut wk_client).await?,
    }

    Ok(())
}

async fn verify_okta_refresh_token(app: Arc<Mutex<App>>) -> Result<(), WKCliError> {
    let mut app_ref = app.lock().await;
    match Config::load_from_default_path() {
        Ok(config) => {
            let okta_config = config.auth.okta.as_ref();

            if let Some(okta_config) = okta_config {
                let token = introspect_token(&config, &okta_config.refresh_token).await?;

                if token.active {
                    app_ref.state.is_okta_authenticated = Some(true);
                } else {
                    app_ref.state.is_okta_authenticated = Some(false);
                }
            } else {
                app_ref.state.is_okta_authenticated = Some(false);
            }
        }
        Err(_error) => {
            app_ref.state.is_okta_authenticated = Some(false);
        }
    };

    Ok(())
}

async fn verify_gcloud_token(
    app: Arc<Mutex<App>>,
    wk_client: &mut WKClient,
) -> Result<(), WKCliError> {
    let gcloud_access_token = auth::google_cloud::get_access_token().await;

    if let Some(gcloud_access_token) = gcloud_access_token {
        let mut app_ref = app.lock().await;
        match wk_client.fetch_access_token_info(gcloud_access_token).await {
            Ok(_token_info) => {
                app_ref.state.is_gcloud_authenticated = Some(true);
            }
            Err(_error) => {
                app_ref.state.is_gcloud_authenticated = Some(false);
            }
        }
    } else {
        let mut app_ref = app.lock().await;
        app_ref.state.is_gcloud_authenticated = Some(false);
    }

    Ok(())
}

fn set_default_version_selections(app_ref: &mut MutexGuard<'_, App>) {
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

fn set_default_namespace_selections(app_ref: &mut MutexGuard<'_, App>) {
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
    order_by: Option<String>,
    gcloud_access_token: String,
    wk_client: &mut WKClient,
) -> Result<LogEntries, WKCliError> {
    wk_client
        .fetch_gcloud_log_entries(
            LogEntriesOptions {
                resource_names,
                page_size,
                filter,
                page_token,
                order_by,
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
                // for some rare cases where the log widget is not rendered and the height is 0
                // so add a check here to prevent overflow
                if app_ref.state.logs_widget_height < 4 {
                    return;
                }

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
        .fetch_cd_pipeline_github(&application, &namespace, &version)
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
            .github_builds
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
    set_default_namespace_selections(&mut app_ref);
    set_default_version_selections(&mut app_ref);

    app_ref.state.is_checking_namespaces = false;
    Ok(())
}

async fn get_gcloud_logs(
    app: Arc<Mutex<App>>,
    wk_client: &mut WKClient,
    tailing: bool,
) -> Result<(), WKCliError> {
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

    let gcloud_access_token = auth::google_cloud::get_token_or_login(None).await;

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

                    let (page_size, order_by) = match tailing {
                        true => (1000, None),
                        false => (100, Some("timestamp desc".to_string())),
                    };
                    let mut log = match fetch_log_entries(
                        Some(resource_names.clone()),
                        Some(page_size),
                        Some(filter.clone()),
                        None,
                        order_by,
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

                    if !tailing {
                        log.entries = log
                            .entries
                            .map(|entries| entries.into_iter().rev().collect::<Vec<LogEntry>>());
                    }

                    update_logs_entries(Arc::clone(&app), log.entries, &id).await;
                }
            }
        }
    }

    let mut app_ref = app.lock().await;
    app_ref.state.is_fetching_log_entries = false;
    Ok(())
}

async fn fetch_appsignal_data(
    app: Arc<Mutex<App>>,
    channel: Arc<ApiChannel>,
    config: Arc<Config>,
) -> Result<(), WKCliError> {
    let appsignal_state = Arc::new(Mutex::new(AppsignalState::default()));

    let mut app_ref = app.lock().await;

    let current_namespace = app_ref.state.current_namespace.clone().unwrap();
    let is_appsignal_enabled = app_ref.state.is_appsignal_enabled;

    // if appsignal is not loaded, load it from the config
    // if the config is not available or not enabled, then appsignal is not enabled
    if is_appsignal_enabled.is_none() {
        let application_configs = ApplicationConfigs::load();

        if let Err(error) = application_configs {
            app_ref.state.appsignal_error = Some(format!("{error}"));
            app_ref.state.is_fetching_appsignal_data = false;

            // return early when error
            return Ok(());
        }

        if let ApplicationConfig {
            enable: true,
            namespaces,
            ..
        } = application_configs.unwrap().application
        {
            let appsignal_config = namespaces
                .iter()
                .find(|ns| ns.namespace_type == current_namespace)
                .and_then(|ns| ns.appsignal.clone());

            if let Some(ApplicationNamespaceAppsignalConfig {
                enable,
                app_id,
                environment: _,
                default_namespace,
            }) = appsignal_config
            {
                app_ref.state.is_appsignal_enabled = Some(enable);
                app_ref.state.appsignal_app_id = Some(app_id);
                app_ref.state.appsignal_namespace = Some(default_namespace);
            } else {
                app_ref.state.is_appsignal_enabled = None;
                app_ref.state.is_fetching_appsignal_data = false;
                // return early if appsignal is not setup
                return Ok(());
            }
        } else {
            app_ref.state.is_appsignal_enabled = Some(false);
            app_ref.state.is_fetching_appsignal_data = false;
            // return early if appsignal is not enabled
            return Ok(());
        }
    }

    let app_id = Arc::new(app_ref.state.appsignal_app_id.clone().unwrap());
    let namespace = Arc::new(app_ref.state.appsignal_namespace.clone().unwrap());
    drop(app_ref);

    let start = "2023-06-01T00:00:00.000Z";
    let until = "2023-12-29T00:00:00.000Z";

    let app_id_clone = Arc::clone(&app_id);
    let namespace_clone = Arc::clone(&namespace);
    let channel_clone = Arc::clone(&channel);
    let config_clone = Arc::clone(&config);
    let appsignal_state_clone = Arc::clone(&appsignal_state);

    let t1: tokio::task::JoinHandle<Result<(), WKCliError>> = tokio::spawn(async move {
        let mut wk_client = WKClient::for_channel(&config_clone, &channel_clone)?;
        let average_error_rate_1h = wk_client
            .fetch_appsignal_average_error_rate(
                &app_id_clone,
                &namespace_clone,
                start,
                until,
                AppsignalTimeFrame::R1H,
            )
            .await?;

        appsignal_state_clone
            .lock()
            .await
            .average_error_rates
            .in_1_hour = match average_error_rate_1h.appsignal_error_rate {
            Some(error_rate) => error_rate.average[0].value,
            None => 0.0,
        };

        Ok(())
    });

    let app_id_clone = Arc::clone(&app_id);
    let namespace_clone = Arc::clone(&namespace);
    let channel_clone = Arc::clone(&channel);
    let config_clone = Arc::clone(&config);
    let appsignal_state_clone = Arc::clone(&appsignal_state);

    let t2: tokio::task::JoinHandle<Result<(), WKCliError>> = tokio::spawn(async move {
        let mut wk_client = WKClient::for_channel(&config_clone, &channel_clone)?;
        let average_error_rate_8h = wk_client
            .fetch_appsignal_average_error_rate(
                &app_id_clone,
                &namespace_clone,
                start,
                until,
                AppsignalTimeFrame::R8H,
            )
            .await?;

        appsignal_state_clone
            .lock()
            .await
            .average_error_rates
            .in_8_hours = match average_error_rate_8h.appsignal_error_rate {
            Some(error_rate) => error_rate.average[0].value,
            None => 0.0,
        };

        Ok(())
    });

    let app_id_clone = Arc::clone(&app_id);
    let namespace_clone = Arc::clone(&namespace);
    let channel_clone = Arc::clone(&channel);
    let config_clone = Arc::clone(&config);
    let appsignal_state_clone = Arc::clone(&appsignal_state);

    let t3: tokio::task::JoinHandle<Result<(), WKCliError>> = tokio::spawn(async move {
        let mut wk_client = WKClient::for_channel(&config_clone, &channel_clone)?;
        let average_error_rate_24h = wk_client
            .fetch_appsignal_average_error_rate(
                &app_id_clone,
                &namespace_clone,
                start,
                until,
                AppsignalTimeFrame::R24H,
            )
            .await?;

        appsignal_state_clone
            .lock()
            .await
            .average_error_rates
            .in_24_hours = match average_error_rate_24h.appsignal_error_rate {
            Some(error_rate) => error_rate.average[0].value,
            None => 0.0,
        };

        Ok(())
    });

    let app_id_clone = Arc::clone(&app_id);
    let namespace_clone = Arc::clone(&namespace);
    let channel_clone = Arc::clone(&channel);
    let config_clone = Arc::clone(&config);
    let appsignal_state_clone = Arc::clone(&appsignal_state);

    let t4: tokio::task::JoinHandle<Result<(), WKCliError>> = tokio::spawn(async move {
        let mut wk_client = WKClient::for_channel(&config_clone, &channel_clone)?;
        let average_throughput_1h = wk_client
            .fetch_appsignal_average_throughput(
                &app_id_clone,
                &namespace_clone,
                start,
                until,
                AppsignalTimeFrame::R1H,
            )
            .await?;

        appsignal_state_clone
            .lock()
            .await
            .average_throughputs
            .in_1_hour = match average_throughput_1h.appsignal_throughput {
            Some(throughput) => throughput.average[0].value,
            None => 0.0,
        };

        Ok(())
    });

    let app_id_clone = Arc::clone(&app_id);
    let namespace_clone = Arc::clone(&namespace);
    let channel_clone = Arc::clone(&channel);
    let config_clone = Arc::clone(&config);
    let appsignal_state_clone = Arc::clone(&appsignal_state);

    let t5: tokio::task::JoinHandle<Result<(), WKCliError>> = tokio::spawn(async move {
        let mut wk_client = WKClient::for_channel(&config_clone, &channel_clone)?;
        let average_throughput_8h = wk_client
            .fetch_appsignal_average_throughput(
                &app_id_clone,
                &namespace_clone,
                start,
                until,
                AppsignalTimeFrame::R8H,
            )
            .await?;

        appsignal_state_clone
            .lock()
            .await
            .average_throughputs
            .in_8_hours = match average_throughput_8h.appsignal_throughput {
            Some(throughput) => throughput.average[0].value,
            None => 0.0,
        };

        Ok(())
    });

    let app_id_clone = Arc::clone(&app_id);
    let namespace_clone = Arc::clone(&namespace);
    let channel_clone = Arc::clone(&channel);
    let config_clone = Arc::clone(&config);
    let appsignal_state_clone = Arc::clone(&appsignal_state);

    let t6: tokio::task::JoinHandle<Result<(), WKCliError>> = tokio::spawn(async move {
        let mut wk_client = WKClient::for_channel(&config_clone, &channel_clone)?;
        let average_throughput_24h = wk_client
            .fetch_appsignal_average_throughput(
                &app_id_clone,
                &namespace_clone,
                start,
                until,
                AppsignalTimeFrame::R24H,
            )
            .await?;

        appsignal_state_clone
            .lock()
            .await
            .average_throughputs
            .in_24_hours = match average_throughput_24h.appsignal_throughput {
            Some(throughput) => throughput.average[0].value,
            None => 0.0,
        };

        Ok(())
    });

    let app_id_clone = Arc::clone(&app_id);
    let namespace_clone = Arc::clone(&namespace);
    let channel_clone = Arc::clone(&channel);
    let config_clone = Arc::clone(&config);
    let appsignal_state_clone = Arc::clone(&appsignal_state);

    let t7: tokio::task::JoinHandle<Result<(), WKCliError>> = tokio::spawn(async move {
        let mut wk_client = WKClient::for_channel(&config_clone, &channel_clone)?;
        let average_latency = wk_client
            .fetch_appsignal_average_latency(
                &app_id_clone,
                &namespace_clone,
                start,
                until,
                AppsignalTimeFrame::R4H,
            )
            .await?;

        appsignal_state_clone.lock().await.average_latencies =
            match average_latency.appsignal_latency {
                Some(latency) => AppsignalAverageLatecies {
                    mean: latency.average[0].value.mean,
                    p90: latency.average[0].value.p90,
                    p95: latency.average[0].value.p95,
                },
                None => AppsignalAverageLatecies {
                    mean: 0.0,
                    p90: 0.0,
                    p95: 0.0,
                },
            };

        Ok(())
    });

    t1.await.unwrap()?;
    t2.await.unwrap()?;
    t3.await.unwrap()?;
    t4.await.unwrap()?;
    t5.await.unwrap()?;
    t6.await.unwrap()?;
    t7.await.unwrap()?;

    let mut app_ref = app.lock().await;

    app_ref.state.appsignal = appsignal_state.lock().await.clone();
    app_ref.state.is_fetching_appsignal_data = false;
    drop(app_ref);

    Ok(())
}

async fn get_database_metrics(
    app: Arc<Mutex<App>>,
    wk_client: &mut WKClient,
) -> Result<(), WKCliError> {
    let app_ref = app.lock().await;
    let application = app_ref.state.current_application.clone();
    let namespace = app_ref.state.current_namespace.clone();
    let version = app_ref.state.current_version.clone();

    drop(app_ref);

    let gcloud_access_token = auth::google_cloud::get_token_or_login(None).await;
    if let Some(namespace) = namespace {
        if let Some(version) = version {
            let application_resp = match wk_client
                .fetch_application_with_k8s_cluster(&application, &namespace, &version)
                .await
            {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    let mut app_ref = app.lock().await;
                    app_ref.state.databases.error = Some(format!("{err}"));
                    Err(err)
                }
            }?
            .application;

            if let Some(application_data) = application_resp {
                if let Some(cluster) = application_data.k8s_cluster {
                    let database_metrics = match wk_client
                        .fetch_gcloud_database_metrics(
                            &cluster.google_project_id,
                            gcloud_access_token,
                        )
                        .await
                    {
                        Ok(resp) => Ok(resp),
                        Err(err) => {
                            let mut app_ref = app.lock().await;
                            app_ref.state.databases.error = Some(format!("{err}"));
                            Err(err)
                        }
                    };
                    if let Ok(database_metrics) = database_metrics {
                        let mut app_ref = app.lock().await;
                        app_ref.state.databases.database_metrics = database_metrics;
                    }
                }
            }
        }
    };

    let mut app_ref = app.lock().await;
    app_ref.state.is_fetching_database_metrics = false;

    Ok(())
}
