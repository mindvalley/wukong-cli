use super::QueryClient;
use crate::error::APIError;
use graphql_client::{GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/kubernetes_pods.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct KubernetesPodsQuery;

impl KubernetesPodsQuery {
    pub(crate) async fn fetch(
        client: &QueryClient,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<kubernetes_pods_query::ResponseData>, APIError> {
        let variables = kubernetes_pods_query::Variables {
            application: application.to_string(),
            namespace: namespace.to_string(),
            version: version.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| {
                if error.message == "Unauthorized" {
                    return Err(APIError::ResponseError {
                        code: error.message,
                        message: format!("Unauthorized"),
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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/is_authorized.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct IsAuthorizedQuery;

impl IsAuthorizedQuery {
    pub(crate) async fn fetch(
        client: &QueryClient,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<is_authorized_query::ResponseData>, APIError> {
        let variables = is_authorized_query::Variables {
            application: application.to_string(),
            namespace: namespace.to_string(),
            version: version.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| {
                Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{error}"),
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
    async fn test_fetch_kubernetes_pods_list_success_should_return_kubernetes_pods_list() {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::default()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

        let api_resp = r#"
{
  "data": {
    "kubernetesPods": [
      {
        "hostIp": "10.0.128.11",
        "name": "the-blue-1",
        "ready": true
      },
      {
        "hostIp": null,
        "name": "the-blue-2",
        "ready": false
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

        let response =
            KubernetesPodsQuery::fetch(&query_client, "valid-application", "staging", "blue").await;

        mock.assert();
        assert!(response.is_ok());

        let kubernetes_pods = response.unwrap().data.unwrap().kubernetes_pods;
        assert_eq!(kubernetes_pods.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_kubernetes_pods_list_failed_with_unautorized_error_should_return_response_error(
    ) {
        let server = MockServer::start();
        let query_client = QueryClientBuilder::default()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

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

        let response =
            KubernetesPodsQuery::fetch(&query_client, "some-application", "prod", "blue").await;

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
        let query_client = QueryClientBuilder::default()
            .with_access_token("test_access_token".to_string())
            .with_api_url(server.base_url())
            .build()
            .unwrap();

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

        let response =
            IsAuthorizedQuery::fetch(&query_client, "valid-application", "staging", "blue").await;

        mock.assert();
        assert!(response.is_ok());

        let is_authorized = response.unwrap().data.unwrap().is_authorized;
        assert!(is_authorized);
    }
}
