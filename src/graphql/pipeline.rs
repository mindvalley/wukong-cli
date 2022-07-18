use super::auth_headers;
use crate::error::{APIError, CliError};
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
        println!("{:?}", response);
        if let Some(errors) = response.errors {
            println!("errors: {:?}", errors);
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
        application: String,
    ) -> Result<Response<pipeline_query::ResponseData>, APIError> {
        let client = reqwest::Client::builder()
            .default_headers(auth_headers())
            .build()?;

        let variables = pipeline_query::Variables { name: application };

        let response = post_graphql::<PipelineQuery, _>(&client, URL, variables).await?;
        // println!("{:#?}", response);

        if let Some(errors) = response.errors {
            println!("errors: {:?}", errors);
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
