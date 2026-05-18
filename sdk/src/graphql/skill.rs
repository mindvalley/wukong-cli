use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/skills.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct SkillsList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/skill.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct SkillBySlug;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/check_skill_updates.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CheckSkillUpdates;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/mutation/publish_skill.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct PublishSkill;
