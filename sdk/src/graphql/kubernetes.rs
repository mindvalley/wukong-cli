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
    use super::*;
    use crate::{error::APIError, graphql::GQLClient};
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_fetch_kubernetes_pods_list_success_should_return_kubernetes_pods_list() {
        let server = MockServer::start();
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

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

        let response = gql_client
            .post_graphql::<KubernetesPodsQuery, _>(
                server.base_url(),
                kubernetes_pods_query::Variables {
                    application: "valid-application".to_string(),
                    namespace: "staging".to_string(),
                    version: "blue".to_string(),
                },
            )
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
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

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

        let response = gql_client
            .post_graphql::<KubernetesPodsQuery, _>(
                server.base_url(),
                kubernetes_pods_query::Variables {
                    application: "some-application".to_string(),
                    namespace: "prod".to_string(),
                    version: "blue".to_string(),
                },
            )
            .await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "Unauthorized");
                assert_eq!(message, "Unauthorized")
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }

    #[tokio::test]
    async fn test_fetch_is_authorized_success_should_return_boolean_value() {
        let server = MockServer::start();
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

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

        let response = gql_client
            .post_graphql::<IsAuthorizedQuery, _>(
                server.base_url(),
                is_authorized_query::Variables {
                    application: "valid-application".to_string(),
                    namespace: "staging".to_string(),
                    version: "blue".to_string(),
                },
            )
            .await;

        mock.assert();
        assert!(response.is_ok());

        let is_authorized = response.unwrap().is_authorized;
        assert!(is_authorized);
    }
}
