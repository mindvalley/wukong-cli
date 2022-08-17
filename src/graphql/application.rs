use super::QueryClient;
use crate::{error::APIError, SETTINGS};
use graphql_client::{reqwest::post_graphql, GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/applications.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ApplicationsQuery;

impl ApplicationsQuery {
    pub(crate) async fn fetch(
        client: &QueryClient,
    ) -> Result<Response<applications_query::ResponseData>, APIError> {
        let variables = applications_query::Variables {};

        let response =
            post_graphql::<ApplicationsQuery, _>(client.inner(), &SETTINGS.api.url, variables)
                .await?;
        if let Some(errors) = response.errors {
            let first_error = errors[0].clone();

            return Err(APIError::ResponseError {
                code: first_error.message,
                message: format!("{}", errors[0].clone()),
            });
        }
        Ok(response)
    }
}
