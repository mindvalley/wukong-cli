use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/github_workflow_templates.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct GithubWorkflowTemplatesQuery;
