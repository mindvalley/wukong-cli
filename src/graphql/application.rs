use super::QueryClient;
use crate::error::APIError;
use graphql_client::{GraphQLQuery, Response};
use log::debug;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/application.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ApplicationQuery;

impl ApplicationQuery {
    pub(crate) async fn fetch(
        client: &QueryClient,
        name: &str,
    ) -> Result<Response<application_query::ResponseData>, APIError> {
        let variables = application_query::Variables {
            name: name.to_string(),
        };

        let response = client
            .call_api::<ApplicationQuery>(variables, |_, error| {
                Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{error}"),
                })
            })
            .await?;

        debug!("response: {:?}", &response);

        Ok(response)
    }
}

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
                    message: format!("{error}"),
                })
            })
            .await?;

        debug!("response: {:?}", &response);

        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::graphql::QueryClientBuilder;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_fetch_application_success_should_return_correct_application_info() {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::new()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

        let api_resp = r#"
{
  "data": {
    "application": {
      "basicInfo": {
        "deploymentStrategy": "basic",
        "deploymentTarget": "kubernetes",
        "links": [
          {
            "title": "Performance Dashboard",
            "url": "https://grafana.mv.tech/aaa"
          },
          {
            "title": "SLOs Dashboard",
            "url": "https://grafana.mv.tech/bbb"
          },
          {
            "title": "Honeycomb Telemetry",
            "url": "https://ui.honeycomb.io/mv/datasets/ccc"
          }
        ]
      },
      "name": "valid-application"
    }
  }
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = ApplicationQuery::fetch(&query_client, "valid-application").await;

        mock.assert();
        assert!(response.is_ok());

        let application = response.unwrap().data.unwrap().application.unwrap();
        assert_eq!(application.name, "valid-application");

        let basic_info = application.basic_info.unwrap();
        assert_eq!(basic_info.deployment_target, "kubernetes");
        assert_eq!(basic_info.deployment_strategy, "basic");

        assert_eq!(basic_info.links.unwrap().len(), 3);
    }

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
