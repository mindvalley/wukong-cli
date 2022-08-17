use super::QueryClient;
use crate::{error::APIError, SETTINGS};
use graphql_client::{reqwest::post_graphql, GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct PipelinesQuery;

impl PipelinesQuery {
    pub(crate) async fn fetch(
        client: &QueryClient,
        application: &str,
    ) -> Result<Response<pipelines_query::ResponseData>, APIError> {
        let variables = pipelines_query::Variables {
            application: Some(application.to_string()),
        };

        let response =
            post_graphql::<PipelinesQuery, _>(client.inner(), &SETTINGS.api.url, variables).await?;
        if let Some(errors) = response.errors {
            let first_error = errors[0].clone();
            if first_error.message == "unable_to_get_pipelines" {
                return Err(APIError::ResponseError {
                    code: first_error.message,
                    message: format!("Unable to get pipelines for application `{}`.", application),
                });
            }

            return Err(APIError::ResponseError {
                code: first_error.message,
                message: format!("{}", errors[0].clone()),
            });
        }
        Ok(response)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct PipelineQuery;

impl PipelineQuery {
    pub(crate) async fn fetch(
        client: &QueryClient,
        name: &str,
    ) -> Result<Response<pipeline_query::ResponseData>, APIError> {
        let variables = pipeline_query::Variables {
            name: name.to_string(),
        };

        let response =
            post_graphql::<PipelineQuery, _>(client.inner(), &SETTINGS.api.url, variables).await?;

        if let Some(errors) = response.errors {
            let first_error = errors[0].clone();
            if first_error.message == "unable_to_get_pipeline" {
                return Err(APIError::ResponseError {
                    code: first_error.message,
                    message: format!("Unable to get pipeline `{}`.", name),
                });
            }

            return Err(APIError::ResponseError {
                code: first_error.message,
                message: format!("{}", errors[0].clone()),
            });
        }
        Ok(response)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/multi_branch_pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct MultiBranchPipelineQuery;

impl MultiBranchPipelineQuery {
    pub async fn fetch(
        client: &QueryClient,
        name: &str,
    ) -> Result<Response<multi_branch_pipeline_query::ResponseData>, APIError> {
        let variables = multi_branch_pipeline_query::Variables {
            name: name.to_string(),
        };

        let response = post_graphql::<MultiBranchPipelineQuery, _>(
            client.inner(),
            &SETTINGS.api.url,
            variables,
        )
        .await?;
        if let Some(errors) = response.errors {
            let first_error = errors[0].clone();
            if first_error.message == "unable_to_get_pipeline" {
                return Err(APIError::ResponseError {
                    code: first_error.message,
                    message: format!("Unable to get pipeline `{}`.", name),
                });
            }

            return Err(APIError::ResponseError {
                code: first_error.message,
                message: format!("{}", errors[0].clone()),
            });
        }
        Ok(response)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/ci_status.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CiStatusQuery;

impl CiStatusQuery {
    pub async fn fetch(
        client: &QueryClient,
        repo_url: &str,
        branch: &str,
    ) -> Result<Response<ci_status_query::ResponseData>, APIError> {
        let variables = ci_status_query::Variables {
            repo_url: repo_url.to_string(),
            branch: branch.to_string(),
        };

        let response =
            post_graphql::<CiStatusQuery, _>(client.inner(), &SETTINGS.api.url, variables).await?;
        if let Some(errors) = response.errors.clone() {
            let first_error = errors[0].clone();
            match first_error.message.as_str() {
                "application_not_found" => {
                    return Err(APIError::ResponseError {
                        code: first_error.message,
                        message: format!("Application `{}` not found.", repo_url),
                    });
                }
                "no_builds_associated_with_this_branch" => {
                    return Ok(response);
                }
                _ => {
                    return Err(APIError::ResponseError {
                        code: first_error.message,
                        message: format!("{}", errors[0].clone()),
                    });
                }
            }
        }
        Ok(response)
    }
}
