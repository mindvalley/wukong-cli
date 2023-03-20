pub mod application;
pub mod changelog;
pub mod deployment;
pub mod pipeline;

use self::{
    application::{application_query, applications_query, ApplicationQuery, ApplicationsQuery},
    changelog::{changelogs_query, ChangelogsQuery},
    deployment::{
        cd_pipeline_for_rollback_query, cd_pipeline_query, cd_pipelines_query, execute_cd_pipeline,
        CdPipelineForRollbackQuery, CdPipelineQuery, CdPipelinesQuery, ExecuteCdPipeline,
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

        if let Some(token) = self.access_token {
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
        let client = self.inner();

        debug!("url: {:?}", &self.api_url);
        debug!("GraphQL query: \n{}", body.query);

        let response = self.retry_request::<Q>(client, body, handler).await;
        debug!("GraphQL response: {:#?}", response);

        response
    }

    async fn retry_request<Q>(
        &self,
        client: &reqwest::Client,
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
        let mut response: Response<<Q as GraphQLQuery>::ResponseData> = client
            .post(&self.api_url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        while response.errors.is_some() && retry_count <= 3 {
            if let Some(errors) = response.errors.clone() {
                let first_error = errors[0].clone();

                if retry_count == 3 {
                    return handler(response, first_error);
                }

                match check_retry_and_auth_error(&first_error) {
                    Some(APIError::UnAuthenticated) => return Err(APIError::UnAuthenticated),
                    Some(APIError::Timeout) => {
                        retry_count += 1;
                        eprintln!(
                            "... request timeout, retrying the request {}/3",
                            retry_count
                        );
                        thread::sleep(time::Duration::from_secs(5));
                        response = client
                            .post(&self.api_url)
                            .json(&body)
                            .send()
                            .await?
                            .json()
                            .await?;
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
}

// pub fn check_auth_error(error: &graphql_client::Error) -> Option<APIError> {
//     if error.message == "Unauthenticated" {
//         return Some(APIError::UnAuthenticated);
//     }
//
//     None
// }

pub fn check_retry_and_auth_error(error: &graphql_client::Error) -> Option<APIError> {
    if error.message == "Unauthenticated" {
        return Some(APIError::UnAuthenticated);
    } else if error.message.contains("timeout") {
        return Some(APIError::Timeout);
    } else if error.message.contains("domain") {
        return Some(APIError::DomainError);
    } else {
        return None;
    }
}
