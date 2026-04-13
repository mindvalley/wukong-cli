use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/application_secrets.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ApplicationSecretsQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/mutation/update_application_secrets.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct UpdateApplicationSecrets;
