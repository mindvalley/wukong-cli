use super::QueryClient;
use crate::error::APIError;
use graphql_client::{GraphQLQuery, Response};

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

        let response = client
            .call_api::<Self>(variables, |_, error| {
                return Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{}", error.clone()),
                });
            })
            .await?;

        Ok(response)
    }
}
