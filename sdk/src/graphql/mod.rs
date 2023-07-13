pub mod application;
pub mod changelog;
pub mod deployment;
pub mod kubernetes;
pub mod pipeline;

pub use self::{
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
    auth::Auth,
    config::{AuthConfig, Config},
    error::{APIError, WKError},
    telemetry::{self, TelemetryData, TelemetryEvent},
    WKClient,
};
use aion::*;
use chrono::{DateTime, Local};
use graphql_client::{GraphQLQuery, QueryBody, Response};
use log::debug;
use openidconnect::RefreshToken;
use reqwest::header;
use std::fmt::Debug;
use std::{thread, time};
use wukong_telemetry_macro::wukong_telemetry;

// TODO: this will be removed on v2.0
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct QueryClientBuilder {
    api_url: String,
    // authentication
    access_token: Option<String>,
    expiry_time: Option<String>,
    // for telemetry usage
    sub: Option<String>,
}

#[allow(dead_code)]
impl QueryClientBuilder {
    pub fn with_access_token(mut self, access_token: String) -> Self {
        self.access_token = Some(access_token);
        self
    }

    // for telemetry usage
    pub fn with_sub(mut self, sub: String) -> Self {
        self.sub = Some(sub);
        self
    }

    pub fn with_api_url(mut self, api_url: String) -> Self {
        self.api_url = api_url;
        self
    }

    pub fn with_expiry_time(mut self, expiry_time: String) -> Self {
        self.expiry_time = Some(expiry_time);
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
            expiry_time: self.expiry_time,
            access_token: self.access_token,
        })
    }
}

pub struct QueryClient {
    reqwest_client: reqwest::Client,
    api_url: String,
    // authentication
    access_token: Option<String>,
    expiry_time: Option<String>,
    // for telemetry usage
    sub: Option<String>,
}

impl QueryClient {
    pub fn from_default_config() -> Result<Self, CliError> {
        let config = Config::load_from_default_path()?;
        Self::from_config(&config)
    }

    pub fn from_config(config: &Config) -> Result<Self, CliError> {
        let auth_config = config.auth.as_ref().ok_or(CliError::UnAuthenticated)?;
        let token = auth_config.id_token.clone();

        let mut headers = header::HeaderMap::new();

        let bearer_token = format!("Bearer {}", token);
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&bearer_token).unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(<reqwest::Error as Into<APIError>>::into)?;

        Ok(QueryClient {
            reqwest_client: client,
            api_url: config.core.wukong_api_url.clone(),
            access_token: Some(token),
            expiry_time: Some(auth_config.expiry_time.clone()),
            sub: Some(auth_config.subject.clone()),
        })
    }

    pub fn inner(&self) -> &reqwest::Client {
        &self.reqwest_client
    }

    pub async fn call_api<Q>(
        &mut self,
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

        // Before calling API, do a token expiry check first

        if let (Some(_token), Some(expiry_time)) = (&self.access_token, &self.expiry_time) {
            let current_time: DateTime<Local> = Local::now();
            let expiry = DateTime::parse_from_rfc3339(expiry_time)
                .unwrap()
                .with_timezone(&Local);
            let remaining_duration = expiry.signed_duration_since(current_time);

            if remaining_duration < 5.minutes() {
                debug!("Access token expired. Refreshing tokens...");

                let mut config = Config::load_from_default_path().map_err(|err| {
                    debug!("Failed to refresh tokens when getting config: {:?}", err);
                    err
                })?;
                let auth_config = config.auth.as_ref().unwrap();

                let new_tokens = Auth::new(&config.core.okta_client_id)
                    .refresh_tokens(&RefreshToken::new(auth_config.refresh_token.clone()))
                    .await
                    .map_err(|err| {
                        debug!("Failed to refresh tokens: {:?}", err);
                        err
                    })?;

                config.auth = Some(AuthConfig {
                    account: auth_config.account.clone(),
                    subject: auth_config.subject.clone(),
                    id_token: new_tokens.id_token.clone(),
                    access_token: new_tokens.access_token.clone(),
                    expiry_time: new_tokens.expiry_time.clone(),
                    refresh_token: new_tokens.refresh_token,
                });

                config.save_to_default_path().map_err(|err| {
                    debug!("Failed to refresh tokens when saving config: {:?}", err);
                    err
                })?;

                // make sure to update the token and expiry time to the updated values
                let mut headers = header::HeaderMap::new();

                let bearer_token = format!("Bearer {}", new_tokens.id_token);
                headers.insert(
                    header::AUTHORIZATION,
                    header::HeaderValue::from_str(&bearer_token).unwrap(),
                );

                let client = reqwest::Client::builder()
                    .default_headers(headers)
                    .build()
                    .map_err(<reqwest::Error as Into<APIError>>::into)?;

                self.reqwest_client = client;
                self.access_token = Some(new_tokens.id_token);
                self.expiry_time = Some(new_tokens.expiry_time);
            }
        }

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
        &mut self,
        application: &str,
    ) -> Result<Response<pipelines_query::ResponseData>, APIError> {
        PipelinesQuery::fetch(self, application).await
    }

    #[wukong_telemetry(api_event = "fetch_pipeline")]
    pub async fn fetch_pipeline(
        &mut self,
        name: &str,
    ) -> Result<Response<pipeline_query::ResponseData>, APIError> {
        PipelineQuery::fetch(self, name).await
    }

    #[wukong_telemetry(api_event = "fetch_multi_branch_pipeline")]
    pub async fn fetch_multi_branch_pipeline(
        &mut self,
        name: &str,
    ) -> Result<Response<multi_branch_pipeline_query::ResponseData>, APIError> {
        MultiBranchPipelineQuery::fetch(self, name).await
    }

    #[wukong_telemetry(api_event = "fetch_ci_status")]
    pub async fn fetch_ci_status(
        &mut self,
        repo_url: &str,
        branch: &str,
    ) -> Result<Response<ci_status_query::ResponseData>, APIError> {
        CiStatusQuery::fetch(self, repo_url, branch).await
    }

    #[wukong_telemetry(api_event = "fetch_application_list")]
    pub async fn fetch_application_list(
        &mut self,
    ) -> Result<Response<applications_query::ResponseData>, APIError> {
        ApplicationsQuery::fetch(self).await
    }

    #[wukong_telemetry(api_event = "fetch_application")]
    pub async fn fetch_application(
        &mut self,
        name: &str,
    ) -> Result<Response<application_query::ResponseData>, APIError> {
        ApplicationQuery::fetch(self, name).await
    }

    #[wukong_telemetry(api_event = "fetch_application_with_k8s_cluster")]
    pub async fn fetch_application_with_k8s_cluster(
        &mut self,
        name: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<application_with_k8s_cluster_query::ResponseData>, APIError> {
        ApplicationWithK8sClusterQuery::fetch(self, name, namespace, version).await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline_list")]
    pub async fn fetch_cd_pipeline_list(
        &mut self,
        application: &str,
    ) -> Result<Response<cd_pipelines_query::ResponseData>, APIError> {
        CdPipelinesQuery::fetch(self, application).await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline")]
    pub async fn fetch_cd_pipeline(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<cd_pipeline_query::ResponseData>, APIError> {
        CdPipelineQuery::fetch(self, application, namespace, version).await
    }

    #[wukong_telemetry(api_event = "fetch_cd_pipeline_for_rollback")]
    pub async fn fetch_cd_pipeline_for_rollback(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<cd_pipeline_for_rollback_query::ResponseData>, APIError> {
        CdPipelineForRollbackQuery::fetch(self, application, namespace, version).await
    }

    #[wukong_telemetry(api_event = "execute_cd_pipeline")]
    pub async fn execute_cd_pipeline(
        &mut self,
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
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
    ) -> Result<Response<changelogs_query::ResponseData>, APIError> {
        ChangelogsQuery::fetch(self, application, namespace, version, build_artifact_name).await
    }

    #[wukong_telemetry(api_event = "fetch_kubernetes_pods")]
    pub async fn fetch_kubernetes_pods(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<kubernetes_pods_query::ResponseData>, APIError> {
        KubernetesPodsQuery::fetch(self, application, namespace, version).await
    }

    #[wukong_telemetry(api_event = "fetch_is_authorized")]
    pub async fn fetch_is_authorized(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<is_authorized_query::ResponseData>, APIError> {
        IsAuthorizedQuery::fetch(self, application, namespace, version).await
    }

    #[wukong_telemetry(api_event = "deploy_livebook")]
    pub async fn deploy_livebook(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
        name: &str,
        port: i64,
    ) -> Result<Response<deploy_livebook::ResponseData>, APIError> {
        DeployLivebook::mutate(self, application, namespace, version, name, port).await
    }

    pub async fn livebook_resource(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<livebook_resource_query::ResponseData>, APIError> {
        LivebookResourceQuery::fetch(self, application, namespace, version).await
    }

    #[wukong_telemetry(api_event = "destroy_livebook")]
    pub async fn destroy_livebook(
        &mut self,
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

pub struct GQLClient {
    inner: reqwest::Client,
}

impl GQLClient {
    pub fn with_authorization(token: &str) -> Result<Self, APIError> {
        let mut headers = header::HeaderMap::new();

        let auth_value = format!("Bearer {}", token);
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&auth_value).unwrap(),
        );

        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            inner: reqwest_client,
        })
    }

    pub fn without_authorization() -> Result<Self, APIError> {
        let reqwest_client = reqwest::Client::builder().build()?;

        Ok(Self {
            inner: reqwest_client,
        })
    }

    pub async fn post_graphql<Q, U>(
        &self,
        url: U,
        variables: Q::Variables,
    ) -> Result<Q::ResponseData, APIError>
    where
        Q: GraphQLQuery,
        U: reqwest::IntoUrl,
        Q::ResponseData: Debug,
    {
        let body = Q::build_query(variables);
        let res: Response<Q::ResponseData> = self
            .inner
            .post(url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        debug!("GraphQL response: {:?}", res);

        if let Some(errors) = res.errors {
            if errors[0].message.to_lowercase().contains("unauthenticated") {
                return Err(APIError::UnAuthenticated);
            } else {
                return Err(APIError::ResponseError {
                    code: errors[0].message.clone(),
                    message: "".to_string(),
                });
            }
        }

        if let Some(data) = res.data {
            Ok(data)
        } else {
            Err(APIError::MissingResponseData)
        }
    }
}

impl WKClient {
    pub async fn fetch_applications(&self) -> Result<applications_query::ResponseData, WKError> {
        let gql_client = GQLClient::with_authorization(
            &self
                .access_token
                .as_ref()
                .ok_or(APIError::UnAuthenticated)?,
        )?;

        gql_client
            .post_graphql::<ApplicationsQuery, _>(&self.api_url, applications_query::Variables)
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_pipelines(
        &self,
        application: &str,
    ) -> Result<pipelines_query::ResponseData, WKError> {
        let gql_client = GQLClient::with_authorization(
            &self
                .access_token
                .as_ref()
                .ok_or(APIError::UnAuthenticated)?,
        )?;

        gql_client
            .post_graphql::<PipelinesQuery, _>(
                &self.api_url,
                pipelines_query::Variables {
                    application: Some(application.to_string()),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_pipeline(
        &self,
        name: &str,
    ) -> Result<pipeline_query::ResponseData, WKError> {
        let gql_client = GQLClient::with_authorization(
            &self
                .access_token
                .as_ref()
                .ok_or(APIError::UnAuthenticated)?,
        )?;

        gql_client
            .post_graphql::<PipelineQuery, _>(
                &self.api_url,
                pipeline_query::Variables {
                    name: name.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_multi_branch_pipeline(
        &self,
        name: &str,
    ) -> Result<multi_branch_pipeline_query::ResponseData, WKError> {
        let gql_client = GQLClient::with_authorization(
            &self
                .access_token
                .as_ref()
                .ok_or(APIError::UnAuthenticated)?,
        )?;

        gql_client
            .post_graphql::<MultiBranchPipelineQuery, _>(
                &self.api_url,
                multi_branch_pipeline_query::Variables {
                    name: name.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_ci_status(
        &self,
        repo_url: &str,
        branch: &str,
    ) -> Result<ci_status_query::ResponseData, WKError> {
        let gql_client = GQLClient::with_authorization(
            &self
                .access_token
                .as_ref()
                .ok_or(APIError::UnAuthenticated)?,
        )?;

        gql_client
            .post_graphql::<CiStatusQuery, _>(
                &self.api_url,
                ci_status_query::Variables {
                    repo_url: repo_url.to_string(),
                    branch: branch.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }
}
