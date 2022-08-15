use super::{QueryClient, URL};
use crate::error::APIError;
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

        let response = post_graphql::<ApplicationsQuery, _>(client.inner(), URL, variables).await?;
        if let Some(errors) = response.errors {
            let first_error = errors[0].clone();
            // if first_error.message == "unable_to_get_pipelines" {
            //     return Err(APIError::ResponseError {
            //         code: first_error.message,
            //         message: format!("Unable to get pipelines for application `{}`.", application),
            //     });
            // }

            return Err(APIError::ResponseError {
                code: first_error.message,
                message: format!("{}", errors[0].clone()),
            });
        }
        Ok(response)
    }
}
