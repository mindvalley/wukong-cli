use crate::{
    auth,
    config::{self, ApiChannel, Config},
    error::WKCliError,
};
use log::debug;
use std::collections::HashMap;
use wukong_sdk::{
    graphql::{
        application_config_query, application_query, application_with_k8s_cluster_query,
        applications_query, appsignal::AppsignalIncidentState, appsignal_apps_query,
        appsignal_average_error_rate_query, appsignal_average_latency_query,
        appsignal_average_throughput_query, appsignal_deploy_markers_query,
        appsignal_exception_incidents_query, cd_pipeline_for_rollback_query,
        cd_pipeline_github_query, cd_pipeline_query, cd_pipelines_query, changelogs_query,
        deploy_livebook, deployment::cd_pipeline_status_query, destroy_livebook,
        execute_cd_pipeline, github_workflow_templates_query, is_authorized_query,
        kubernetes_pods_query, livebook_resource_query, AppsignalTimeFrame,
    },
    services::{
        gcloud::{DatabaseMetrics, LogEntries, LogEntriesOptions, TokenInfo},
        vault::client::FetchSecretsData,
    },
    WKClient as WKSdkClient, WKConfig,
};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

pub struct WKClient {
    inner: WKSdkClient,
    // for telemetry
    sub: String,
    // for refresh tokens
    config: Config,
}

impl From<config::ApiChannel> for wukong_sdk::ApiChannel {
    fn from(channel: config::ApiChannel) -> Self {
        match channel {
            config::ApiChannel::Canary => wukong_sdk::ApiChannel::Canary,
            config::ApiChannel::Stable => wukong_sdk::ApiChannel::Stable,
        }
    }
}
impl WKClient {
    pub fn for_channel(config: &Config, channel: &ApiChannel) -> Result<Self, WKCliError> {
        let auth_config = config
            .auth
            .okta
            .as_ref()
            .ok_or(WKCliError::UnAuthenticated)?;

        Ok(Self {
            inner: WKSdkClient::new(WKConfig {
                api_url: config.core.wukong_api_url.clone(),
                access_token: auth_config.id_token.clone(),
                channel: channel.clone().into(),
            }),
            sub: auth_config.subject.clone(),
            config: config.clone(),
        })
    }

    async fn check_and_refresh_tokens(&mut self) -> Result<(), WKCliError> {
        if auth::okta::need_tokens_refresh(&self.config)? {
            debug!("Access token expired. Refreshing tokens...");

            let new_tokens = auth::okta::refresh_tokens(&self.config).await?;
            self.config.auth.okta = Some(new_tokens.clone().into());

            // update config file
            self.config.save_to_default_path()?;

            // update WKClient to the new tokens
            self.inner.set_access_token(new_tokens.id_token);
            self.sub = new_tokens.subject;

            debug!("The token is refreshed now.");
        }

        Ok(())
    }

    #[wukong_telemetry(api_event = "fetch_application_list")]
    pub async fn fetch_applications(
        &mut self,
    ) -> Result<applications_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner.fetch_applications().await
    }

    #[wukong_telemetry(api_event = "fetch_application")]
    pub async fn fetch_application(
        &mut self,
        name: &str,
    ) -> Result<application_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner.fetch_application(name).await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline_list")]
    pub async fn fetch_cd_pipelines(
        &mut self,
        application: &str,
    ) -> Result<cd_pipelines_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner.fetch_cd_pipelines(application).await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline_github")]
    pub async fn fetch_cd_pipeline_github(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_github_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_cd_pipeline_github(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline")]
    pub async fn fetch_cd_pipeline(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_cd_pipeline(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline_status")]
    pub async fn fetch_cd_pipeline_status(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_status_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_cd_pipeline_status(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_changelogs")]
    pub async fn fetch_changelogs(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
    ) -> Result<changelogs_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_changelogs(application, namespace, version, build_artifact_name)
            .await
    }

    #[wukong_telemetry(api_event = "execute_cd_pipeline")]
    pub async fn deploy_cd_pipeline_build(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
        changelogs: Option<String>,
        send_to_slack: bool,
    ) -> Result<execute_cd_pipeline::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .deploy_cd_pipeline_build(
                application,
                namespace,
                version,
                build_artifact_name,
                changelogs,
                send_to_slack,
            )
            .await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline_for_rollback")]
    pub async fn fetch_previous_cd_pipeline_build(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_for_rollback_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_previous_cd_pipeline_build(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_is_authorized")]
    pub async fn fetch_is_authorized(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<is_authorized_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_is_authorized(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_kubernetes_pods")]
    pub async fn fetch_kubernetes_pods(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<kubernetes_pods_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_kubernetes_pods(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "livebook_resource")]
    pub async fn check_livebook_resource(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<livebook_resource_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .check_livebook_resource(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "deploy_livebook")]
    pub async fn deploy_livebook(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
        name: &str,
        port: i64,
    ) -> Result<deploy_livebook::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .deploy_livebook(application, namespace, version, name, port)
            .await
    }

    #[wukong_telemetry(api_event = "destroy_livebook")]
    pub async fn destroy_livebook(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<destroy_livebook::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .destroy_livebook(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_application_with_k8s_cluster")]
    pub async fn fetch_application_with_k8s_cluster(
        &mut self,
        name: &str,
        namespace: &str,
        version: &str,
    ) -> Result<application_with_k8s_cluster_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_application_with_k8s_cluster(name, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_gcloud_log_entries")]
    pub async fn fetch_gcloud_log_entries(
        &self,
        options: LogEntriesOptions,
        access_token: String,
    ) -> Result<LogEntries, WKCliError> {
        self.inner
            .fetch_gcloud_log_entries(options, access_token)
            .await
    }

    #[wukong_telemetry(api_event = "get_access_token_info")]
    pub async fn fetch_access_token_info(
        &self,
        access_token: String,
    ) -> Result<TokenInfo, WKCliError> {
        self.inner.fetch_access_token_info(access_token).await
    }

    #[wukong_telemetry(api_event = "fetch_vault_secrets")]
    pub async fn get_secrets(
        &self,
        api_token: &str,
        path: &str,
    ) -> Result<FetchSecretsData, WKCliError> {
        self.inner.get_secrets(api_token, path).await
    }

    pub async fn get_secret(
        &self,
        api_token: &str,
        path: &str,
        key: &str,
    ) -> Result<String, WKCliError> {
        self.inner
            .get_secret(api_token, path, key)
            .await
            .map_err(|err| err.into())
    }

    #[wukong_telemetry(api_event = "update_vault_secrets")]
    pub async fn update_secret(
        &self,
        api_token: &str,
        path: &str,
        data: &HashMap<&str, &str>,
    ) -> Result<bool, WKCliError> {
        self.inner.update_secret(api_token, path, data).await
    }

    #[wukong_telemetry(api_event = "fetch_appsignal_average_error_rate")]
    pub async fn fetch_appsignal_average_error_rate(
        &mut self,
        app_id: &str,
        namespace: &str,
        start: &str,
        until: &str,
        timeframe: AppsignalTimeFrame,
    ) -> Result<appsignal_average_error_rate_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_appsignal_average_error_rate(app_id, namespace, start, until, timeframe)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_appsignal_average_latency")]
    pub async fn fetch_appsignal_average_latency(
        &mut self,
        app_id: &str,
        namespace: &str,
        start: &str,
        until: &str,
        timeframe: AppsignalTimeFrame,
    ) -> Result<appsignal_average_latency_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_appsignal_average_latency(app_id, namespace, start, until, timeframe)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_appsignal_average_throughput")]
    pub async fn fetch_appsignal_average_throughput(
        &mut self,
        app_id: &str,
        namespace: &str,
        start: &str,
        until: &str,
        timeframe: AppsignalTimeFrame,
    ) -> Result<appsignal_average_throughput_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_appsignal_average_throughput(app_id, namespace, start, until, timeframe)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_appsignal_apps")]
    pub async fn fetch_appsignal_apps(
        &mut self,
    ) -> Result<appsignal_apps_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner.fetch_appsignal_apps().await
    }

    #[wukong_telemetry(api_event = "fetch_appsignal_deploy_markers")]
    pub async fn fetch_appsignal_deploy_markers(
        &mut self,
        app_id: &str,
        limit: Option<i64>,
    ) -> Result<appsignal_deploy_markers_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_appsignal_deploy_markers(app_id, limit)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_appsignal_exception_incidents")]
    pub async fn fetch_appsignal_exception_incidents(
        &mut self,
        app_id: &str,
        namespaces: Vec<String>,
        limit: Option<i64>,
        marker: Option<String>,
        state: Option<AppsignalIncidentState>,
    ) -> Result<appsignal_exception_incidents_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner
            .fetch_appsignal_exception_incidents(app_id, namespaces, limit, marker, state)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_gcloud_database_metrics")]
    pub async fn fetch_gcloud_database_metrics(
        &self,
        project_id: &str,
        access_token: String,
    ) -> Result<Vec<DatabaseMetrics>, WKCliError> {
        self.inner
            .fetch_gcloud_database_metrics(project_id, access_token)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_application_config")]
    pub async fn fetch_application_config(
        &mut self,
        name: &str,
    ) -> Result<application_config_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner.fetch_application_config(name).await
    }

    #[wukong_telemetry(api_event = "fetch_github_workflow_templates")]
    pub async fn fetch_github_workflow_templates(
        &mut self,
    ) -> Result<github_workflow_templates_query::ResponseData, WKCliError> {
        self.check_and_refresh_tokens().await?;
        self.inner.fetch_github_workflow_templates().await
    }
}
