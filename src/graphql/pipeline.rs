use super::auth_headers;
use crate::error::APIError;
use graphql_client::{reqwest::post_graphql, GraphQLQuery, Response};

const URL: &'static str = "http://localhost:4000/api";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct PipelinesQuery;

impl PipelinesQuery {
    pub async fn fetch() -> Result<Response<pipelines_query::ResponseData>, APIError> {
        let client = reqwest::Client::builder()
            .default_headers(auth_headers())
            .build()?;

        let variables = pipelines_query::Variables {};

        let response = post_graphql::<PipelinesQuery, _>(&client, URL, variables).await?;
        if let Some(errors) = response.errors {
            let first_error = errors[0].clone();
            if first_error.message == "unable_to_get_pipelines" {
                return Err(APIError::ResponseError {
                    code: first_error.message,
                    message: "Unable to get pipelines.".to_string(),
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
    pub async fn fetch(
        application: &str,
    ) -> Result<Response<pipeline_query::ResponseData>, APIError> {
        let client = reqwest::Client::builder()
            .default_headers(auth_headers())
            .build()?;

        let variables = pipeline_query::Variables {
            name: application.to_string(),
        };

        let response = post_graphql::<PipelineQuery, _>(&client, URL, variables).await?;

        if let Some(errors) = response.errors {
            let first_error = errors[0].clone();
            if first_error.message == "unable_to_get_pipeline" {
                return Err(APIError::ResponseError {
                    code: first_error.message,
                    message: "Unable to get pipeline.".to_string(),
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
        name: &str,
    ) -> Result<Response<multi_branch_pipeline_query::ResponseData>, APIError> {
        let client = reqwest::Client::builder()
            .default_headers(auth_headers())
            .build()?;

        let variables = multi_branch_pipeline_query::Variables {
            name: name.to_string(),
        };

        let response = post_graphql::<MultiBranchPipelineQuery, _>(&client, URL, variables).await?;
        if let Some(errors) = response.errors {
            let first_error = errors[0].clone();
            if first_error.message == "unable_to_get_pipeline" {
                return Err(APIError::ResponseError {
                    code: first_error.message,
                    message: "Unable to get pipeline.".to_string(),
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
        repo_url: &str,
        branch: &str,
    ) -> Result<Response<ci_status_query::ResponseData>, APIError> {
        let client = reqwest::Client::builder()
            .default_headers(auth_headers())
            .build()?;

        let variables = ci_status_query::Variables {
            repo_url: repo_url.to_string(),
            branch: branch.to_string(),
        };

        let response = post_graphql::<CiStatusQuery, _>(&client, URL, variables).await?;
        if let Some(errors) = response.errors.clone() {
            let first_error = errors[0].clone();
            match first_error.message.as_str() {
                "application_not_found" => {
                    return Err(APIError::ResponseError {
                        code: first_error.message,
                        message: "Application not found.".to_string(),
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
