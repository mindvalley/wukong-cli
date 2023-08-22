use std::sync::Arc;

use tokio::sync::Mutex;
use wukong_sdk::services::gcloud::LogEntriesOptions;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkEvent {
    FetchBuilds,
    FetchDeployments,
    FetchGCloudLogs,
}

pub struct NetworkManager {
    app: Arc<Mutex<App>>,
}

impl NetworkManager {
    pub fn new(app: Arc<Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn handle_network_event(
        &mut self,
        network_event: NetworkEvent,
    ) -> Result<(), WKCliError> {
        match network_event {
            NetworkEvent::FetchBuilds => {
                let mut app = self.app.lock().await;
                let application = app.state.current_application.clone();
                let namespace = app.state.current_namespace.clone();
                app.state.is_fetching_builds = true;

                drop(app);

                let config = Config::load_from_default_path()?;
                let mut wk_client = WKClient::new(&config)?;

                let cd_pipeline_data = wk_client
                    .fetch_cd_pipeline(&application, &namespace, "green")
                    .await?
                    .cd_pipeline;

                if let Some(cd_pipeline_data) = cd_pipeline_data {
                    let mut app = self.app.lock().await;
                    app.state.builds = cd_pipeline_data
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
                    let mut app = self.app.lock().await;
                    app.state.builds = vec![];
                }

                let mut app = self.app.lock().await;
                app.state.is_fetching_builds = false;
            }
            NetworkEvent::FetchDeployments => {
                let mut app = self.app.lock().await;
                let application = app.state.current_application.clone();
                app.state.is_fetching_deployments = true;
                app.state.is_checking_namespaces = true;

                drop(app);

                let config = Config::load_from_default_path()?;
                let mut wk_client = WKClient::new(&config)?;

                let cd_pipelines_data = wk_client
                    .fetch_cd_pipelines(&application)
                    .await?
                    .cd_pipelines;

                let mut app = self.app.lock().await;
                app.state.deployments = cd_pipelines_data
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

                app.state.is_fetching_deployments = false;

                // we only know the available namespaces after the deployments is fetched
                // so update namespace selections here
                let has_prod_namespace = app
                    .state
                    .deployments
                    .iter()
                    .any(|pipeline| pipeline.environment == "prod");
                let has_staging_namespace = app
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
                app.namespace_selections = namespace_selections;
                app.state.is_checking_namespaces = false;
            }
            NetworkEvent::FetchGCloudLogs => {
                let mut app = self.app.lock().await;
                let application = app.state.current_application.clone();
                let namespace = app.state.current_namespace.clone();
                let version = "green";
                app.state.is_fetching_logs = true;
                drop(app);

                let config = Config::load_from_default_path()?;
                let mut wk_client = WKClient::new(&config)?;

                let gcloud_access_token = auth::google_cloud::get_token_or_login().await;

                let application_resp = wk_client
                    .fetch_application_with_k8s_cluster(&application, &namespace, &version)
                    .await?
                    .application;

                if let Some(application_data) = application_resp {
                    if let Some(cluster) = application_data.k8s_cluster {
                        let filter = generate_filter(
                            version,
                            &cluster.cluster_name,
                            &cluster.k8s_namespace,
                            &None,
                            &None,
                            &true,
                        )?;
                        let resource_names =
                            vec![format!("projects/{}", cluster.google_project_id)];

                        let log = wk_client
                            .get_gcloud_log_entries(
                                LogEntriesOptions {
                                    resource_names: Some(resource_names),
                                    page_size: Some(500),
                                    filter: Some(filter),
                                    ..Default::default()
                                },
                                gcloud_access_token,
                            )
                            .await?;

                        let mut app = self.app.lock().await;
                        app.state.log_entries = log.entries.unwrap_or_default();
                    }
                }

                let mut app = self.app.lock().await;
                app.state.is_fetching_logs = false;

                // let log = wk_client
                //     .get_gcloud_log_entries(
                //         LogEntriesOptions {
                //             resource_names: Some(resource_names),
                //             page_size: Some(*limit),
                //             filter: Some(filter),
                //             ..Default::default()
                //         },
                //         gcloud_access_token,
                //     )
                //     .await?;
            }
        }

        Ok(())
    }
}
