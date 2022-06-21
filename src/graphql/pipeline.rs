use super::auth_headers;
use graphql_client::{reqwest::post_graphql, GraphQLQuery, Response};

const URL: &'static str = "http://localhost:4000/api";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct PipelinesQuery;

impl PipelinesQuery {
    pub async fn fetch() -> Result<Response<pipelines_query::ResponseData>, reqwest::Error> {
        let client = reqwest::Client::builder()
            .default_headers(auth_headers())
            .build()?;

        let variables = pipelines_query::Variables {};

        let response = post_graphql::<PipelinesQuery, _>(&client, URL, variables).await?;
        println!("{:?}", response);
        todo!();
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct PipelineQuery;

impl PipelineQuery {
    pub async fn fetch(
        application: String,
    ) -> Result<Response<pipeline_query::ResponseData>, reqwest::Error> {
        let client = reqwest::Client::builder()
            .default_headers(auth_headers())
            .build()?;

        let variables = pipeline_query::Variables {
            application: Some(application),
        };

        // let variables = pipeline_query::Variables { application };

        let response = post_graphql::<PipelineQuery, _>(&client, URL, variables).await?;
        // if let Some(errors) = response.errors {
        //     return Err(anyhow::anyhow!(errors[0].clone()));
        // }
        println!("{:?}", response);
        todo!();
    }
}
