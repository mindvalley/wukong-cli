use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    commands::tui::app::{App, Build, Commit},
    config::Config,
    error::WKCliError,
    wukong_client::WKClient,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkEvent {
    FetchBuilds,
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
        let mut app = self.app.lock().await;

        match network_event {
            NetworkEvent::FetchBuilds => {
                app.state.is_fetching_builds = true;
                let config = Config::load_from_default_path()?;
                let mut wk_client = WKClient::new(&config)?;

                let cd_pipeline_data = wk_client
                    .fetch_cd_pipeline(
                        &app.state.current_application,
                        &app.state.current_namespace,
                        "green",
                    )
                    .await?
                    .cd_pipeline;

                if let Some(cd_pipeline_data) = cd_pipeline_data {
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
                    app.state.builds = vec![];
                }

                app.state.is_fetching_builds = false;
            }
        }

        Ok(())
    }
}
