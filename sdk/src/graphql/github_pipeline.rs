use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/github_pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct GithubPipelinesQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/github_cd_pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct GithubCdPipelineQuery;
