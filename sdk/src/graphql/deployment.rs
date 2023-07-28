use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipelines.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelinesQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelineQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/cd_pipeline_for_rollback.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct CdPipelineForRollbackQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/mutation/execute_cd_pipeline.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ExecuteCdPipeline;

#[cfg(test)]
mod test {
    use crate::{error::APIError, graphql::GQLClient};

    use super::*;
    use base64::Engine;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_fetch_cd_pipeline_list_success_should_return_cd_pipeline_list() {
        let server = MockServer::start();
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

        let api_resp = r#"
{
  "data": {
    "cdPipelines": [
      {
        "deployedRef": null,
        "enabled": true,
        "environment": "prod",
        "lastDeployment": 1663161661001,
        "name": "pipeline-blue",
        "status": "TERMINAL",
        "version": "blue",
        "buildArtifact": "master-build-250"
      },
      {
        "deployedRef": null,
        "enabled": true,
        "environment": "prod",
        "lastDeployment": null,
        "name": "pipeline-green",
        "status": null,
        "version": "green",
        "buildArtifact": "master-build-1235"
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

        let response = gql_client
            .post_graphql::<CdPipelinesQuery, _>(
                server.base_url(),
                cd_pipelines_query::Variables {
                    application: "valid-application".to_string(),
                },
            )
            .await;

        mock.assert();
        assert!(response.is_ok());

        let cd_pipelines = response.unwrap().cd_pipelines;
        assert_eq!(cd_pipelines.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_cd_pipeline_list_failed_with_application_not_found_error_should_return_response_error(
    ) {
        let server = MockServer::start();
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

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
      "message": "application_not_found",
      "path": [
        "cdPipelines"
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

        let response = gql_client
            .post_graphql::<CdPipelinesQuery, _>(
                server.base_url(),
                cd_pipelines_query::Variables {
                    application: "invalid-application".to_string(),
                },
            )
            .await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "application_not_found");
                assert_eq!(message, "Application `invalid-application` not found.");
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }

    #[tokio::test]
    async fn test_fetch_cd_pipeline_for_rollback_success_should_return_cd_pipeline() {
        let server = MockServer::start();
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

        let api_resp = r#"
{
  "data": {
    "cdPipeline": {
      "buildArtifact": "main-build-10",
      "deployedRef": "d70dddc743d428f8de97610f27b75723992cbec4",
      "enabled": true,
      "environment": "prod",
      "lastDeployment": 1675324454720,
      "name": "valid-application-deployment-green",
      "previousDeployedArtifacts": [
        "main-build-10"
      ],
      "status": "SUCCEEDED",
      "version": "green"
    }
  }
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = gql_client
            .post_graphql::<CdPipelineForRollbackQuery, _>(
                server.base_url(),
                cd_pipeline_for_rollback_query::Variables {
                    application: "invalid-application".to_string(),
                    namespace: "prod".to_string(),
                    version: "green".to_string(),
                },
            )
            .await;

        mock.assert();
        assert!(response.is_ok());

        let cd_pipeline = response.unwrap().cd_pipeline.unwrap();
        let previous_deployed_artifacts = cd_pipeline.previous_deployed_artifacts;
        assert_eq!(previous_deployed_artifacts.len(), 1);
        assert_eq!(
            previous_deployed_artifacts.first().unwrap(),
            "main-build-10"
        );
    }

    #[tokio::test]
    async fn test_execute_cd_pipeline_success_should_return_deployment_url() {
        let server = MockServer::start();
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

        let api_resp = r#"
{
  "data": {
    "executeCdPipeline" : {
      "url": "https://cd_pipeline_deployment_url.com"
    }
  }
}"#;

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });

        let response = gql_client
            .post_graphql::<ExecuteCdPipeline, _>(
                server.base_url(),
                execute_cd_pipeline::Variables {
                    application: "valid-application".to_string(),
                    build_number: 0,
                    build_artifact_name: Some("main-build-100".to_string()),
                    namespace: "prod".to_string(),
                    version: "green".to_string(),
                    changelogs: Some(
                        base64::engine::general_purpose::STANDARD
                            .encode("This is a changelog.\n\nThis is a new changelog.\n"),
                    ),
                    send_to_slack: true,
                },
            )
            .await;

        mock.assert();
        assert!(response.is_ok());

        let deployment_url = response.unwrap().execute_cd_pipeline.url;
        assert_eq!(deployment_url, "https://cd_pipeline_deployment_url.com");
    }

    #[tokio::test]
    async fn test_execute_cd_pipeline_list_failed_with_deploy_for_this_build_is_currently_running_error_should_return_response_error(
    ) {
        let server = MockServer::start();
        let gql_client = GQLClient::with_authorization("test_access_token").unwrap();

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
      "message": "deploy_for_this_build_is_currently_running",
      "path": [
        "executeCdPipeline"
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

        let response = gql_client
            .post_graphql::<ExecuteCdPipeline, _>(
                server.base_url(),
                execute_cd_pipeline::Variables {
                    application: "valid-application".to_string(),
                    build_number: 0,
                    build_artifact_name: Some("main-build-100".to_string()),
                    namespace: "prod".to_string(),
                    version: "green".to_string(),
                    changelogs: Some(
                        base64::engine::general_purpose::STANDARD
                            .encode("This is a changelog.\n\nThis is a new changelog.\n"),
                    ),
                    send_to_slack: true,
                },
            )
            .await;

        mock.assert();
        assert!(response.is_err());

        match response.as_ref().unwrap_err() {
            APIError::ResponseError { code, message } => {
                assert_eq!(code, "deploy_for_this_build_is_currently_running");
                assert_eq!(message, "Cannot submit this deployment request, since there is another running deployment with the same arguments is running on Spinnaker.\nYou can wait a few minutes and submit the deployment again.");
            }
            _ => panic!("it should be returning ResponseError"),
        }
    }
}
