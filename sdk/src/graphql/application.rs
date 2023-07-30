use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/application.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ApplicationQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/application_with_k8s_cluster.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ApplicationWithK8sClusterQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/applications.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ApplicationsQuery;

#[cfg(test)]
mod test {
    use crate::{WKClient, WKConfig};
    use httpmock::prelude::*;

    fn setup_wk_client(api_url: &str) -> WKClient {
        WKClient::new(WKConfig {
            api_url: api_url.to_string(),
            access_token: "test_access_token".to_string(),
        })
    }

    #[tokio::test]
    async fn test_fetch_application_success_should_return_correct_application_info() {
        let server = MockServer::start();
        let wk_clinet = setup_wk_client(&server.base_url());

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

        let response = wk_clinet.fetch_application("valid-application").await;

        mock.assert();
        assert!(response.is_ok());

        let application = response.unwrap().application.unwrap();
        assert_eq!(application.name, "valid-application");

        let basic_info = application.basic_info.unwrap();
        assert_eq!(basic_info.deployment_target, "kubernetes");
        assert_eq!(basic_info.deployment_strategy, "basic");

        assert_eq!(basic_info.links.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_fetch_application_list_success_should_return_application_list() {
        let server = MockServer::start();
        let wk_clinet = setup_wk_client(&server.base_url());

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

        let response = wk_clinet.fetch_applications().await;

        mock.assert();
        assert!(response.is_ok());

        let applications = response.unwrap().applications;
        assert_eq!(applications.len(), 3);
    }
}
