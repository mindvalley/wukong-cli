use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/kubernetes_pods.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct KubernetesPodsQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/is_authorized.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct IsAuthorizedQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/mutation/deploy_livebook.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct DeployLivebook;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/mutation/destroy_livebook.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct DestroyLivebook;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/livebook_resource.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct LivebookResourceQuery;

#[cfg(test)]
mod test {
    use crate::{
        error::{APIError, WKError},
        WKClient, WKConfig,
    };
    use httpmock::prelude::*;

    fn setup_wk_client(api_url: &str) -> WKClient {
        WKClient::new(WKConfig {
            api_url: api_url.to_string(),
            access_token: "test_access_token".to_string(),
        })
    }

    #[tokio::test]
    async fn test_fetch_kubernetes_pods_list_success_should_return_kubernetes_pods_list() {
        let server = MockServer::start();
        let wk_client = setup_wk_client(&server.base_url());

        let api_resp = r#"
{
  "data": {
    "kubernetesPods": [
      {
        "hostIp": "10.0.128.11",
        "name": "the-blue-1",
        "ready": true,
        "labels": ["label1", "label2"]
      },
      {
        "hostIp": null,
        "name": "the-blue-2",
        "ready": false,
        "labels": ["label1", "label2"]
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

        let response = wk_client
            .fetch_kubernetes_pods("valid-application", "staging", "blue")
            .await;

        mock.assert();
        assert!(response.is_ok());

        let kubernetes_pods = response.unwrap().kubernetes_pods;
        assert_eq!(kubernetes_pods.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_kubernetes_pods_list_failed_with_unautorized_error_should_return_response_error(
    ) {
        let server = MockServer::start();
        let wk_client = setup_wk_client(&server.base_url());

        let api_resp = r#"
{
  "data": null,
  "errors": [
    {
      "locations": [
        {
          "column": 3,
          "line": 2
        }
      ],
      "message": "Unauthorized",
      "path": [
        "kubernetesPods"
      ]
    }
  ]
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = wk_client
            .fetch_kubernetes_pods("some-application", "prod", "blue")
            .await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            WKError::APIError(APIError::ResponseError { code, message }) => {
                assert_eq!(code, "Unauthorized");
                assert_eq!(message, "Unauthorized")
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }

    #[tokio::test]
    async fn test_fetch_is_authorized_success_should_return_boolean_value() {
        let server = MockServer::start();
        let wk_client = setup_wk_client(&server.base_url());

        let api_resp = r#"
{
  "data": {
    "isAuthorized": true
  }
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = wk_client
            .fetch_is_authorized("valid-application", "staging", "blue")
            .await;

        mock.assert();
        assert!(response.is_ok());

        let is_authorized = response.unwrap().is_authorized;
        assert!(is_authorized);
    }
}
