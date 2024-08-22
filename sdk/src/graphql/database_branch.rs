use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/mutation/create_database_branch.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CreateDatabaseBranch;
