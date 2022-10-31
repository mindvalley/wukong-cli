use super::QueryClient;
use crate::error::APIError;
use graphql_client::{GraphQLQuery, Response};

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

        let response = client
            .call_api::<ApplicationsQuery>(variables, |_, error| {
                Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{}", error),
                })
            })
            .await?;

        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::graphql::QueryClientBuilder;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_fetch_application_list_success_should_return_application_list() {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::new()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

        let api_resp = r#"
{
  "data": {
    "applications": [
      {
        "name": "application-1"
      },
      {
        "name": "application-2"
      },
      {
        "name": "application-3"
      }
    ]
  }
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = ApplicationsQuery::fetch(&query_client).await;

        mock.assert();
        assert!(response.is_ok());

        let applications = response.unwrap().data.unwrap().applications;
        assert_eq!(applications.len(), 3);
    }
}
