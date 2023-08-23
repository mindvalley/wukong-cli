use super::QueryClient;
use crate::error::APIError;
use graphql_client::{GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/github_pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct GithubPipelinesQuery;

impl GithubPipelinesQuery {
    pub async fn fetch(
        client: &mut QueryClient,
        application: &str,
    ) -> Result<Response<github_pipelines_query::ResponseData>, APIError> {
        let variables = github_pipelines_query::Variables {
            application: Some(application.to_string()),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| {
                if error.message == "unable_to_get_github_pipelines" {
                    return Err(APIError::ResponseError {
                        code: error.message,
                        message: format!(
                            "Unable to get github pipelines for application `{application}`."
                        ),
                    });
                }

                Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{error}"),
                })
            })
            .await?;

        Ok(response)
    }
}
