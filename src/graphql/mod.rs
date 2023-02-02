pub mod application;
pub mod changelog;
pub mod deployment;
pub mod pipeline;

use self::{
    application::{application_query, applications_query, ApplicationQuery, ApplicationsQuery},
    changelog::{changelogs_query, ChangelogsQuery},
    deployment::{
        cd_pipeline_query, cd_pipelines_query, execute_cd_pipeline, CdPipelineQuery,
        CdPipelinesQuery, ExecuteCdPipeline, CdPipelineForRollbackQuery, cd_pipeline_for_rollback_query
    },
    pipeline::{
        ci_status_query, multi_branch_pipeline_query, pipeline_query, pipelines_query,
        CiStatusQuery, MultiBranchPipelineQuery, PipelineQuery, PipelinesQuery,
    },
};
use crate::{
    app::APP_STATE,
    error::APIError,
    telemetry::{self, TelemetryData, TelemetryEvent},
};
use graphql_client::{GraphQLQuery, Response};
use log::debug;
use reqwest::header;
use wukong_telemetry_macro::wukong_telemetry;

pub struct QueryClientBuilder {
    access_token: Option<String>,
    sub: Option<String>,
    api_url: String,
}

impl QueryClientBuilder {
    pub fn new() -> Self {
        let api_url = match APP_STATE.get() {
            Some(state) => state.api_url.clone(),
            None => "".to_string(),
        };

        Self {
            access_token: None,
            api_url,
            sub: None,
        }
    }

    pub fn with_access_token(mut self, access_token: String) -> Self {
        self.access_token = Some(access_token);
        self
    }

    // for telemetry usage
    pub fn with_sub(mut self, sub: Option<String>) -> Self {
        self.sub = sub;
        self
    }

    #[allow(dead_code)]
    pub fn with_api_url(mut self, api_url: String) -> Self {
        self.api_url = api_url;
        self
    }

    pub fn build(self) -> Result<QueryClient, APIError> {
        let auth_value = format!("Bearer {}", self.access_token.unwrap_or_default());

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&auth_value).unwrap(),
        );

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
    // for telemetry usage
    sub: Option<String>,
    pub api_url: String,
}

impl QueryClient {
    pub fn inner(&self) -> &reqwest::Client {
        &self.reqwest_client
    }

    pub async fn call_api<Q: GraphQLQuery>(
        &self,
        variables: Q::Variables,
        handler: impl Fn(
            Response<Q::ResponseData>,
            graphql_client::Error,
        ) -> Result<Response<Q::ResponseData>, APIError>,
    ) -> Result<Response<Q::ResponseData>, APIError> {
        let body = Q::build_query(variables);
        let request = self.inner().post(&self.api_url).json(&body);

        debug!("request: {:?}", request);
        debug!("graphQL query: \n{}", body.query);

        let response: Response<Q::ResponseData> = request.send().await?.json().await?;

        if let Some(errors) = response.errors.clone() {
            let first_error = errors[0].clone();

            match check_auth_error(&first_error) {
                Some(err) => return Err(err),
                None => return handler(response, first_error),
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
        build_number: i64,
        build_artifact_name: Option<String>,
        changelogs: Option<String>,
        send_to_slack: bool,
    ) -> Result<Response<execute_cd_pipeline::ResponseData>, APIError> {
        ExecuteCdPipeline::mutate(
            self,
            application,
            namespace,
            version,
            build_number,
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
        build_number: i64,
    ) -> Result<Response<changelogs_query::ResponseData>, APIError> {
        ChangelogsQuery::fetch(self, application, namespace, version, build_number).await
    }
}

pub fn check_auth_error(error: &graphql_client::Error) -> Option<APIError> {
    if error.message == "Unauthenticated" {
        return Some(APIError::UnAuthenticated);
    }

    None
}
