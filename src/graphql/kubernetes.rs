use super::QueryClient;
use crate::error::APIError;
use async_tungstenite::tungstenite::{client::IntoClientRequest, Message};
use futures::StreamExt;
use graphql_client::{GraphQLQuery, Response};
use graphql_ws_client::{
    graphql::{GraphQLClient, StreamingOperation},
    AsyncWebsocketClient, GraphQLClientClientBuilder, SubscriptionStream,
};

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
                        code: error.message.clone(),
                        message: error.message,
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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/mutation/deploy_livebook.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct DeployLivebook;

impl DeployLivebook {
    pub(crate) async fn mutate(
        client: &QueryClient,
        application: &str,
        namespace: &str,
        version: &str,
        name: &str,
        port: i64,
    ) -> Result<Response<deploy_livebook::ResponseData>, APIError> {
        let variables = deploy_livebook::Variables {
            application: application.to_string(),
            namespace: namespace.to_string(),
            version: version.to_string(),
            name: name.to_string(),
            port,
        };

        let response = client
            .call_api::<Self>(variables, |_, error| {
                if error.message == "Unauthorized" {
                    return Err(APIError::ResponseError {
                        code: error.message.clone(),
                        message: error.message,
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
    query_path = "src/graphql/mutation/destroy_livebook.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct DestroyLivebook;

impl DestroyLivebook {
    pub(crate) async fn mutate(
        client: &QueryClient,
        application: &str,
        namespace: &str,
        version: &str,
    ) -> Result<Response<destroy_livebook::ResponseData>, APIError> {
        let variables = destroy_livebook::Variables {
            application: application.to_string(),
            namespace: namespace.to_string(),
            version: version.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| {
                if error.message == "Unauthorized" {
                    return Err(APIError::ResponseError {
                        code: error.message.clone(),
                        message: error.message,
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
    query_path = "src/graphql/subscription/watch_livebook.graphql",
    response_derives = "Debug, Serialize, Deserialize, Clone"
)]
pub struct WatchLivebook;

// impl WatchLivebook {
//     pub(crate) async fn subscribe(
//         client: &QueryClient,
//         application: &str,
//         namespace: &str,
//         version: &str,
//         name: &str,
//     ) -> Result<
//         (
//             AsyncWebsocketClient<GraphQLClient, Message>,
//             SubscriptionStream<GraphQLClient, StreamingOperation<WatchLivebook>>,
//         ),
//         APIError,
//     > {
//         let variables = watch_livebook::Variables {
//             application: application.to_string(),
//             namespace: namespace.to_string(),
//             version: version.to_string(),
//             name: name.to_string(),
//         };
//
//         // let response = client
//         //     .call_api::<Self>(variables, |_, error| {
//         //         if error.message == "Unauthorized" {
//         //             return Err(APIError::ResponseError {
//         //                 code: error.message.clone(),
//         //                 message: error.message,
//         //             });
//         //         }
//         //
//         //         Err(APIError::ResponseError {
//         //             code: error.message.clone(),
//         //             message: format!("{error}"),
//         //         })
//         //     })
//         //     .await?;
//         //
//         // Ok(response)
//
//         // let configs = Configs::new()?;
//         //     let Some(token) = configs.root_config.user.token.clone() else {
//         //   bail!("Unauthorized. Please login with `railway login`")
//         // };
//
//         let token = "token";
//         let bearer = format!("Bearer {token}");
//         // let hostname = configs.get_host();
//         let mut request = format!("wss://localhost:4000/api")
//             .into_client_request()
//             .unwrap();
//
//         // request.headers_mut().insert(
//         //     "Sec-WebSocket-Protocol",
//         //     HeaderValue::from_str("graphql-transport-ws").unwrap(),
//         // );
//         // request
//         //     .headers_mut()
//         //     .insert("Authorization", HeaderValue::from_str(&bearer)?);
//
//         let (connection, _) = async_tungstenite::tokio::connect_async(request)
//             .await
//             .unwrap();
//
//         let (sink, stream) = connection.split::<Message>();
//
//         let mut client = GraphQLClientClientBuilder::new()
//             .build(stream, sink, TokioSpawner::current())
//             .await
//             .unwrap();
//         let stream = client
//             .streaming_operation(StreamingOperation::<WatchLivebook>::new(variables))
//             .await
//             .unwrap();
//
//         Ok((client, stream))
//     }
// }

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
