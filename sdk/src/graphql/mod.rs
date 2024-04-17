pub mod application;
pub mod appsignal;
pub mod changelog;
pub mod deployment;
pub mod deployment_github;
pub mod kubernetes;
pub mod pipeline;

use self::{
    application::ApplicationConfigQuery,
    appsignal::AppsignalIncidentState,
    deployment::{cd_pipeline_status_query, CdPipelineStatusQuery},
};
pub use self::{
    application::{
        application_config_query, application_query, application_with_k8s_cluster_query,
        applications_query, ApplicationQuery, ApplicationWithK8sClusterQuery, ApplicationsQuery,
    },
    appsignal::{
        appsignal_apps_query, appsignal_average_error_rate_query, appsignal_average_latency_query,
        appsignal_average_throughput_query, appsignal_deploy_markers_query,
        appsignal_exception_incidents_query, AppsignalAppsQuery, AppsignalAverageErrorRateQuery,
        AppsignalAverageLatencyQuery, AppsignalAverageThroughputQuery, AppsignalDeployMarkersQuery,
        AppsignalExceptionIncidentsQuery, AppsignalTimeFrame,
    },
    changelog::{changelogs_query, ChangelogsQuery},
    deployment::{
        cd_pipeline_for_rollback_query, cd_pipeline_query, cd_pipelines_query, execute_cd_pipeline,
        CdPipelineForRollbackQuery, CdPipelineQuery, CdPipelinesQuery, ExecuteCdPipeline,
    },
    deployment_github::{cd_pipeline_github_query, CdPipelineGithubQuery},
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
    error::{APIError, WKError},
    ApiChannel, WKClient,
};
use graphql_client::{GraphQLQuery, Response};
use log::debug;
use reqwest::header;
use std::fmt::Debug;
use std::{thread, time};

// Check if the error is a timeout error.
// For Timeout errors, we get the domain and return it as part of the Timeout error.
fn get_timeout_error(error_code: &str) -> Option<APIError> {
    let error_code = error_code.to_lowercase();

    if error_code.contains("timeout") {
        // The Wukong API returns a message like
        // "{{domain}_request_timeout}" in stable channel or
        // "{{domain}_timeout}" in canary channel,
        // so we need to extract the domain from the message.
        // The domain can be one of 'jenkins', 'spinnaker' or 'github'
        let domain = error_code.split('_').next().unwrap();
        Some(APIError::Timeout {
            domain: domain.to_string(),
        })
    } else {
        None
    }
}

pub struct GQLClientBuilder<'a> {
    token: &'a str,
    channel: &'a ApiChannel,
}

impl<'a> Default for GQLClientBuilder<'a> {
    fn default() -> Self {
        Self {
            token: Default::default(),
            channel: &ApiChannel::Stable,
        }
    }
}

impl<'a> GQLClientBuilder<'a> {
    pub fn with_token(mut self, token: &'a str) -> Self {
        self.token = token;
        self
    }
    pub fn with_channel(mut self, channel: &'a ApiChannel) -> Self {
        self.channel = channel;
        self
    }

    pub fn build(self) -> Result<GQLClient, APIError> {
        let mut headers = header::HeaderMap::new();

        let auth_value = format!("Bearer {}", self.token);
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&auth_value).unwrap(),
        );

        // if the channel is canary, we have to include the MV-Canary-Stage header
        // so it will call the canary api
        if let ApiChannel::Canary = self.channel {
            headers.insert(
                "MV-Canary-Stage",
                header::HeaderValue::from_str("always").unwrap(),
            );
        }

        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(GQLClient {
            inner: reqwest_client,
            error_handler: setup_error_handler(self.channel),
        })
    }
}

pub struct GQLClient {
    inner: reqwest::Client,
    error_handler: Box<dyn ErrorHandler>,
}

impl GQLClient {
    async fn post_graphql<Q, U>(
        &self,
        url: U,
        variables: Q::Variables,
    ) -> Result<Q::ResponseData, APIError>
    where
        Q: GraphQLQuery,
        U: reqwest::IntoUrl + Clone + Debug,
        Q::ResponseData: Debug,
    {
        let mut retry_count = 0;
        let body = Q::build_query(variables);
        debug!("url: {:?}", &url);
        debug!("query: \n{}", body.query);

        debug!("reqwest client: {:#?}", self.inner);
        let request = self.inner.post(url.clone()).json(&body);
        debug!("request: {:#?}", request);

        let response: Response<Q::ResponseData> =
            request
                .send()
                .await?
                .json()
                .await
                .map_err(|err: reqwest::Error| {
                    debug!("response: {:#?}", err);
                    APIError::MissingResponseData
                })?;

        debug!("response: {:#?}", response);

        // We use <= 3 so it does one extra loop where the last response is checked
        // in order to return an APIError::Timeout if it was a timeout error in the
        // case of it failing all 3 retries.
        while response.errors.is_some() && retry_count <= 3 {
            if let Some(errors) = response.errors.clone() {
                let first_error = errors[0].clone();

                let first_error_code = self.error_handler.extract_error_code(&first_error);
                match get_timeout_error(first_error_code) {
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

                        let request = self.inner.post(url.clone()).json(&body);
                        debug!("request: {:#?}", request);

                        let response: Response<Q::ResponseData> =
                            request.send().await?.json().await?;
                        debug!("response: {:#?}", response);
                    }
                    _ => {
                        return Err(self.error_handler.handle_error(&first_error));
                    }
                }
            }
        }

        if let Some(data) = response.data {
            Ok(data)
        } else {
            Err(APIError::MissingResponseData)
        }
    }
}

/// Functions from WuKong API Proxy GraphQL endpoints.
impl WKClient {
    /// Fetch supported applications from Wukong API Proxy.
    pub async fn fetch_applications(&self) -> Result<applications_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<ApplicationsQuery, _>(&self.api_url, applications_query::Variables)
            .await
            .map_err(|err| err.into())
    }

    /// Fetch the application info from Wukong API Proxy.
    pub async fn fetch_application(
        &self,
        name: &str,
    ) -> Result<application_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<ApplicationQuery, _>(
                &self.api_url,
                application_query::Variables {
                    name: name.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Fetch the pipelines for an application from Wukong API Proxy.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::UnableToGetPipelines)`](APIError::UnableToGetPipelines) if there is no pipelines under the `application`.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    ///
    /// The application name can be obtained from the [`fetch_applications`](WKClient::fetch_applications) method.
    pub async fn fetch_pipelines(
        &self,
        application: &str,
    ) -> Result<pipelines_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

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

    /// Fetch the pipeline info from Wukong API Proxy.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::UnableToGetPipeline)`](APIError::UnableToGetPipeline) if the pipeline does not exist.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    pub async fn fetch_pipeline(
        &self,
        name: &str,
    ) -> Result<pipeline_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

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

    /// Fetch the multi-branch pipeline info from Wukong API Proxy.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::UnableToGetPipeline)`](APIError::UnableToGetPipeline) if the pipeline does not exist.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    pub async fn fetch_multi_branch_pipeline(
        &self,
        name: &str,
    ) -> Result<multi_branch_pipeline_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

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

    /// Fetch CI status from Wukong API Proxy.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::CIStatusApplicationNotFound)`](APIError::CIStatusApplicationNotFound) if the `application` does not exist.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    pub async fn fetch_ci_status(
        &self,
        repo_url: &str,
        branch: &str,
    ) -> Result<ci_status_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

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

    /// Fetch CD pipelines from Wukong API Proxy.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::ApplicationNotFound)`](APIError::ApplicationNotFound) if the `application` does not exist.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    pub async fn fetch_cd_pipelines(
        &self,
        application: &str,
    ) -> Result<cd_pipelines_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<CdPipelinesQuery, _>(
                &self.api_url,
                cd_pipelines_query::Variables {
                    application: application.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Fetch CD pipeline from Wukong API Proxy.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::ApplicationNotFound)`](APIError::ApplicationNotFound) if the `application` does not exist.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    pub async fn fetch_cd_pipeline(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<CdPipelineQuery, _>(
                &self.api_url,
                cd_pipeline_query::Variables {
                    application: application.to_string(),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }
    /// Fetch CD pipeline from Wukong API Proxy.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::ApplicationNotFound)`](APIError::ApplicationNotFound) if the `application` does not exist.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    pub async fn fetch_cd_pipeline_status(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_status_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<CdPipelineStatusQuery, _>(
                &self.api_url,
                cd_pipeline_status_query::Variables {
                    application: application.to_string(),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Fetch CD pipeline (Github) from Wukong API Proxy.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::ApplicationNotFound)`](APIError::ApplicationNotFound) if the `application` does not exist.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    pub async fn fetch_cd_pipeline_github(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_github_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<CdPipelineGithubQuery, _>(
                &self.api_url,
                cd_pipeline_github_query::Variables {
                    application: application.to_string(),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Fetch changelogs from Wukong API Proxy.
    /// The changelogs is generated by comparing the `build_artifact_name` version with the current deployed version.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::ApplicationNotFound)`](APIError::ApplicationNotFound) if the `application` does not exist.
    /// - [`WKError::APIError(APIError::UnableToDetermineChangelog)`](APIError::UnableToDetermineChangelog) if the changelog generation error.
    /// - [`WKError::APIError(APIError::ChangelogComparingSameBuild)`](APIError::ChangelogComparingSameBuild) if the `build_artifact_name` is same as the current deployed build artifact.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    pub async fn fetch_changelogs(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
    ) -> Result<changelogs_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<ChangelogsQuery, _>(
                &self.api_url,
                changelogs_query::Variables {
                    application: application.to_string(),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                    build_artifact_name: build_artifact_name.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Deploy CD pipeline build to Kubernetes cluster.
    ///
    /// If the `send_to_slack` is `true`, it will send a notification to the Slack channel once the deployment is completed.
    /// If the `changelogs` is not `None`, it will be shown in the Slack notification.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::ApplicationNotFound)`](APIError::ApplicationNotFound) if the `application` does not exist.
    /// - [`WKError::APIError(APIError::DuplicatedDeployment)`](APIError::DuplicatedDeployment) if trying to deploy the same `build_artifact_name` as the current deployed build artifact.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    pub async fn deploy_cd_pipeline_build(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
        changelogs: Option<String>,
        send_to_slack: bool,
    ) -> Result<execute_cd_pipeline::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<ExecuteCdPipeline, _>(
                &self.api_url,
                execute_cd_pipeline::Variables {
                    application: application.to_string(),
                    build_number: 0,
                    build_artifact_name: Some(build_artifact_name.to_string()),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                    changelogs,
                    send_to_slack,
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Fetch previous CD pipeline build from Wukong API Proxy.
    /// This is useful to check the previous build artifact name and probably rollback to the previous build.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::ApplicationNotFound)`](APIError::ApplicationNotFound) if the `application` does not exist.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    pub async fn fetch_previous_cd_pipeline_build(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_for_rollback_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<CdPipelineForRollbackQuery, _>(
                &self.api_url,
                cd_pipeline_for_rollback_query::Variables {
                    application: application.to_string(),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Check whether the current user is authorized to the `application` (with the `namespace` and `version`) or not from Wukong API Proxy.
    ///
    /// It will return:
    /// - [`WKError::APIError(APIError::ApplicationNotFound)`](APIError::ApplicationNotFound) if the `application` does not exist.
    /// - [`WKError::APIError(APIError::NamespaceNotFound)`](APIError::NamespaceNotFound) if the `namespace` does not exist.
    /// - [`WKError::APIError(APIError::VersionNotFound)`](APIError::VersionNotFound) if the `version` does not exist.
    /// - [`WKError::APIError(APIError::ResponseError)`](APIError::ResponseError)  for the rest.
    pub async fn fetch_is_authorized(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<is_authorized_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<IsAuthorizedQuery, _>(
                &self.api_url,
                is_authorized_query::Variables {
                    application: application.to_string(),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Fetch Kubernetes pods for the `application` (with the `namespace` and `version`) from Wukong API Proxy.
    pub async fn fetch_kubernetes_pods(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<kubernetes_pods_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<KubernetesPodsQuery, _>(
                &self.api_url,
                kubernetes_pods_query::Variables {
                    application: application.to_string(),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Check the status of the livebook instance for the `application` (with the `namespace` and `version`) from Wukong API Proxy.
    pub async fn check_livebook_resource(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<livebook_resource_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<LivebookResourceQuery, _>(
                &self.api_url,
                livebook_resource_query::Variables {
                    application: application.to_string(),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Deploy the livebook instance for the `application` (with the `namespace` and `version`) from Wukong API Proxy.
    /// The `name` is the instance name the livebook connect to. The `port` is the livebook port.
    pub async fn deploy_livebook(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
        name: &str,
        port: i64,
    ) -> Result<deploy_livebook::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<DeployLivebook, _>(
                &self.api_url,
                deploy_livebook::Variables {
                    application: application.to_string(),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                    name: name.to_string(),
                    port,
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Destroy the livebook instance for the `application` (with the `namespace` and `version`) from Wukong API Proxy.
    pub async fn destroy_livebook(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<destroy_livebook::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<DestroyLivebook, _>(
                &self.api_url,
                destroy_livebook::Variables {
                    application: application.to_string(),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Fetch the application with k8s cluster info from Wukong API Proxy.
    pub async fn fetch_application_with_k8s_cluster(
        &self,
        name: &str,
        namespace: &str,
        version: &str,
    ) -> Result<application_with_k8s_cluster_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<ApplicationWithK8sClusterQuery, _>(
                &self.api_url,
                application_with_k8s_cluster_query::Variables {
                    name: name.to_string(),
                    namespace: namespace.to_string(),
                    version: version.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_appsignal_average_error_rate(
        &self,
        app_id: &str,
        namespace: &str,
        start: &str,
        until: &str,
        timeframe: AppsignalTimeFrame,
    ) -> Result<appsignal_average_error_rate_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<AppsignalAverageErrorRateQuery, _>(
                &self.api_url,
                appsignal_average_error_rate_query::Variables {
                    app_id: app_id.to_string(),
                    namespace: namespace.to_string(),
                    start: start.to_string(),
                    until: until.to_string(),
                    timeframe: timeframe.into(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_appsignal_average_latency(
        &self,
        app_id: &str,
        namespace: &str,
        start: &str,
        until: &str,
        timeframe: AppsignalTimeFrame,
    ) -> Result<appsignal_average_latency_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<AppsignalAverageLatencyQuery, _>(
                &self.api_url,
                appsignal_average_latency_query::Variables {
                    app_id: app_id.to_string(),
                    namespace: namespace.to_string(),
                    start: start.to_string(),
                    until: until.to_string(),
                    timeframe: timeframe.into(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_appsignal_average_throughput(
        &self,
        app_id: &str,
        namespace: &str,
        start: &str,
        until: &str,
        timeframe: AppsignalTimeFrame,
    ) -> Result<appsignal_average_throughput_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<AppsignalAverageThroughputQuery, _>(
                &self.api_url,
                appsignal_average_throughput_query::Variables {
                    app_id: app_id.to_string(),
                    namespace: namespace.to_string(),
                    start: start.to_string(),
                    until: until.to_string(),
                    timeframe: timeframe.into(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_appsignal_apps(
        &self,
    ) -> Result<appsignal_apps_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<AppsignalAppsQuery, _>(
                &self.api_url,
                appsignal_apps_query::Variables {},
            )
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_application_config(
        &self,
        name: &str,
    ) -> Result<application_config_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<ApplicationConfigQuery, _>(
                &self.api_url,
                application_config_query::Variables {
                    name: name.to_string(),
                },
            )
            .await
            .map_err(|err| err.into())
    }

    /// Fetch the deploy markers from Appsignal
    /// the default value for `limit` is 1
    pub async fn fetch_appsignal_deploy_markers(
        &self,
        app_id: &str,
        limit: Option<i64>,
    ) -> Result<appsignal_deploy_markers_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<AppsignalDeployMarkersQuery, _>(
                &self.api_url,
                appsignal_deploy_markers_query::Variables {
                    app_id: app_id.to_string(),
                    limit,
                },
            )
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_appsignal_exception_incidents(
        &self,
        app_id: &str,
        namespaces: Vec<String>,
        limit: Option<i64>,
        marker: Option<String>,
        state: Option<AppsignalIncidentState>,
    ) -> Result<appsignal_exception_incidents_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token, &self.channel)?;

        gql_client
            .post_graphql::<AppsignalExceptionIncidentsQuery, _>(
                &self.api_url,
                appsignal_exception_incidents_query::Variables {
                    app_id: app_id.to_string(),
                    namespaces,
                    limit,
                    marker,
                    state: state.map(|s| s.into()),
                },
            )
            .await
            .map_err(|err| err.into())
    }
}

fn setup_gql_client(access_token: &str, channel: &ApiChannel) -> Result<GQLClient, WKError> {
    GQLClientBuilder::default()
        .with_token(access_token)
        .with_channel(channel)
        .build()
        .map_err(|err| err.into())
}

pub trait ErrorHandler: Send + Sync {
    fn handle_error(&self, error: &graphql_client::Error) -> APIError;
    fn extract_error_code<'a>(&'a self, error: &'a graphql_client::Error) -> &'a str;
}

pub struct DefaultErrorHandler;
pub struct CanaryErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle_error(&self, error: &graphql_client::Error) -> APIError {
        let original_error_code = self.extract_error_code(error);
        let lowercase_error_code = original_error_code.to_lowercase();
        debug!("Error code: {original_error_code}");

        match lowercase_error_code.as_str() {
            "unauthenticated" => APIError::UnAuthenticated,
            "unable_to_get_pipelines" => APIError::UnableToGetPipelines,
            "unable_to_get_pipeline" => APIError::UnableToGetPipeline,
            "application_not_found" => APIError::ApplicationNotFound,
            "application_config_not_defined" => APIError::ApplicationNotFound,
            "unable_to_determine_changelog" => APIError::UnableToDetermineChangelog,
            "comparing_same_build" => APIError::ChangelogComparingSameBuild,
            "deploy_for_this_build_is_currently_running" => APIError::DuplicatedDeployment,
            "k8s_cluster_namespace_config_not_defined" => APIError::NamespaceNotFound,
            "k8s_cluster_version_config_not_defined" => APIError::VersionNotFound,
            "no_builds_associated_with_this_branch" => APIError::BuildNotFound,
            "unauthorized" => APIError::UnAuthorized,
            _ => APIError::ResponseError {
                code: original_error_code.to_string(),
                message: format!("{error}"),
            },
        }
    }

    fn extract_error_code<'a>(&'a self, error: &'a graphql_client::Error) -> &'a str {
        &error.message
    }
}
impl ErrorHandler for CanaryErrorHandler {
    fn handle_error(&self, error: &graphql_client::Error) -> APIError {
        let original_error_code = self.extract_error_code(error);
        let lowercase_error_code = original_error_code.to_lowercase();
        debug!("Error code: {original_error_code}");

        match lowercase_error_code.as_str() {
            "application_not_found" => APIError::ApplicationNotFound,
            "application_namespace_not_found" => APIError::NamespaceNotFound,
            "application_version_not_found" => APIError::VersionNotFound,

            // authentication
            "unauthenticated" | "invalid_token" => APIError::UnAuthenticated,
            "unauthorized" => APIError::UnAuthorized,

            // pipeline
            "pipeline_not_configured" | "pipeline_not_found" => APIError::UnableToGetPipeline,
            "pipeline_deployment_in_progress" => APIError::DuplicatedDeployment,
            // "pipeline_changelogs_not_provided" => {}
            "jenkins_build_not_found" => APIError::BuildNotFound,
            "jenkins_pipeline_not_found" => APIError::UnableToGetPipeline,

            "changelog_unable_to_determine" => APIError::UnableToDetermineChangelog,
            "changelog_same_commit" => APIError::ChangelogComparingSameBuild,

            // "application_k8s_cluster_not_found" => {}
            // "application_spinnaker_pipeline_not_found" => {}
            // "application_config_error" => {}
            // "k8s_destroy_livebook_failed" => {}
            // "k8s_cluster_context_missing" => {}
            // "k8s_kubeconfig_missing" => {}
            // "k8s_service_not_found_or_deleted" => {}
            // "k8s_ingress_not_found_or_deleted" => {}
            // "k8s_issuer_not_found_or_deleted" => {}
            // "k8s_pod_not_found_or_deleted" => {}
            // "k8s_operation_timed_out" => {}
            // "k8s_ingress_ip_not_found" => {}
            // "k8s_cluster_ip_not_found" => {}
            // "k8s_context_not_found" => {}
            // "k8s_kubeconfig_not_found" => {}
            // "spinnaker_x509_failure" => {}
            // "spinnaker_invalid_domain" => {}
            // "spinnaker_error" => {}
            // "jenkins_invalid_domain" => {}
            // "jenkins_commit_id_not_found" => {}
            // "github_repo_name_not_found" => {}
            // "github_error" => {}
            // "github_invalid_domain" => {}
            // "github_pr_not_found" => {}
            // "github_ref_not_found" => {}
            // "github_commit_history_not_found" => {}
            "github_workflow_not_found" => APIError::GithubWorkflowNotFound,
            "application_config_not_found" => APIError::ApplicationConfigNotFound,
            // "slack_webhook_not_configured" => {}
            _ => APIError::ResponseError {
                code: original_error_code.to_string(),
                message: format!("{error}"),
            },
        }
    }

    fn extract_error_code<'a>(&'a self, error: &'a graphql_client::Error) -> &'a str {
        if let Some(ref error_extensions) = error.extensions {
            if let Some(error_code) = error_extensions.get("code") {
                return error_code.as_str().unwrap_or_default();
            }
        }

        ""
    }
}

fn setup_error_handler(_channel: &ApiChannel) -> Box<dyn ErrorHandler> {
    // TODO: Cleanup outdated error handler:
    Box::new(CanaryErrorHandler)
}

#[cfg(test)]
mod test {
    use crate::{graphql::GQLClientBuilder, ApiChannel};
    use graphql_client::Error;
    use serde_json::json;

    #[test]
    fn test_stable_error_handler_extract_error_code() {
        let gql_client = GQLClientBuilder::default()
            .with_token("test_access_token")
            .with_channel(&ApiChannel::Stable)
            .build()
            .unwrap();

        let err = json!({
          "locations": [
            {
              "column": 3,
              "line": 2
            }
          ],
          "message": "Application not found",
          "path": [
            "ciStatus"
          ],
          "extensions": {
            "code": "application_not_found"
          }
        });

        let deserialized_error: Error = serde_json::from_value(err).unwrap();

        assert_eq!(
            gql_client
                .error_handler
                .extract_error_code(&deserialized_error),
            "application_not_found"
        );
    }

    #[test]
    fn test_canary_error_handler_extract_error_code() {
        let gql_client = GQLClientBuilder::default()
            .with_token("test_access_token")
            .with_channel(&ApiChannel::Canary)
            .build()
            .unwrap();

        let err = json!({
          "locations": [
            {
              "column": 3,
              "line": 2
            }
          ],
          "message": "Application not found",
          "path": [
            "ciStatus"
          ],
          "extensions": {
            "code": "application_not_found"
          }
        });

        let deserialized_error: Error = serde_json::from_value(err).unwrap();

        assert_eq!(
            gql_client
                .error_handler
                .extract_error_code(&deserialized_error),
            "application_not_found"
        );
    }
}
