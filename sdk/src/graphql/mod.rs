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
    error::{APIError, WKError},
    WKClient,
};
use graphql_client::{GraphQLQuery, Response};
use log::debug;
use reqwest::header;
use std::fmt::Debug;
use std::{thread, time};

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
        U: reqwest::IntoUrl + Clone + Debug,
        Q::ResponseData: Debug,
    {
        let mut retry_count = 0;
        let body = Q::build_query(variables);
        debug!("url: {:?}", &url);
        debug!("query: \n{}", body.query);

        let request = self.inner.post(url.clone()).json(&body);
        debug!("request: {:#?}", request);

        let response: Response<Q::ResponseData> = request.send().await?.json().await?;
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

                        let request = self.inner.post(url.clone()).json(&body);
                        debug!("request: {:#?}", request);

                        let response: Response<Q::ResponseData> =
                            request.send().await?.json().await?;
                        debug!("response: {:#?}", response);
                    }
                    _ => {
                        return Err(APIError::ResponseError {
                            code: first_error.message.clone(),
                            message: format!("{first_error}"),
                        });
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

impl WKClient {
    pub async fn fetch_applications(&self) -> Result<applications_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

        gql_client
            .post_graphql::<ApplicationsQuery, _>(&self.api_url, applications_query::Variables)
            .await
            .map_err(|err| err.into())
    }

    pub async fn fetch_application(
        &self,
        name: &str,
    ) -> Result<application_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

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

    pub async fn fetch_pipelines(
        &self,
        application: &str,
    ) -> Result<pipelines_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

        gql_client
            .post_graphql::<PipelinesQuery, _>(
                &self.api_url,
                pipelines_query::Variables {
                    application: Some(application.to_string()),
                },
            )
            .await
            .map_err(|err| {
                match &err {
                    APIError::ResponseError { code, message: _ } => match code.as_str() {
                        "unable_to_get_pipelines" => APIError::UnableToGetPipelines {
                            application: application.to_string(),
                        },
                        _ => err,
                    },
                    _ => err,
                }
                .into()
            })
    }

    pub async fn fetch_pipeline(
        &self,
        name: &str,
    ) -> Result<pipeline_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

        gql_client
            .post_graphql::<PipelineQuery, _>(
                &self.api_url,
                pipeline_query::Variables {
                    name: name.to_string(),
                },
            )
            .await
            .map_err(|err| {
                match &err {
                    APIError::ResponseError { code, message: _ } => match code.as_str() {
                        "unable_to_get_pipeline" => APIError::UnableToGetPipeline {
                            pipeline: name.to_string(),
                        },
                        _ => err,
                    },
                    _ => err,
                }
                .into()
            })
    }

    pub async fn fetch_multi_branch_pipeline(
        &self,
        name: &str,
    ) -> Result<multi_branch_pipeline_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

        gql_client
            .post_graphql::<MultiBranchPipelineQuery, _>(
                &self.api_url,
                multi_branch_pipeline_query::Variables {
                    name: name.to_string(),
                },
            )
            .await
            .map_err(|err| {
                match &err {
                    APIError::ResponseError { code, message: _ } => match code.as_str() {
                        "unable_to_get_pipeline" => APIError::UnableToGetPipeline {
                            pipeline: name.to_string(),
                        },
                        _ => err,
                    },
                    _ => err,
                }
                .into()
            })
    }

    // TODO: Error handling
    pub async fn fetch_ci_status(
        &self,
        repo_url: &str,
        branch: &str,
    ) -> Result<ci_status_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

        let response = gql_client
            .post_graphql::<CiStatusQuery, _>(
                &self.api_url,
                ci_status_query::Variables {
                    repo_url: repo_url.to_string(),
                    branch: branch.to_string(),
                },
            )
            .await;

        if let Err(err) = &response {
            match err {
                APIError::ResponseError { code, message: _ } => {
                    if code == "application_not_found" {
                        return Err(APIError::CIStatusApplicationNotFound.into());
                    }

                    // we don't want this to be an error
                    if code == "no_builds_associated_with_this_branch" {
                        return Ok(ci_status_query::ResponseData { ci_status: None });
                    }
                }
                _ => return response.map_err(|err| err.into()),
            }
        }

        response.map_err(|err| err.into())
    }

    pub async fn fetch_cd_pipelines(
        &self,
        application: &str,
    ) -> Result<cd_pipelines_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

        gql_client
            .post_graphql::<CdPipelinesQuery, _>(
                &self.api_url,
                cd_pipelines_query::Variables {
                    application: application.to_string(),
                },
            )
            .await
            .map_err(|err| {
                match &err {
                    APIError::ResponseError { code, message: _ } => match code.as_str() {
                        "application_not_found" => APIError::ApplicationNotFound {
                            application: application.to_string(),
                        },
                        _ => err,
                    },
                    _ => err,
                }
                .into()
            })
    }

    pub async fn fetch_cd_pipeline(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

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
            .map_err(|err| {
                match &err {
                    APIError::ResponseError { code, message: _ } => match code.as_str() {
                        "application_not_found" => APIError::ApplicationNotFound {
                            application: application.to_string(),
                        },
                        _ => err,
                    },
                    _ => err,
                }
                .into()
            })
    }

    pub async fn fetch_changelogs(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
    ) -> Result<changelogs_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

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
            .map_err(|err| {
                match &err {
                    APIError::ResponseError { code, message: _ } => match code.as_str() {
                        "application_not_found" => APIError::ApplicationNotFound {
                            application: application.to_string(),
                        },
                        "unable_to_determine_changelog" => APIError::UnableToDetermineChangelog {
                            build: build_artifact_name.to_string(),
                        },
                        "comparing_same_build" => APIError::ChangelogComparingSameBuild,
                        _ => err,
                    },
                    _ => err,
                }
                .into()
            })
    }

    pub async fn deploy_cd_pipeline_build(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
        build_artifact_name: &str,
        changelogs: Option<String>,
        send_to_slack: bool,
    ) -> Result<execute_cd_pipeline::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

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
            .map_err(|err| {
                match &err {
                    APIError::ResponseError { code, message: _ } => match code.as_str() {
                        "application_not_found" => APIError::ApplicationNotFound {
                            application: application.to_string(),
                        },
                        "deploy_for_this_build_is_currently_running" => {
                            APIError::DuplicatedDeployment
                        }
                        _ => err,
                    },
                    _ => err,
                }
                .into()
            })
    }

    pub async fn fetch_previous_cd_pipeline_build(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<cd_pipeline_for_rollback_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

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
            .map_err(|err| {
                match &err {
                    APIError::ResponseError { code, message: _ } => match code.as_str() {
                        "application_not_found" => APIError::ApplicationNotFound {
                            application: application.to_string(),
                        },
                        _ => err,
                    },
                    _ => err,
                }
                .into()
            })
    }

    pub async fn fetch_is_authorized(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<is_authorized_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

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

    pub async fn fetch_kubernetes_pods(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<kubernetes_pods_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

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
            .map_err(|err| {
                match &err {
                    APIError::ResponseError { code, message: _ } => match code.as_str() {
                        "Unauthorized" => APIError::ResponseError {
                            code: code.clone(),
                            message: code.to_string(),
                        },
                        _ => err,
                    },
                    _ => err,
                }
                .into()
            })
    }

    pub async fn check_livebook_resource(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<livebook_resource_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

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

    pub async fn deploy_livebook(
        &mut self,
        application: &str,
        namespace: &str,
        version: &str,
        name: &str,
        port: i64,
    ) -> Result<deploy_livebook::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

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
            .map_err(|err| {
                match &err {
                    APIError::ResponseError { code, message: _ } => match code.as_str() {
                        "Unauthorized" => APIError::ResponseError {
                            code: code.clone(),
                            message: code.to_string(),
                        },
                        _ => err,
                    },
                    _ => err,
                }
                .into()
            })
    }

    pub async fn destroy_livebook(
        &self,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<destroy_livebook::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

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
            .map_err(|err| {
                match &err {
                    APIError::ResponseError { code, message: _ } => match code.as_str() {
                        "Unauthorized" => APIError::ResponseError {
                            code: code.clone(),
                            message: code.to_string(),
                        },
                        _ => err,
                    },
                    _ => err,
                }
                .into()
            })
    }

    pub async fn fetch_application_with_k8s_cluster(
        &mut self,
        name: &str,
        namespace: &str,
        version: &str,
    ) -> Result<application_with_k8s_cluster_query::ResponseData, WKError> {
        let gql_client = setup_gql_client(&self.access_token)?;

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
}

fn setup_gql_client(access_token: &str) -> Result<GQLClient, WKError> {
    GQLClient::with_authorization(access_token).map_err(|err| err.into())
}
