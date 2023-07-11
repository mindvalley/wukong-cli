pub mod application;
pub mod changelog;
pub mod deployment;
pub mod kubernetes;
pub mod pipeline;

use self::{
    application::{
        application_query, application_with_k8s_cluster_query, applications_query,
        ApplicationQuery, ApplicationWithK8sClusterQuery, ApplicationsQuery,
    },
    changelog::{changelogs_query, ChangelogsQuery},
    deployment::{
        cd_pipeline_for_rollback_query, cd_pipeline_query, cd_pipelines_query, execute_cd_pipeline,
        CdPipelineForRollbackQuery, CdPipelineQuery, CdPipelinesQuery, ExecuteCdPipeline,
    },
    kubernetes::{
        deploy_livebook, destroy_livebook, is_authorized_query, kubernetes_pods_query,
        livebook_resource_query, DeployLivebook, DestroyLivebook, IsAuthorizedQuery,
        KubernetesPodsQuery, LivebookResourceQuery,
    },
    pipeline::{
        ci_status_query, multi_branch_pipeline_query, pipeline_query, pipelines_query,
        CiStatusQuery, MultiBranchPipelineQuery, PipelineQuery, PipelinesQuery,
    },
};
use crate::{
    error::APIError,
    telemetry::{self, TelemetryData, TelemetryEvent},
};
use graphql_client::{GraphQLQuery, QueryBody, Response};
use log::debug;
use reqwest::header;
use std::fmt::Debug;
use std::{thread, time};
use wukong_telemetry_macro::wukong_telemetry;

#[derive(Debug, Default)]
pub struct QueryClientBuilder {
    access_token: Option<String>,
    sub: Option<String>,
    api_url: String,
}

impl QueryClientBuilder {
    pub fn with_access_token(mut self, access_token: String) -> Self {
        self.access_token = Some(access_token);
        self
    }

    // for telemetry usage
    pub fn with_sub(mut self, sub: Option<String>) -> Self {
        self.sub = sub;
        self
    }

    pub fn with_api_url(mut self, api_url: String) -> Self {
        self.api_url = api_url;
        self
    }

    pub fn build(self) -> Result<QueryClient, APIError> {
        let mut headers = header::HeaderMap::new();

        if let Some(token) = self.access_token.as_ref() {
            let auth_value = format!("Bearer {}", token);
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&auth_value).unwrap(),
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(QueryClient {
            reqwest_client: client,
            api_url: self.api_url,
            sub: self.sub,
        })
    }
}

pub struct QueryClient {
    reqwest_client: reqwest::Client,
    api_url: String,
    // for telemetry usage
    sub: Option<String>,
}

impl QueryClient {
    pub fn inner(&self) -> &reqwest::Client {
        &self.reqwest_client
    }

    pub async fn call_api<Q>(
        &self,
        variables: Q::Variables,
        handler: impl Fn(
            Response<Q::ResponseData>,
            graphql_client::Error,
        ) -> Result<Response<Q::ResponseData>, APIError>,
    ) -> Result<Response<Q::ResponseData>, APIError>
    where
        Q: GraphQLQuery,
        Q::ResponseData: Debug,
    {
        let body = Q::build_query(variables);

        debug!("url: {:?}", &self.api_url);
        debug!("query: \n{}", body.query);

        let response: Result<Response<Q::ResponseData>, APIError> =
            self.retry_request::<Q>(body, handler).await;

        response
    }

    // Attempts the request and retries up to 3 times if the request times out.
    async fn retry_request<Q>(
        &self,
        body: QueryBody<Q::Variables>,
        handler: impl Fn(
            Response<Q::ResponseData>,
            graphql_client::Error,
        ) -> Result<Response<Q::ResponseData>, APIError>,
    ) -> Result<Response<Q::ResponseData>, APIError>
    where
        Q: GraphQLQuery,
        Q::ResponseData: Debug,
    {
        let mut retry_count = 0;
        let request = self.inner().post(&self.api_url).json(&body);

        debug!("request: {:#?}", request);

        let mut response: Response<<Q as GraphQLQuery>::ResponseData> =
            request.send().await?.json().await?;

        debug!("response: {:#?}", response);

        // We use <= 3 so it does one extra loop where the last response is checked
        // in order to return an APIError::Timeout if it was a timeout error in the
        // case of it failing all 3 retries.
        while response.errors.is_some() && retry_count <= 3 {
            if let Some(errors) = response.errors.clone() {
                let first_error = errors[0].clone();

                match check_retry_and_auth_error(&first_error) {
                    Some(APIError::UnAuthenticated) => return Err(APIError::UnAuthenticated),
                    Some(APIError::Timeout { domain }) => {
                        if retry_count == 3 {
                            return Err(APIError::Timeout { domain });
                        }
                        retry_count += 1;
                        eprintln!(
                            "... request to {domain} timed out, retrying the request {}/3",
                            retry_count
                        );

                        thread::sleep(time::Duration::from_secs(5));

                        let request = self.inner().post(&self.api_url).json(&body);

                        debug!("request: {:#?}", request);

                        response = request.send().await?.json().await?;

                        debug!("response: {:#?}", response);
                    }
                    _ => return handler(response, first_error),
                }
            }
        }

        Ok(response)
    }

    #[wukong_telemetry(api_event = "fetch_pipeline_list")]
    pub async fn fetch_pipeline_list(
        &self,
        application: &str,
    ) -> Result<Response<pipelines_query::ResponseData>, APIError> {
        PipelinesQuery::fetch(self, application).await
    }

    #[wukong_telemetry(api_event = "fetch_pipeline")]
    pub async fn fetch_pipeline(
        &self,
        name: &str,
    ) -> Result<Response<pipeline_query::ResponseData>, APIError> {
        PipelineQuery::fetch(self, name).await
    }

    #[wukong_telemetry(api_event = "fetch_multi_branch_pipeline")]
    pub async fn fetch_multi_branch_pipeline(
        &self,
        name: &str,
    ) -> Result<Response<multi_branch_pipeline_query::ResponseData>, APIError> {
        MultiBranchPipelineQuery::fetch(self, name).await
    }

    #[wukong_telemetry(api_event = "fetch_ci_status")]
    pub async fn fetch_ci_status(
        &self,
        repo_url: &str,
        branch: &str,
    ) -> Result<Response<ci_status_query::ResponseData>, APIError> {
        CiStatusQuery::fetch(self, repo_url, branch).await
    }

    #[wukong_telemetry(api_event = "fetch_application_list")]
    pub async fn fetch_application_list(
        &self,
    ) -> Result<Response<applications_query::ResponseData>, APIError> {
        ApplicationsQuery::fetch(self).await
    }

    #[wukong_telemetry(api_event = "fetch_application")]
    pub async fn fetch_application(
        &self,
        name: &str,
    ) -> Result<Response<application_query::ResponseData>, APIError> {
        ApplicationQuery::fetch(self, name).await
    }

    #[wukong_telemetry(api_event = "fetch_application_with_k8s_cluster")]
    pub async fn fetch_application_with_k8s_cluster(
        &self,
        name: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<application_with_k8s_cluster_query::ResponseData>, APIError> {
        ApplicationWithK8sClusterQuery::fetch(self, name, namespace, version).await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline_list")]
    pub async fn fetch_cd_pipeline_list(
        &self,
        application: &str,
    ) -> Result<Response<cd_pipelines_query::ResponseData>, APIError> {
        CdPipelinesQuery::fetch(self, application).await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline")]
    pub async fn fetch_cd_pipeline(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<cd_pipeline_query::ResponseData>, APIError> {
        CdPipelineQuery::fetch(self, application, namespace, version).await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline_for_rollback")]
    pub async fn fetch_cd_pipeline_for_rollback(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<cd_pipeline_for_rollback_query::ResponseData>, APIError> {
        CdPipelineForRollbackQuery::fetch(self, application, namespace, version).await
    }

    #[wukong_telemetry(api_event = "execute_cd_pipeline")]
    pub async fn execute_cd_pipeline(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
        changelogs: Option<String>,
        send_to_slack: bool,
    ) -> Result<Response<execute_cd_pipeline::ResponseData>, APIError> {
        ExecuteCdPipeline::mutate(
            self,
            application,
            namespace,
            version,
            build_artifact_name,
            changelogs,
            send_to_slack,
        )
        .await
    }

    #[wukong_telemetry(api_event = "fetch_changelogs")]
    pub async fn fetch_changelogs(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
    ) -> Result<Response<changelogs_query::ResponseData>, APIError> {
        ChangelogsQuery::fetch(self, application, namespace, version, build_artifact_name).await
    }

    #[wukong_telemetry(api_event = "fetch_kubernetes_pods")]
    pub async fn fetch_kubernetes_pods(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<kubernetes_pods_query::ResponseData>, APIError> {
        KubernetesPodsQuery::fetch(self, application, namespace, version).await
    }

    #[wukong_telemetry(api_event = "fetch_is_authorized")]
    pub async fn fetch_is_authorized(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<is_authorized_query::ResponseData>, APIError> {
        IsAuthorizedQuery::fetch(self, application, namespace, version).await
    }

    #[wukong_telemetry(api_event = "deploy_livebook")]
    pub async fn deploy_livebook(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
        name: &str,
        port: i64,
    ) -> Result<Response<deploy_livebook::ResponseData>, APIError> {
        DeployLivebook::mutate(self, application, namespace, version, name, port).await
    }

    pub async fn livebook_resource(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<livebook_resource_query::ResponseData>, APIError> {
        LivebookResourceQuery::fetch(self, application, namespace, version).await
    }

    #[wukong_telemetry(api_event = "destroy_livebook")]
    pub async fn destroy_livebook(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<destroy_livebook::ResponseData>, APIError> {
        DestroyLivebook::mutate(self, application, namespace, version).await
    }
}

// Check if the error is a timeout error or an authentication error.
// For Timeout errors, we get the domain and return it as part of the Timeout error.
fn check_retry_and_auth_error(error: &graphql_client::Error) -> Option<APIError> {
    if error.message == "Unauthenticated" {
        Some(APIError::UnAuthenticated)
    } else if error.message.contains("request_timeout") {
        // The Wukong API returns a message like "{{domain}_request_timeout}", so we need to extract the domain
        // from the message. The domain can be one of 'jenkins', 'spinnaker' or 'github'
        let domain = error.message.split('_').next().unwrap();
        return Some(APIError::Timeout {
            domain: domain.to_string(),
        });
    } else {
        return None;
    }
}
