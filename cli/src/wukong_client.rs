use std::collections::HashMap;

use crate::{config::Config, error::WKCliError};
use wukong_sdk::{
    graphql::{
        application_query, application_with_k8s_cluster_query, applications_query,
        cd_pipeline_for_rollback_query, cd_pipeline_query, cd_pipelines_query, changelogs_query,
        ci_status_query, deploy_livebook, destroy_livebook, execute_cd_pipeline,
        is_authorized_query, kubernetes_pods_query, livebook_resource_query,
        multi_branch_pipeline_query, pipeline_query, pipelines_query,
    },
    services::{
        gcloud::{LogEntries, LogEntriesOptions},
        vault::client::FetchSecretsData,
    },
    WKClient as WKSdkClient, WKConfig,
};

use wukong_telemetry::*;
use wukong_telemetry_macro::*;

pub struct WKClient {
    inner: WKSdkClient,
    // for telemetry
    sub: Option<String>,
}

impl WKClient {
    pub fn new(config: &Config) -> Self {
        Self {
            inner: WKSdkClient::new(WKConfig {
                api_url: config.core.wukong_api_url.clone(),
                access_token: config.auth.clone().map(|auth| auth.id_token),
            }),
            sub: config.auth.clone().map(|auth| auth.subject),
        }
    }

    #[wukong_telemetry(api_event = "fetch_application_list")]
    pub async fn fetch_applications(&self) -> Result<applications_query::ResponseData, WKCliError> {
        self.inner.fetch_applications().await
    }

    #[wukong_telemetry(api_event = "fetch_application")]
    pub async fn fetch_application(
        &self,
        name: &str,
    ) -> Result<application_query::ResponseData, WKCliError> {
        self.inner.fetch_application(name).await
    }

    #[wukong_telemetry(api_event = "fetch_pipeline_list")]
    pub async fn fetch_pipelines(
        &self,
        application: &str,
    ) -> Result<pipelines_query::ResponseData, WKCliError> {
        self.inner.fetch_pipelines(application).await
    }

    #[wukong_telemetry(api_event = "fetch_pipeline")]
    pub async fn fetch_pipeline(
        &self,
        name: &str,
    ) -> Result<pipeline_query::ResponseData, WKCliError> {
        self.inner.fetch_pipeline(name).await
    }

    #[wukong_telemetry(api_event = "fetch_multi_branch_pipeline")]
    pub async fn fetch_multi_branch_pipeline(
        &self,
        name: &str,
    ) -> Result<multi_branch_pipeline_query::ResponseData, WKCliError> {
        self.inner.fetch_multi_branch_pipeline(name).await
    }

    // TODO: Error handling
    #[wukong_telemetry(api_event = "fetch_ci_status")]
    pub async fn fetch_ci_status(
        &self,
        repo_url: &str,
        branch: &str,
    ) -> Result<ci_status_query::ResponseData, WKCliError> {
        self.inner.fetch_ci_status(repo_url, branch).await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline_list")]
    pub async fn fetch_cd_pipelines(
        &self,
        application: &str,
    ) -> Result<cd_pipelines_query::ResponseData, WKCliError> {
        self.inner.fetch_cd_pipelines(application).await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline")]
    pub async fn fetch_cd_pipeline(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_query::ResponseData, WKCliError> {
        self.inner
            .fetch_cd_pipeline(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_changelogs")]
    pub async fn fetch_changelogs(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
    ) -> Result<changelogs_query::ResponseData, WKCliError> {
        self.inner
            .fetch_changelogs(application, namespace, version, build_artifact_name)
            .await
    }

    #[wukong_telemetry(api_event = "execute_cd_pipeline")]
    pub async fn deploy_cd_pipeline_build(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
        changelogs: Option<String>,
        send_to_slack: bool,
    ) -> Result<execute_cd_pipeline::ResponseData, WKCliError> {
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
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_for_rollback_query::ResponseData, WKCliError> {
        self.inner
            .fetch_previous_cd_pipeline_build(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_is_authorized")]
    pub async fn fetch_is_authorized(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<is_authorized_query::ResponseData, WKCliError> {
        self.inner
            .fetch_is_authorized(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_kubernetes_pods")]
    pub async fn fetch_kubernetes_pods(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<kubernetes_pods_query::ResponseData, WKCliError> {
        self.inner
            .fetch_kubernetes_pods(application, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "livebook_resource")]
    pub async fn check_livebook_resource(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<livebook_resource_query::ResponseData, WKCliError> {
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
        self.inner
            .deploy_livebook(application, namespace, version, name, port)
            .await
    }

    #[wukong_telemetry(api_event = "destroy_livebook")]
    pub async fn destroy_livebook(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<destroy_livebook::ResponseData, WKCliError> {
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
        self.inner
            .fetch_application_with_k8s_cluster(name, namespace, version)
            .await
    }

    #[wukong_telemetry(api_event = "fetch_gcloud_log_entries")]
    pub async fn get_gcloud_log_entries(
        &self,
        optons: LogEntriesOptions,
        access_token: String,
    ) -> Result<LogEntries, WKCliError> {
        self.inner
            .get_gcloud_log_entries(optons, access_token)
            .await
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
}
