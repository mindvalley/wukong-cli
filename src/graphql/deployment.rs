use super::{check_auth_error, QueryClient};
use crate::{error::APIError, SETTINGS};
use graphql_client::{reqwest::post_graphql, GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelinesQuery;

impl CdPipelinesQuery {
    pub(crate) async fn fetch(
        client: &QueryClient,
        application: &str,
    ) -> Result<Response<cd_pipelines_query::ResponseData>, APIError> {
        let variables = cd_pipelines_query::Variables {
            application: application.to_string(),
        };

        let response =
            post_graphql::<Self, _>(client.inner(), &SETTINGS.api.url, variables).await?;
        if let Some(errors) = response.errors {
            let first_error = errors[0].clone();
            match check_auth_error(&first_error) {
                Some(err) => return Err(err),
                None => {
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
