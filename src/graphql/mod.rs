pub mod application;
pub mod changelog;
pub mod deployment;
pub mod pipeline;

use self::{
    application::{applications_query, ApplicationsQuery},
    changelog::{changelogs_query, ChangelogsQuery},
    deployment::{
        cd_pipeline_query, cd_pipelines_query, execute_cd_pipeline, CdPipelineQuery,
        CdPipelinesQuery, ExecuteCdPipeline,
    },
    pipeline::{
        ci_status_query, multi_branch_pipeline_query, pipeline_query, pipelines_query,
        CiStatusQuery, MultiBranchPipelineQuery, PipelineQuery, PipelinesQuery,
    },
};
use crate::{error::APIError, API_URL};
use graphql_client::{reqwest::post_graphql, GraphQLQuery, Response};
use reqwest::header;

pub struct QueryClientBuilder {
    access_token: Option<String>,
    api_url: String,
}

impl QueryClientBuilder {
    pub fn new() -> Self {
        Self {
            access_token: None,
            api_url: API_URL.to_string(),
        }
    }

    pub fn with_access_token(mut self, access_token: String) -> Self {
        self.access_token = Some(access_token);
        self
    }

    #[allow(dead_code)]
    pub fn with_api_url(mut self, api_url: String) -> Self {
        self.api_url = api_url;
        self
    }

    pub fn build(self) -> Result<QueryClient, APIError> {
        let auth_value = format!(
            "Bearer {}",
            self.access_token.unwrap_or_else(|| "".to_string())
        );

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
        })
    }
}

pub struct QueryClient {
    reqwest_client: reqwest::Client,
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
        let response = post_graphql::<Q, _>(self.inner(), &self.api_url, variables).await?;

        if let Some(errors) = response.errors.clone() {
            let first_error = errors[0].clone();

            match check_auth_error(&first_error) {
                Some(err) => return Err(err),
                None => return handler(response, first_error),
            }
        }

        Ok(response)
    }

    pub async fn fetch_pipeline_list(
        &self,
        application: &str,
    ) -> Result<Response<pipelines_query::ResponseData>, APIError> {
        PipelinesQuery::fetch(self, application).await
    }

    pub async fn fetch_pipeline(
        &self,
        name: &str,
    ) -> Result<Response<pipeline_query::ResponseData>, APIError> {
        PipelineQuery::fetch(self, name).await
    }

    pub async fn fetch_multi_branch_pipeline(
        &self,
        name: &str,
    ) -> Result<Response<multi_branch_pipeline_query::ResponseData>, APIError> {
        MultiBranchPipelineQuery::fetch(self, name).await
    }

    pub async fn fetch_ci_status(
        &self,
        repo_url: &str,
        branch: &str,
    ) -> Result<Response<ci_status_query::ResponseData>, APIError> {
        CiStatusQuery::fetch(self, repo_url, branch).await
    }

    pub async fn fetch_application_list(
        &self,
    ) -> Result<Response<applications_query::ResponseData>, APIError> {
        ApplicationsQuery::fetch(self).await
    }

    pub async fn fetch_cd_pipeline_list(
        &self,
        application: &str,
    ) -> Result<Response<cd_pipelines_query::ResponseData>, APIError> {
        CdPipelinesQuery::fetch(self, application).await
    }

    pub async fn fetch_cd_pipeline(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<cd_pipeline_query::ResponseData>, APIError> {
        CdPipelineQuery::fetch(self, application, namespace, version).await
    }

    pub async fn execute_cd_pipeline(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
        build_number: Option<i64>,
    ) -> Result<Response<execute_cd_pipeline::ResponseData>, APIError> {
        ExecuteCdPipeline::mutate(self, application, namespace, version, build_number).await
    }

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
