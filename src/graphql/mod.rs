pub mod application;
pub mod pipeline;

use self::{
    application::{applications_query, ApplicationsQuery},
    pipeline::{
        ci_status_query, multi_branch_pipeline_query, pipeline_query, pipelines_query,
        CiStatusQuery, MultiBranchPipelineQuery, PipelineQuery, PipelinesQuery,
    },
};
use crate::error::APIError;
use graphql_client::Response;
use reqwest::header;

pub const URL: &'static str = "http://localhost:4000/api";

pub struct QueryClientBuilder {
    access_token: Option<String>,
}

impl QueryClientBuilder {
    pub fn new() -> Self {
        Self { access_token: None }
    }

    pub fn with_access_token(mut self, access_token: String) -> Self {
        self.access_token = Some(access_token);
        self
    }

    pub fn build(self) -> Result<QueryClient, APIError> {
        let auth_value = format!("Bearer {}", self.access_token.unwrap_or("".to_string()));

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
        })
    }
}

pub struct QueryClient {
    reqwest_client: reqwest::Client,
}

impl QueryClient {
    pub fn inner(&self) -> &reqwest::Client {
        &self.reqwest_client
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
}
