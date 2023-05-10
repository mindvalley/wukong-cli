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
        namespace: &str,
    ) -> Result<Response<kubernetes_pods_query::ResponseData>, APIError> {
        let variables = kubernetes_pods_query::Variables {
            namespace: namespace.to_string(),
        };

        let response = client
            .call_api::<Self>(variables, |_, error| match error.message.as_str() {
                _ => Err(APIError::ResponseError {
                    code: error.message.clone(),
                    message: format!("{error}"),
                }),
            })
            .await?;

        Ok(response)
    }
}
