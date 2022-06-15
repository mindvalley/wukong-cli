use graphql_client::{reqwest::post_graphql, GraphQLQuery, Response};
use reqwest::header;

const URL: &'static str = "http://localhost:4000/api";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct PipelinesQuery;

impl PipelinesQuery {
    pub async fn fetch() -> Result<Response<pipelines_query::ResponseData>, reqwest::Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_static("Bearer valid"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let variables = pipelines_query::Variables {};

        let response = post_graphql::<PipelinesQuery, _>(&client, URL, variables).await?;
        Ok(response)
    }
}
