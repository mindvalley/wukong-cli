use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    commands::tui::{
        app::{App, Build, Commit, Deployment},
        StatefulList,
    },
    config::{ApiChannel, Config},
    error::WKCliError,
    wukong_client::WKClient,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkEvent {
    FetchBuilds,
    FetchDeployments,
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
        channel: &ApiChannel,
    ) -> Result<(), WKCliError> {
        let config = Config::load_from_default_path()?;
        let mut wk_client = WKClient::for_channel(&config, channel)?;

        match network_event {
            NetworkEvent::FetchBuilds => {
                let mut app = self.app.lock().await;
                let application = app.state.current_application.clone();
                let namespace = app.state.current_namespace.clone();
                app.state.is_fetching_builds = true;

                drop(app);

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
        }

        Ok(())
    }
}
